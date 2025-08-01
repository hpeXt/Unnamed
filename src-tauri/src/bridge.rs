use anyhow::{anyhow, Result};
use minimal_kernel::kernel::message::Message;
use minimal_kernel::kernel::Kernel;
use minimal_kernel::storage::layout::{CreateWidgetRequest, LayoutManager, LayoutWidget};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

/// UI 插件订阅信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISubscription {
    pub plugin_id: String,
    pub topics: HashSet<String>,
}

/// 转发到 UI 的消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub topic: Option<String>,
    pub payload: Value,
    pub timestamp: u64,
}

pub struct KernelBridge {
    kernel: Arc<Mutex<Option<Kernel>>>,
    kernel_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// UI 插件订阅映射: plugin_id -> topics
    ui_subscriptions: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// 消息监听器任务句柄
    message_listener_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl KernelBridge {
    pub fn new() -> Self {
        Self {
            kernel: Arc::new(Mutex::new(None)),
            kernel_handle: Arc::new(Mutex::new(None)),
            ui_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            message_listener_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // 创建内核配置
        let mut config = minimal_kernel::config::Config::default();

        // 禁用自动加载插件以避免路径问题
        config.plugins.auto_load = false;

        // 使用应用数据目录中的数据库
        if let Some(data_dir) = minimal_kernel::config::Config::get_data_dir() {
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("kernel.db");
            config.database.url = format!("sqlite:{}", db_path.display());
        }

        // 禁用 keyring 避免权限问题
        config.identity.use_keyring = false;
        config.identity.allow_env_key = true;

        // 初始化内核
        let kernel = Kernel::new(config).await?;

        // 保存内核实例
        *self.kernel.lock().await = Some(kernel);

        // 启动内核任务（在后台运行）
        let _kernel_clone = self.kernel.clone();
        let handle = tokio::spawn(async move {
            // 这里不启动内核的 run() 方法，因为它会阻塞
            // 内核已经初始化完成，消息总线等已经在运行
            tracing::info!("Kernel bridge is ready");
        });

        *self.kernel_handle.lock().await = Some(handle);

        tracing::info!("Kernel bridge initialized successfully");
        Ok(())
    }

    /// 加载插件 - 支持开发模式和生产模式
    pub async fn load_plugins(&self, app_handle: tauri::AppHandle) -> Result<Vec<String>> {
        let mut kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_mut() {
            // 确定插件目录路径
            let plugin_dir = self.get_plugin_directory(&app_handle)?;

            tracing::info!("Loading plugins from: {}", plugin_dir.display());

            // 扫描并加载插件
            let loaded_plugins = kernel.scan_and_load_plugins(&plugin_dir)?;

            tracing::info!(
                "Loaded {} plugins: {:?}",
                loaded_plugins.len(),
                loaded_plugins
            );
            Ok(loaded_plugins)
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }

    /// 获取插件目录路径
    pub fn get_plugin_directory(&self, app_handle: &tauri::AppHandle) -> Result<PathBuf> {
        // 开发模式：使用项目根目录的 plugins 文件夹
        if cfg!(debug_assertions) {
            // 获取当前执行文件的目录，然后向上查找 plugins 目录
            let current_dir = std::env::current_dir()?;
            let plugin_dir = current_dir
                .parent()
                .ok_or_else(|| anyhow!("Cannot find parent directory"))?
                .join("plugins");

            if plugin_dir.exists() {
                return Ok(plugin_dir);
            }
        }

        // 生产模式：使用应用资源目录
        // 在 Tauri v2 中，使用 path() API
        if let Ok(resource_path) = app_handle.path().resource_dir() {
            let plugin_dir = resource_path.join("plugins");
            if plugin_dir.exists() {
                return Ok(plugin_dir);
            }
        }

        // 备选方案：使用应用数据目录
        if let Some(data_dir) = minimal_kernel::config::Config::get_data_dir() {
            let plugin_dir = data_dir.join("plugins");
            std::fs::create_dir_all(&plugin_dir)?;
            return Ok(plugin_dir);
        }

        Err(anyhow!("Cannot determine plugin directory"))
    }

    pub async fn get_ui_plugins(&self) -> Result<Vec<String>> {
        // 扫描插件目录，查找所有 UI 插件
        let mut ui_plugins = Vec::new();

        // 获取插件目录路径
        let plugin_dir = if cfg!(debug_assertions) {
            // 开发模式：使用项目根目录的 plugins 文件夹
            let current_dir = std::env::current_dir()?;
            current_dir
                .parent()
                .ok_or_else(|| anyhow!("Cannot find parent directory"))?
                .join("plugins")
        } else {
            // 生产模式：使用应用资源目录或数据目录
            if let Some(data_dir) = minimal_kernel::config::Config::get_data_dir() {
                data_dir.join("plugins")
            } else {
                return Err(anyhow!("Cannot determine plugin directory"));
            }
        };

        // 检查插件目录是否存在
        if !plugin_dir.exists() {
            tracing::warn!("Plugin directory does not exist: {:?}", plugin_dir);
            return Ok(ui_plugins);
        }

        // 扫描插件目录
        let entries = std::fs::read_dir(&plugin_dir)?;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // 检查是否是 UI 插件（以 ui- 开头）
                    if dir_name.starts_with("ui-") {
                        // 检查是否有 index.html 文件
                        let index_path = path.join("index.html");
                        if index_path.exists() {
                            // 提取插件 ID（去掉 ui- 前缀）
                            let plugin_id =
                                dir_name.strip_prefix("ui-").unwrap_or(dir_name).to_string();
                            tracing::info!("Found UI plugin: {}", &plugin_id);
                            ui_plugins.push(plugin_id);
                        }
                    }
                }
            }
        }

        tracing::info!("Found {} UI plugins in {:?}", ui_plugins.len(), plugin_dir);
        Ok(ui_plugins)
    }

