//! 插件加载器
//!
//! 负责管理 WebAssembly 插件的加载、调用和卸载

use crate::identity::IdentityManager;
use crate::storage::Storage;
use anyhow::{anyhow, Result};
use extism::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use walkdir::WalkDir;

use super::dependency_resolver::DependencyResolver;
use super::host_functions::{build_plugin_with_host_functions, create_context_store, HostContext};
use super::manifest::{find_and_read_manifest, PluginManifest};
use super::message::Message;
use super::message_bus::MessageBusHandle;

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    /// 插件文件路径
    pub path: PathBuf,
    /// 文件大小
    pub file_size: u64,
    /// 最后修改时间
    pub modified: std::time::SystemTime,
    /// 是否已加载
    pub loaded: bool,

    // 新增字段：从 manifest.toml 读取
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 插件作者
    pub author: Option<String>,
    /// 必需依赖
    pub dependencies: Vec<String>,
    /// 可选依赖
    pub optional_dependencies: Vec<String>,
    /// 插件标签
    pub tags: Vec<String>,
    /// 最小内核版本要求
    pub min_kernel_version: Option<String>,
}

impl PluginInfo {
    /// 从 manifest 文件和路径创建插件信息
    pub fn from_manifest(path: &Path, manifest: PluginManifest) -> Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();
        let modified = metadata.modified()?;

        Ok(Self {
            name: manifest.plugin.name,
            path: path.to_path_buf(),
            file_size,
            modified,
            loaded: false, // 新创建时都是未加载状态
            version: manifest.plugin.version,
            description: manifest.plugin.description,
            author: manifest.plugin.author,
            dependencies: manifest.dependencies.requires,
            optional_dependencies: manifest.dependencies.optional,
            tags: manifest.metadata.tags,
            min_kernel_version: manifest.metadata.min_kernel_version,
        })
    }

    /// 从路径创建插件信息（不使用 manifest）
    pub fn from_path(path: &Path) -> Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();
        let modified = metadata.modified()?;

        let plugin_name = path
            .file_stem()
            .ok_or_else(|| anyhow!("Invalid plugin path: {}", path.display()))?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            name: plugin_name.clone(),
            path: path.to_path_buf(),
            file_size,
            modified,
            loaded: false,
            version: "unknown".to_string(),
            description: format!("{plugin_name} 插件"),
            author: None,
            dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            tags: Vec::new(),
            min_kernel_version: None,
        })
    }

    /// 检查是否兼容指定的内核版本
    pub fn is_compatible_with_kernel(&self, kernel_version: &str) -> bool {
        if let Some(min_version) = &self.min_kernel_version {
            // 简单的字符串比较，生产环境应使用 semver
            min_version.as_str() <= kernel_version
        } else {
            true
        }
    }

    /// 获取所有依赖（必需 + 可选）
    pub fn all_dependencies(&self) -> Vec<String> {
        let mut deps = self.dependencies.clone();
        deps.extend(self.optional_dependencies.clone());
        deps
    }

    /// 检查是否有依赖
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty() || !self.optional_dependencies.is_empty()
    }
}

/// 插件加载器
pub struct PluginLoader {
    /// 已加载的插件集合
    plugins: HashMap<String, Plugin>,
    /// 上下文存储
    context_store: UserData<super::host_functions::ContextStore>,
    /// 依赖解析器
    dependency_resolver: DependencyResolver,
}

