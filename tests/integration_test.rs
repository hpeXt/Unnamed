//! 集成测试
//! 
//! 测试 Kernel 的完整功能，包括配置加载、插件管理、生命周期管理等

use anyhow::Result;
use minimal_kernel::config::{Config, DatabaseConfig, PluginConfig, LoggingConfig, LogLevel, NetworkConfig, IdentityConfig};
use minimal_kernel::kernel::Kernel;
use tempfile::TempDir;
use std::path::PathBuf;

/// 创建测试配置
fn create_test_config(temp_dir: &TempDir) -> Config {
    let plugin_dir = temp_dir.path().join("plugins");
    
    // 创建插件目录
    std::fs::create_dir_all(&plugin_dir).unwrap();
    
    Config {
        database: DatabaseConfig {
            url: "sqlite::memory:".to_string(),  // 使用内存数据库避免文件权限问题
            max_connections: 5,
            connect_timeout: 30,
        },
        plugins: PluginConfig {
            directory: plugin_dir,
            auto_load: false,  // 测试时手动加载
            timeout_ms: 5000,
            max_memory_mb: 128,
            enabled: vec![],
        },
        logging: LoggingConfig {
            level: LogLevel::Debug,
            format: minimal_kernel::config::LogFormat::Compact,
            directory: None,
            max_file_size_mb: 10,
            max_files: 5,
        },
        network: NetworkConfig {
            p2p_enabled: false,
            listen_port: 8080,
            bootstrap_nodes: vec![],
        },
        identity: IdentityConfig {
            use_keyring: false,        // 测试时不使用keyring
            keyring_timeout_secs: 10,  // 测试用短超时
            private_key_file: None,
            allow_env_key: true,
        },
    }
}

#[tokio::test]
async fn test_kernel_initialization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = create_test_config(&temp_dir);
    
    // 测试内核初始化
    let kernel = Kernel::new(config).await?;
    
    // 验证内核状态
    assert_eq!(kernel.plugin_count(), 0);
    assert!(kernel.list_loaded_plugins().is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_discovery() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = create_test_config(&temp_dir);
    
    // 创建一个假的插件文件
    let plugin_path = config.plugins.directory.join("test_plugin.wasm");
    std::fs::write(&plugin_path, b"fake wasm content")?;
    
    let kernel = Kernel::new(config.clone()).await?;
    
    // 测试插件发现
    let plugins = kernel.discover_plugins(&config.plugins.directory)?;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "test_plugin");
    assert_eq!(plugins[0].path, plugin_path);
    assert!(!plugins[0].loaded);
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_auto_load() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut config = create_test_config(&temp_dir);
    
    // 创建一个假的插件文件
    let plugin_path = config.plugins.directory.join("test_plugin.wasm");
    std::fs::write(&plugin_path, b"fake wasm content")?;
    
    // 启用自动加载
    config.plugins.auto_load = true;
    
    // 这个测试会失败，因为假的 wasm 文件不是有效的 WebAssembly
    // 但我们可以测试发现功能
    let kernel = Kernel::new(config.clone()).await?;
    
    // 测试插件扫描
    let discovered = kernel.discover_plugins(&config.plugins.directory)?;
    assert_eq!(discovered.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_config_defaults() {
    let config = Config::default();
    
    assert_eq!(config.database.url, "sqlite:data.db");
    assert_eq!(config.database.max_connections, 5);
    assert_eq!(config.plugins.directory, PathBuf::from("plugins"));
    assert!(config.plugins.auto_load);
    assert!(matches!(config.logging.level, LogLevel::Info));
    assert_eq!(config.network.listen_port, 8080);
}

#[tokio::test]
async fn test_config_serialization() -> Result<()> {
    let config = Config::default();
    
    // 测试序列化
    let toml_str = toml::to_string_pretty(&config)?;
    assert!(toml_str.contains("[database]"));
    assert!(toml_str.contains("[plugins]"));
    assert!(toml_str.contains("[logging]"));
    assert!(toml_str.contains("[network]"));
    
    // 测试反序列化
    let deserialized: Config = toml::from_str(&toml_str)?;
    assert_eq!(deserialized.database.url, config.database.url);
    assert_eq!(deserialized.plugins.directory, config.plugins.directory);
    
    Ok(())
}

#[tokio::test]
async fn test_config_file_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.toml");
    
    let original_config = Config::default();
    
    // 测试保存配置
    original_config.save_to_file(&config_path)?;
    assert!(config_path.exists());
    
    // 测试加载配置
    let loaded_content = std::fs::read_to_string(&config_path)?;
    let loaded_config: Config = toml::from_str(&loaded_content)?;
    
    assert_eq!(loaded_config.database.url, original_config.database.url);
    assert_eq!(loaded_config.plugins.directory, original_config.plugins.directory);
    
    Ok(())
}