    pub async fn send_message(&self, plugin_id: &str, message: Value) -> Result<()> {
        let kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_ref() {
            // 序列化 JSON 消息为字节
            let payload = serde_json::to_vec(&message)
                .map_err(|e| anyhow!("Failed to serialize message: {}", e))?;

            // 创建内核消息
            let msg = Message::new("tauri-ui".to_string(), plugin_id.to_string(), payload);

            // 获取消息总线句柄并发送消息
            let bus_handle = kernel.get_message_bus_handle();
            bus_handle.send_message(msg).await?;

            tracing::debug!("Sent message to plugin {}", plugin_id);
            Ok(())
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }

    pub async fn subscribe(&self, topic: &str, plugin_id: &str) -> Result<()> {
        let kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_ref() {
            // 通过消息总线订阅主题
            let bus_handle = kernel.get_message_bus_handle();
            bus_handle.subscribe_topic(plugin_id, topic);

            // 记录 UI 插件的订阅
            if plugin_id.starts_with("ui-") {
                let mut subscriptions = self.ui_subscriptions.write().await;
                subscriptions
                    .entry(plugin_id.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(topic.to_string());
            }

            tracing::debug!("Plugin {} subscribed to topic {}", plugin_id, topic);
            Ok(())
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }

    /// 启动消息监听器，监听内核消息并转发到 UI
    pub async fn start_message_listener(&self, app_handle: AppHandle) -> Result<()> {
        let kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_ref() {
            // 为桥接器注册一个接收器
            let bus_handle = kernel.get_message_bus_handle();
            let mut receiver = bus_handle.register_plugin("tauri-bridge".to_string());

            // 订阅所有消息（作为中转站）
            bus_handle.subscribe_topic("tauri-bridge", "*");

            let ui_subscriptions = self.ui_subscriptions.clone();
            let listener_handle = tokio::spawn(async move {
                tracing::info!("消息监听器已启动");

                while let Some(message) = receiver.recv().await {
                    // 检查是否有 UI 插件订阅了这个消息
                    let should_forward = {
                        let subs = ui_subscriptions.read().await;

                        // 检查点对点消息
                        if message.to.starts_with("ui-") {
                            true
                        } else if let Some(topic) = &message.topic {
                            // 检查主题消息
                            subs.values().any(|topics| topics.contains(topic))
                        } else {
                            false
                        }
                    };

                    if should_forward {
                        // 转换为 UI 消息格式
                        let ui_message = UIMessage {
                            id: uuid::Uuid::new_v4().to_string(),
                            from: message.from.clone(),
                            to: message.to.clone(),
                            topic: message.topic.clone(),
                            payload: serde_json::from_slice(&message.payload)
                                .unwrap_or(serde_json::Value::Null),
                            timestamp: message.timestamp.timestamp_millis() as u64,
                        };

                        // 通过 Tauri 事件系统发送到前端
                        if let Err(e) = app_handle.emit("kernel-message", &ui_message) {
                            tracing::error!("发送消息到前端失败: {}", e);
                        } else {
                            tracing::debug!("转发消息到前端: {:?}", ui_message.topic);
                        }
                    }
                }

                tracing::info!("消息监听器已停止");
            });

            *self.message_listener_handle.lock().await = Some(listener_handle);
            Ok(())
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }

    /// 取消 UI 插件的订阅
    pub async fn unsubscribe(&self, topic: &str, plugin_id: &str) -> Result<()> {
        let kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_ref() {
            // 从消息总线取消订阅
            let bus_handle = kernel.get_message_bus_handle();
            bus_handle.unsubscribe_topic(plugin_id, topic);

            // 更新 UI 订阅记录
            if plugin_id.starts_with("ui-") {
                let mut subscriptions = self.ui_subscriptions.write().await;
                if let Some(topics) = subscriptions.get_mut(plugin_id) {
                    topics.remove(topic);
                    if topics.is_empty() {
                        subscriptions.remove(plugin_id);
                    }
                }
            }

            tracing::debug!("Plugin {} unsubscribed from topic {}", plugin_id, topic);
            Ok(())
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }

    /// 获取所有 UI 插件的订阅信息
    pub async fn get_ui_subscriptions(&self) -> Result<Vec<UISubscription>> {
        let subscriptions = self.ui_subscriptions.read().await;
        Ok(subscriptions
            .iter()
            .map(|(plugin_id, topics)| UISubscription {
                plugin_id: plugin_id.clone(),
                topics: topics.clone(),
            })
            .collect())
    }

    /// 注销 UI 插件（移除所有订阅）
    pub async fn unregister_ui_plugin(&self, plugin_id: &str) -> Result<()> {
        let kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_ref() {
            let bus_handle = kernel.get_message_bus_handle();

            // 获取并移除所有订阅
            let topics_to_remove = {
                let mut subscriptions = self.ui_subscriptions.write().await;
                subscriptions.remove(plugin_id).unwrap_or_default()
            };

            // 从消息总线取消所有订阅
            for topic in topics_to_remove {
                bus_handle.unsubscribe_topic(plugin_id, &topic);
            }

            tracing::info!("UI 插件 {} 已注销", plugin_id);
            Ok(())
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }

    /// 保存布局
    pub async fn save_layout(
        &self,
        name: String,
        widgets: Vec<CreateWidgetRequest>,
    ) -> Result<String> {
        let kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_ref() {
            let storage = kernel.get_storage();
            let layout_manager = LayoutManager::new(storage.pool().clone());

            let layout = layout_manager.save_current_layout(name, widgets).await?;
            Ok(layout.id.to_string())
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }

    /// 列出所有布局
    pub async fn list_layouts(&self) -> Result<Vec<serde_json::Value>> {
        let kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_ref() {
            let storage = kernel.get_storage();
            let layout_manager = LayoutManager::new(storage.pool().clone());

            let layouts = layout_manager.list_layouts().await?;
            let json_layouts = layouts
                .into_iter()
                .map(|l| serde_json::to_value(&l).unwrap())
                .collect();
            Ok(json_layouts)
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }

    /// 获取布局的组件
    pub async fn get_layout_widgets(&self, layout_id: i64) -> Result<Vec<LayoutWidget>> {
        let kernel_guard = self.kernel.lock().await;
        if let Some(kernel) = kernel_guard.as_ref() {
            let storage = kernel.get_storage();
            let layout_manager = LayoutManager::new(storage.pool().clone());

            let widgets = layout_manager.get_layout_widgets(layout_id).await?;
            Ok(widgets)
        } else {
            Err(anyhow!("Kernel not initialized"))
        }
    }
}
