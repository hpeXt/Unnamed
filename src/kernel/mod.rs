//! 内核模块
//!
//! 负责插件管理和消息总线

pub mod dependency_resolver;
pub mod host_functions;
pub mod manifest;
pub mod message;
pub mod message_bus;
pub mod plugin_loader;

pub use plugin_loader::PluginInfo;

use crate::config::Config;
use crate::identity::IdentityManager;
use crate::storage::Storage;
use anyhow::{anyhow, Result};
use message_bus::{create_message_bus, MessageBusHandle, MessageRouter};
use plugin_loader::PluginLoader;
use std::sync::Arc;

pub struct Kernel {
    /// 插件加载器
    plugin_loader: PluginLoader,
    /// 消息总线句柄（可克隆）
    message_bus_handle: MessageBusHandle,
    /// 消息路由器（Option 因为会被 take 出来运行）
    message_router: Option<MessageRouter>,
    /// 存储实例
    storage: Arc<Storage>,
    /// 身份管理器
    identity: Arc<IdentityManager>,
}

impl Kernel {
    pub async fn new(config: Config) -> Result<Self> {
        tracing::info!("正在初始化内核...");

        // 创建存储实例
        let storage = Arc::new(Storage::new(&config.database.url).await?);

        // 创建身份管理器，使用配置中的超时时间
        tracing::info!("正在初始化身份管理器...");
        let identity_config = config.identity.clone();
        let timeout_duration = std::time::Duration::from_secs(identity_config.keyring_timeout_secs);
        let timeout_secs = identity_config.keyring_timeout_secs; // 保存超时秒数用于错误消息

        let identity = if identity_config.use_keyring {
            // 使用 keyring 时需要在阻塞线程中执行
            tracing::info!("使用系统 keyring 管理身份密钥（超时: {}秒）", timeout_secs);
            match tokio::time::timeout(
                timeout_duration,
                tokio::task::spawn_blocking(move || {
                    tokio::runtime::Runtime::new()?.block_on(async {
                        IdentityManager::new_with_config(&identity_config).await
                    })
                }),
            )
            .await
            {
                Ok(Ok(Ok(identity))) => Arc::new(identity),
                Ok(Ok(Err(e))) => {
                    tracing::error!("身份管理器初始化失败: {}", e);
                    return Err(e);
                }
                Ok(Err(e)) => {
                    return Err(anyhow::anyhow!("身份管理器任务失败: {}", e));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "身份管理器初始化超时（{}秒）。如果系统要求输入密码，请增加 identity.keyring_timeout_secs 配置值",
                        timeout_secs
                    ));
                }
            }
        } else {
            // 不使用 keyring 时可以直接异步执行
            tracing::info!("不使用系统 keyring，从文件或环境变量加载身份密钥");
            Arc::new(IdentityManager::new_with_config(&identity_config).await?)
        };

        tracing::info!("主地址: {:?}", identity.get_master_address());

        // 创建新的消息系统
        tracing::info!("正在创建消息总线...");
        let (message_bus_handle, message_router) = create_message_bus(1000);

        // 获取消息发送器用于插件加载器
        let msg_sender = message_bus_handle.get_sender();

        // 创建插件加载器，传入消息发送器、存储和身份管理器
        tracing::info!("正在初始化插件加载器...");
        let mut plugin_loader =
            PluginLoader::new(msg_sender, storage.clone(), Some(identity.clone()))?;

        // 为插件加载器设置消息总线句柄
        plugin_loader.set_message_bus(message_bus_handle.clone());

        // 自动加载插件
        if config.plugins.auto_load {
            tracing::info!("正在扫描并加载插件...");
            let loaded_plugins = plugin_loader
                .load_plugins_from_config(&config.plugins.directory, &config.plugins.enabled)?;
            tracing::info!(
                "已自动加载 {} 个插件: {:?}",
                loaded_plugins.len(),
                loaded_plugins
            );
        }

        tracing::info!("内核初始化完成");
        Ok(Self {
            plugin_loader,
            message_bus_handle,
            message_router: Some(message_router),
            storage,
            identity,
        })
    }

    /// 使用默认配置创建 Kernel
    pub async fn new_with_defaults() -> Result<Self> {
        let config = Config::default();
        Self::new(config).await
    }

    pub async fn run(mut self) -> Result<()> {
        // 启动消息总线路由
        tracing::info!("启动消息总线...");

        // 获取关闭信号发送器（从新的 handle 获取）
        let shutdown_tx = self.message_bus_handle.get_shutdown_sender();

        // 保存必要的信息以供后续使用
        let plugin_count = self.plugin_count();

        // 取出消息路由器
        let router = self
            .message_router
            .take()
            .ok_or_else(|| anyhow!("消息路由器已被使用"))?;

        // 启动消息路由器
        let message_bus_handle = tokio::spawn(async move {
            router.run().await;
        });

        // 等待关闭信号
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("收到 Ctrl+C 信号，正在关闭...");
            }
            _ = Self::wait_for_term_signal() => {
                tracing::info!("收到 TERM 信号，正在关闭...");
            }
        }

        // 发送关闭信号给消息总线
        if let Err(e) = shutdown_tx.send(()).await {
            tracing::warn!("发送关闭信号失败: {}", e);
        }

        // 执行关闭清理
        tracing::info!("正在关闭内核...");
        tracing::info!("已卸载 {} 个插件", plugin_count);
        tracing::info!("内核已关闭");

        // 等待消息总线任务完成
        match tokio::time::timeout(std::time::Duration::from_secs(5), message_bus_handle).await {
            Ok(Ok(_)) => tracing::info!("消息总线已正常关闭"),
            Ok(Err(e)) => tracing::error!("消息总线任务失败: {}", e),
            Err(_) => {
                tracing::warn!("消息总线关闭超时，强制终止");
                // 这里不再调用 abort()，因为 handle 已经被移动了
                // timeout 会自动取消任务
            }
        }

        Ok(())
    }

    /// 等待 TERM 信号
    async fn wait_for_term_signal() {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            if let Ok(mut term) = signal(SignalKind::terminate()) {
                term.recv().await;
            }
        }

        #[cfg(not(unix))]
        {
            // Windows 不支持 SIGTERM，使用 Ctrl+C 替代
            let _ = tokio::signal::ctrl_c().await;
        }
    }

    /// 加载插件
    pub fn load_plugin(&mut self, name: &str, path: &str) -> Result<()> {
        self.plugin_loader.load_plugin(name, path)
    }

    /// 调用插件函数
    pub fn call_plugin<I, O>(
        &mut self,
        plugin_name: &str,
        function_name: &str,
        input: I,
    ) -> Result<O>
    where
        I: serde::Serialize,
        O: serde::de::DeserializeOwned,
    {
        self.plugin_loader
            .call_plugin(plugin_name, function_name, input)
    }

    /// 调用插件函数（字符串版本）
    pub fn call_plugin_string(
        &mut self,
        plugin_name: &str,
        function_name: &str,
        input: &str,
    ) -> Result<String> {
        self.plugin_loader
            .call_plugin_string(plugin_name, function_name, input)
    }

    /// 列出所有已加载的插件
    pub fn list_loaded_plugins(&self) -> Vec<&str> {
        self.plugin_loader.plugin_names()
    }

    /// 发现指定目录中的插件
    pub fn discover_plugins(&self, plugin_dir: &std::path::Path) -> Result<Vec<PluginInfo>> {
        self.plugin_loader.discover_plugins(plugin_dir)
    }

    /// 扫描并加载插件
    pub fn scan_and_load_plugins(&mut self, plugin_dir: &std::path::Path) -> Result<Vec<String>> {
        self.plugin_loader.scan_and_load_plugins(plugin_dir)
    }

    /// 获取插件数量
    pub fn plugin_count(&self) -> usize {
        self.plugin_loader.plugin_count()
    }

    /// 卸载插件
    pub fn unload_plugin(&mut self, plugin_name: &str) -> Result<()> {
        self.plugin_loader.unload_plugin(plugin_name)
    }

    /// 获取存储引用
    pub fn get_storage(&self) -> &Arc<Storage> {
        &self.storage
    }

    /// 获取身份管理器引用
    pub fn get_identity(&self) -> &Arc<IdentityManager> {
        &self.identity
    }

    /// 获取消息总线句柄
    pub fn get_message_bus_handle(&self) -> &MessageBusHandle {
        &self.message_bus_handle
    }

    /// 获取插件加载器的可变引用
    pub fn get_plugin_loader_mut(&mut self) -> &mut PluginLoader {
        &mut self.plugin_loader
    }

    /// 优雅关闭
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("正在关闭内核...");

        // 这里可以添加资源清理逻辑
        // 比如关闭数据库连接、停止消息总线等

        tracing::info!("内核已关闭");
        Ok(())
    }
}