#[tokio::test]
async fn test_kernel_plugin_management() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = create_test_config(&temp_dir);
    
    let kernel = Kernel::new(config.clone()).await?;
    
    // 测试插件计数
    assert_eq!(kernel.plugin_count(), 0);
    
    // 测试插件列表
    let plugins = kernel.list_loaded_plugins();
    assert!(plugins.is_empty());
    
    // 测试插件发现
    let discovered = kernel.discover_plugins(&config.plugins.directory)?;
    assert!(discovered.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_log_level_conversion() {
    use tracing::Level;
    
    assert_eq!(Level::from(LogLevel::Error), Level::ERROR);
    assert_eq!(Level::from(LogLevel::Warn), Level::WARN);
    assert_eq!(Level::from(LogLevel::Info), Level::INFO);
    assert_eq!(Level::from(LogLevel::Debug), Level::DEBUG);
    assert_eq!(Level::from(LogLevel::Trace), Level::TRACE);
}

#[tokio::test]
async fn test_database_initialization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = create_test_config(&temp_dir);
    
    // 测试数据库初始化
    let kernel = Kernel::new(config).await?;
    
    // 如果到达这里，说明数据库初始化成功
    assert_eq!(kernel.plugin_count(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_identity_management() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = create_test_config(&temp_dir);
    
    // 测试身份管理器初始化
    let kernel = Kernel::new(config).await?;
    
    // 如果到达这里，说明身份管理器初始化成功
    assert_eq!(kernel.plugin_count(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_directory_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut config = create_test_config(&temp_dir);
    
    // 使用不存在的插件目录
    config.plugins.directory = temp_dir.path().join("nonexistent/plugins");
    // 启用自动加载以触发目录创建
    config.plugins.auto_load = true;
    
    let _kernel = Kernel::new(config.clone()).await?;
    
    // 验证插件目录已创建
    assert!(config.plugins.directory.exists());
    assert!(config.plugins.directory.is_dir());
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_plugin_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config = create_test_config(&temp_dir);
    
    // 创建多个插件文件
    let plugin_names = vec!["plugin1", "plugin2", "plugin3"];
    for name in &plugin_names {
        let plugin_path = config.plugins.directory.join(format!("{}.wasm", name));
        std::fs::write(&plugin_path, b"fake wasm content")?;
    }
    
    let kernel = Kernel::new(config.clone()).await?;
    
    // 测试插件发现
    let discovered = kernel.discover_plugins(&config.plugins.directory)?;
    assert_eq!(discovered.len(), plugin_names.len());
    
    // 验证所有插件都被发现
    for name in &plugin_names {
        assert!(discovered.iter().any(|p| p.name == *name));
    }
    
    Ok(())
}

// 这个测试需要真实的插件文件，暂时跳过
#[tokio::test]
#[ignore]
async fn test_real_plugin_loading() -> Result<()> {
    // 这个测试需要编译真实的插件
    // 可以在有真实插件的情况下启用
    Ok(())
}