impl std::fmt::Debug for PluginLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginLoader")
            .field("plugins", &self.plugins.keys().collect::<Vec<_>>())
            .field("context_store", &"<UserData>")
            .field("dependency_resolver", &"<DependencyResolver>")
            .finish()
    }
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new(
        msg_sender: mpsc::Sender<Message>,
        storage: Arc<Storage>,
        identity: Option<Arc<IdentityManager>>,
    ) -> Result<Self> {
        // 创建主机上下文（暂时不传递 MessageBus 引用）
        let host_context = HostContext::new(Some(storage), msg_sender, identity, None);
        let host_context = Arc::new(Mutex::new(host_context));

        // 创建上下文存储
        let context_store = create_context_store(host_context);

        Ok(Self {
            plugins: HashMap::new(),
            context_store,
            dependency_resolver: DependencyResolver::new(),
        })
    }

    /// 设置消息总线引用（在 Kernel 初始化后调用）
    pub fn set_message_bus(&mut self, message_bus: MessageBusHandle) {
        let store = self.context_store.get().unwrap();
        let store = store.lock().unwrap();
        let inner_store = store.lock().unwrap();

        if let Some(ctx_arc) = inner_store.get("context") {
            let mut ctx = ctx_arc.lock().unwrap();
            ctx.message_bus = Some(message_bus);
        }
    }

    /// 获取已加载插件的数量
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// 获取所有已加载插件的名称
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// 从文件加载插件
    pub fn load_plugin(&mut self, name: &str, path: &str) -> Result<()> {
        // 检查插件是否已存在
        if self.plugins.contains_key(name) {
            return Err(anyhow!("Plugin '{}' already loaded", name));
        }

        // 加载 WASM 文件
        let wasm = Wasm::file(path);
        let manifest = Manifest::new([wasm]);

        // 使用带有主机函数的插件构建器
        let plugin = build_plugin_with_host_functions(manifest, self.context_store.clone())?;

        // 存储插件
        self.plugins.insert(name.to_string(), plugin);

        Ok(())
    }

    /// 获取指定名称的插件
    pub fn get_plugin(&self, name: &str) -> Result<&Plugin> {
        self.plugins
            .get(name)
            .ok_or_else(|| anyhow!("Plugin '{}' not found", name))
    }

    /// 获取指定名称的插件（可变引用）
    pub fn get_plugin_mut(&mut self, name: &str) -> Result<&mut Plugin> {
        self.plugins
            .get_mut(name)
            .ok_or_else(|| anyhow!("Plugin '{}' not found", name))
    }

    /// 卸载指定插件
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        self.plugins
            .remove(name)
            .ok_or_else(|| anyhow!("Plugin '{}' not found", name))
            .map(|_| ())
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
        // 获取插件
        let plugin = self.get_plugin_mut(plugin_name)?;

        // 序列化输入
        let input_json = serde_json::to_string(&input)?;

        // 调用插件函数
        let output = plugin
            .call::<&str, &str>(function_name, &input_json)
            .map_err(|e| anyhow!("Failed to call plugin function '{}': {}", function_name, e))?;

        // 反序列化输出
        serde_json::from_str(output)
            .map_err(|e| anyhow!("Failed to deserialize plugin output: {}", e))
    }

    /// 调用插件函数（返回字符串）
    pub fn call_plugin_string(
        &mut self,
        plugin_name: &str,
        function_name: &str,
        input: &str,
    ) -> Result<String> {
        let plugin = self.get_plugin_mut(plugin_name)?;
        plugin
            .call::<&str, String>(function_name, input)
            .map_err(|e| anyhow!("Failed to call plugin function '{}': {}", function_name, e))
    }

    /// 扫描目录并自动加载插件
    pub fn scan_and_load_plugins(&mut self, plugin_dir: &Path) -> Result<Vec<String>> {
        let mut loaded_plugins = Vec::new();

        // 确保插件目录存在
        if !plugin_dir.exists() {
            std::fs::create_dir_all(plugin_dir)?;
            return Ok(loaded_plugins);
        }

        // 使用 walkdir 遍历目录查找 .wasm 文件
        for entry in WalkDir::new(plugin_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "wasm"))
        {
            let wasm_path = entry.path();
            let plugin_name = self.extract_plugin_name(wasm_path)?;

            // 跳过已加载的插件
            if self.plugins.contains_key(&plugin_name) {
                continue;
            }

            // 尝试加载插件
            match self.load_plugin(&plugin_name, wasm_path.to_str().unwrap()) {
                Ok(_) => {
                    loaded_plugins.push(plugin_name.clone());
                    tracing::info!("已加载插件: {} ({})", plugin_name, wasm_path.display());
                }
                Err(e) => {
                    tracing::warn!(
                        "加载插件失败: {} ({}): {}",
                        plugin_name,
                        wasm_path.display(),
                        e
                    );
                }
            }
        }

        Ok(loaded_plugins)
    }

    /// 从路径提取插件名称
    fn extract_plugin_name(&self, path: &Path) -> Result<String> {
        let file_stem = path
            .file_stem()
            .ok_or_else(|| anyhow!("Invalid plugin path: {}", path.display()))?;

        Ok(file_stem.to_string_lossy().to_string())
    }

    /// 发现插件文件但不加载
    pub fn discover_plugins(&self, plugin_dir: &Path) -> Result<Vec<PluginInfo>> {
        let mut plugins = Vec::new();

        if !plugin_dir.exists() {
            return Ok(plugins);
        }

        // 使用 walkdir 遍历目录查找 .wasm 文件
        for entry in WalkDir::new(plugin_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "wasm"))
        {
            let wasm_path = entry.path();

            // 尝试读取 manifest.toml，如果有的话
            let plugin_info = match find_and_read_manifest(wasm_path) {
                Ok(manifest) => {
                    tracing::debug!("使用 manifest 创建插件信息: {}", wasm_path.display());
                    PluginInfo::from_manifest(wasm_path, manifest)?
                }
                Err(_) => {
                    tracing::debug!("使用路径创建插件信息: {}", wasm_path.display());
                    PluginInfo::from_path(wasm_path)?
                }
            };

            // 检查插件是否已加载
            let mut info = plugin_info;
            info.loaded = self.plugins.contains_key(&info.name);

            plugins.push(info);
        }

        Ok(plugins)
    }

    /// 按配置加载插件
    pub fn load_plugins_from_config(
        &mut self,
        plugin_dir: &Path,
        enabled_plugins: &[String],
    ) -> Result<Vec<String>> {
        let mut loaded_plugins = Vec::new();

        // 如果没有指定启用的插件，扫描并加载所有插件
        if enabled_plugins.is_empty() {
            return self.scan_and_load_plugins(plugin_dir);
        }

        // 只加载指定的插件
        for plugin_name in enabled_plugins {
            // 尝试多种可能的路径
            let possible_paths = vec![
                plugin_dir.join(format!("{plugin_name}.wasm")),
                plugin_dir
                    .join(plugin_name)
                    .join(format!("{plugin_name}.wasm")),
                plugin_dir
                    .join(plugin_name)
                    .join("target/wasm32-unknown-unknown/release")
                    .join(format!("{plugin_name}.wasm")),
            ];

            let mut loaded = false;
            for path in possible_paths {
                if path.exists() {
                    match self.load_plugin(plugin_name, path.to_str().unwrap()) {
                        Ok(_) => {
                            loaded_plugins.push(plugin_name.clone());
                            loaded = true;
                            tracing::info!("已加载插件: {} ({})", plugin_name, path.display());
                            break;
                        }
                        Err(e) => {
                            tracing::warn!(
                                "加载插件失败: {} ({}): {}",
                                plugin_name,
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }

            if !loaded {
                tracing::warn!("未找到插件: {}", plugin_name);
            }
        }

        Ok(loaded_plugins)
    }

    /// 使用依赖解析加载插件
    pub fn load_plugins_with_dependencies(
        &mut self,
        plugin_dir: &Path,
        target_plugins: &[String],
    ) -> Result<Vec<String>> {
        // 发现所有插件
        let discovered_plugins = self.discover_plugins(plugin_dir)?;

        // 添加到依赖解析器
        self.dependency_resolver.add_plugins(discovered_plugins);

        // 解析加载顺序
        let load_order = self.dependency_resolver.resolve_order(target_plugins)?;

        let mut loaded_plugins = Vec::new();

        // 按顺序加载插件
        for plugin_name in load_order {
            if self.plugins.contains_key(&plugin_name) {
                continue; // 已加载
            }

            // 查找插件文件
            if let Some(plugin_path) = self.find_plugin_path(plugin_dir, &plugin_name)? {
                match self.load_plugin(&plugin_name, plugin_path.to_str().unwrap()) {
                    Ok(_) => {
                        loaded_plugins.push(plugin_name.clone());
                        tracing::info!("依赖加载插件: {} ({})", plugin_name, plugin_path.display());
                    }
                    Err(e) => {
                        tracing::warn!(
                            "依赖加载失败: {} ({}): {}",
                            plugin_name,
                            plugin_path.display(),
                            e
                        );
                    }
                }
            } else {
                tracing::warn!("未找到插件文件: {}", plugin_name);
            }
        }

        Ok(loaded_plugins)
    }

    /// 查找插件文件路径
    fn find_plugin_path(&self, plugin_dir: &Path, plugin_name: &str) -> Result<Option<PathBuf>> {
        // 尝试多种可能的路径
        let possible_paths = vec![
            plugin_dir.join(format!("{plugin_name}.wasm")),
            plugin_dir
                .join(plugin_name)
                .join(format!("{plugin_name}.wasm")),
            plugin_dir
                .join(plugin_name)
                .join("target/wasm32-unknown-unknown/release")
                .join(format!("{plugin_name}.wasm")),
        ];

        for path in possible_paths {
            if path.exists() {
                return Ok(Some(path));
            }
        }

        // 使用 walkdir 搜索
        for entry in WalkDir::new(plugin_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "wasm"))
        {
            let wasm_path = entry.path();
            if self.extract_plugin_name(wasm_path)? == plugin_name {
                return Ok(Some(wasm_path.to_path_buf()));
            }
        }

        Ok(None)
    }

    /// 检查依赖是否满足
    pub fn check_dependencies(&self, plugin_name: &str) -> bool {
        let available_plugins: Vec<String> = self.plugins.keys().cloned().collect();
        self.dependency_resolver
            .check_dependencies_satisfied(plugin_name, &available_plugins)
    }

    /// 获取插件的所有依赖
    pub fn get_plugin_dependencies(&self, plugin_name: &str) -> Result<Vec<String>> {
        self.dependency_resolver.get_all_dependencies(plugin_name)
    }

    /// 检查循环依赖
    pub fn check_circular_dependencies(&self) -> Result<()> {
        self.dependency_resolver.check_circular_dependencies()
    }

    /// 获取依赖解析统计信息
    pub fn get_dependency_stats(&self) -> (usize, usize, usize) {
        self.dependency_resolver.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_loader() -> PluginLoader {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        let storage = Arc::new(Storage::new(&db_url).await.unwrap());
        let (tx, _rx) = mpsc::channel(100);
        PluginLoader::new(tx, storage, None).unwrap()
    }

    #[tokio::test]
    async fn test_plugin_loader_creation() {
        let loader = create_test_loader().await;
        assert_eq!(loader.plugin_count(), 0);
        assert!(loader.plugin_names().is_empty());
    }

    #[tokio::test]
    async fn test_plugin_not_found() {
        let loader = create_test_loader().await;
        let result = loader.get_plugin("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
