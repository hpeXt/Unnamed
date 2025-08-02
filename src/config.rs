//! 配置系统模块
//!
//! 统一处理 TOML 配置文件、环境变量、命令行参数

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use config::{Config as ConfigBuilder, Environment, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// 命令行参数
#[derive(Parser, Debug, Clone)]
#[command(name = "minimal-kernel")]
#[command(about = "最小化内核 - 长寿极客的数字孪生平台")]
#[command(version)]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// 日志级别
    #[arg(short, long, value_enum)]
    pub log_level: Option<LogLevel>,

    /// 数据库 URL
    #[arg(short, long)]
    pub database_url: Option<String>,

    /// 插件目录
    #[arg(short, long)]
    pub plugin_dir: Option<PathBuf>,

    /// 子命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 支持的命令
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// 运行内核
    Run,
    /// 列出插件
    ListPlugins,
    /// 插件信息
    PluginInfo {
        /// 插件名称
        name: String,
    },
    /// 重置配置
    ResetConfig,
}

/// 日志级别
#[derive(clap::ValueEnum, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Level::ERROR,
            LogLevel::Warn => Level::WARN,
            LogLevel::Info => Level::INFO,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        }
    }
}

/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 插件配置
    pub plugins: PluginConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 网络配置
    pub network: NetworkConfig,
    /// 身份管理配置
    pub identity: IdentityConfig,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// 数据库 URL
    pub url: String,
    /// 最大连接数
    pub max_connections: u32,
    /// 连接超时（秒）
    pub connect_timeout: u64,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    /// 插件目录
    pub directory: PathBuf,
    /// 自动加载插件
    pub auto_load: bool,
    /// 插件超时（毫秒）
    pub timeout_ms: u64,
    /// 最大内存（MB）
    pub max_memory_mb: u32,
    /// 启用的插件列表
    pub enabled: Vec<String>,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: LogLevel,
    /// 日志格式
    pub format: LogFormat,
    /// 日志输出目录
    pub directory: Option<PathBuf>,
    /// 日志文件大小限制（MB）
    pub max_file_size_mb: u32,
    /// 保留的日志文件数
    pub max_files: u32,
}

/// 日志格式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// 简洁格式
    Compact,
    /// 详细格式
    Full,
    /// JSON 格式
    Json,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// 是否启用 P2P
    pub p2p_enabled: bool,
    /// 监听端口
    pub listen_port: u16,
    /// 引导节点
    pub bootstrap_nodes: Vec<String>,
}

/// 身份管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IdentityConfig {
    /// 是否使用系统 keyring
    pub use_keyring: bool,
    /// keyring 访问超时时间（秒）
    pub keyring_timeout_secs: u64,
    /// 私钥文件路径（当 use_keyring 为 false 时使用）
    pub private_key_file: Option<PathBuf>,
    /// 是否允许从环境变量加载私钥
    pub allow_env_key: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:data.db".to_string(),
            max_connections: 5,
            connect_timeout: 30,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("plugins"),
            auto_load: true,
            timeout_ms: 5000,
            max_memory_mb: 128,
            enabled: vec![],
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Compact,
            directory: None,
            max_file_size_mb: 10,
            max_files: 5,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            p2p_enabled: false,
            listen_port: 8080,
            bootstrap_nodes: vec![],
        }
    }
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            use_keyring: true,        // 默认使用 keyring
            keyring_timeout_secs: 30, // 30秒超时，给用户足够时间输入密码
            private_key_file: None,
            allow_env_key: true, // 允许从环境变量加载
        }
    }
}

impl Config {
    /// 从多种配置源加载配置
    pub fn load() -> Result<Self> {
        let cli = Cli::parse();
        Self::load_with_cli(cli)
    }

    /// 使用指定的 CLI 参数加载配置
    pub fn load_with_cli(cli: Cli) -> Result<Self> {
        let mut builder = ConfigBuilder::builder();

        // 1. 首先加载默认配置
        builder = builder.add_source(config::Config::try_from(&Config::default())?);

        // 2. 加载系统配置文件
        if let Some(system_config) = Self::get_system_config_path() {
            if system_config.exists() {
                builder = builder.add_source(File::from(system_config));
            }
        }

        // 3. 加载用户配置文件
        if let Some(user_config) = Self::get_user_config_path() {
            if user_config.exists() {
                builder = builder.add_source(File::from(user_config));
            }
        }

        // 4. 加载指定的配置文件
        if let Some(config_path) = cli.config {
            if config_path.exists() {
                builder = builder.add_source(File::from(config_path));
            } else {
                return Err(anyhow!("配置文件不存在: {}", config_path.display()));
            }
        }

        // 5. 加载环境变量（前缀 MINIMAL_KERNEL_）
        builder = builder.add_source(
            Environment::with_prefix("MINIMAL_KERNEL")
                .prefix_separator("_")
                .separator("__"),
        );

        // 6. 构建配置
        let mut config: Config = builder.build()?.try_deserialize()?;

        // 7. 应用命令行参数覆盖
        if let Some(log_level) = cli.log_level {
            config.logging.level = log_level;
        }

        if let Some(database_url) = cli.database_url {
            config.database.url = database_url;
        }

        if let Some(plugin_dir) = cli.plugin_dir {
            config.plugins.directory = plugin_dir;
        }

        // 8. 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 获取系统配置文件路径
    pub fn get_system_config_path() -> Option<PathBuf> {
        Some(PathBuf::from("/etc/minimal-kernel/config.toml"))
    }

    /// 获取用户配置文件路径
    pub fn get_user_config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "minimal-kernel")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// 获取数据目录
    pub fn get_data_dir() -> Option<PathBuf> {
        ProjectDirs::from("", "", "minimal-kernel").map(|dirs| dirs.data_dir().to_path_buf())
    }

    /// 获取日志目录
    pub fn get_log_dir() -> Option<PathBuf> {
        ProjectDirs::from("", "", "minimal-kernel").map(|dirs| dirs.cache_dir().join("logs"))
    }

    /// 生成默认配置文件
    pub fn generate_default_config() -> Result<String> {
        let config = Config::default();
        toml::to_string_pretty(&config).map_err(|e| anyhow!("生成默认配置失败: {}", e))
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| anyhow!("序列化配置失败: {}", e))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置
    fn validate(&self) -> Result<()> {
        // 验证数据库 URL
        if self.database.url.is_empty() {
            return Err(anyhow!("数据库 URL 不能为空"));
        }

        // 验证插件目录
        if !self.plugins.directory.exists() {
            tracing::warn!(
                "插件目录不存在，将自动创建: {}",
                self.plugins.directory.display()
            );
            std::fs::create_dir_all(&self.plugins.directory)?;
        }

        // 验证日志目录
        if let Some(log_dir) = &self.logging.directory {
            if !log_dir.exists() {
                std::fs::create_dir_all(log_dir)?;
            }
        }

        // 验证网络配置
        if self.network.listen_port == 0 {
            return Err(anyhow!("监听端口不能为 0"));
        }

        Ok(())
    }

    /// 初始化日志系统
    pub fn init_logging(&self) -> Result<()> {
        let level_filter = EnvFilter::builder()
            .with_default_directive(Level::from(self.logging.level.clone()).into())
            .from_env_lossy();

        // 根据格式选择不同的初始化方式
        match self.logging.format {
            LogFormat::Compact => {
                let fmt_layer = fmt::layer().compact();
                if let Some(log_dir) = &self.logging.directory {
                    std::fs::create_dir_all(log_dir)?;
                    let file_appender =
                        tracing_appender::rolling::daily(log_dir, "minimal-kernel.log");
                    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
                    let file_layer = fmt::layer()
                        .compact()
                        .with_ansi(false)
                        .with_writer(non_blocking);
                    tracing_subscriber::registry()
                        .with(level_filter)
                        .with(fmt_layer)
                        .with(file_layer)
                        .init();
                } else {
                    tracing_subscriber::registry()
                        .with(level_filter)
                        .with(fmt_layer)
                        .init();
                }
            }
            LogFormat::Full => {
                let fmt_layer = fmt::layer();
                if let Some(log_dir) = &self.logging.directory {
                    std::fs::create_dir_all(log_dir)?;
                    let file_appender =
                        tracing_appender::rolling::daily(log_dir, "minimal-kernel.log");
                    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
                    let file_layer = fmt::layer().with_ansi(false).with_writer(non_blocking);
                    tracing_subscriber::registry()
                        .with(level_filter)
                        .with(fmt_layer)
                        .with(file_layer)
                        .init();
                } else {
                    tracing_subscriber::registry()
                        .with(level_filter)
                        .with(fmt_layer)
                        .init();
                }
            }
            LogFormat::Json => {
                // JSON格式使用不同的层
                let fmt_layer = fmt::layer().with_target(true).with_level(true);
                if let Some(log_dir) = &self.logging.directory {
                    std::fs::create_dir_all(log_dir)?;
                    let file_appender =
                        tracing_appender::rolling::daily(log_dir, "minimal-kernel.log");
                    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
                    let file_layer = fmt::layer().with_ansi(false).with_writer(non_blocking);
                    tracing_subscriber::registry()
                        .with(level_filter)
                        .with(fmt_layer)
                        .with(file_layer)
                        .init();
                } else {
                    tracing_subscriber::registry()
                        .with(level_filter)
                        .with(fmt_layer)
                        .init();
                }
            }
        }

        tracing::info!("日志系统已初始化，级别: {:?}", self.logging.level);
        Ok(())
    }

    /// 快速初始化日志系统（使用默认配置）
    pub fn init_default_logging() -> Result<()> {
        let config = Config::default();
        config.init_logging()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database.url, "sqlite:data.db");
        assert_eq!(config.plugins.directory, PathBuf::from("plugins"));
        assert!(matches!(config.logging.level, LogLevel::Info));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("database"));
        assert!(toml_str.contains("plugins"));
        assert!(toml_str.contains("logging"));
    }

    #[test]
    fn test_config_file_loading() {
        // 在CI环境中，使用更明确的临时目录路径
        let temp_dir = if std::env::var("CI").is_ok() {
            TempDir::new_in(".").unwrap_or_else(|_| TempDir::new().unwrap())
        } else {
            TempDir::new().unwrap()
        };
        let config_path = temp_dir.path().join("config.toml");

        // 创建测试配置文件
        let test_config = r#"
[database]
url = "sqlite:test.db"
max_connections = 10

[plugins]
directory = "test_plugins"
auto_load = false

[logging]
level = "debug"
format = "full"
        "#;

        std::fs::write(&config_path, test_config).unwrap();

        // 测试加载
        let builder = ConfigBuilder::builder()
            .add_source(File::from(config_path))
            .build()
            .unwrap();

        let config: Config = builder.try_deserialize().unwrap();
        assert_eq!(config.database.url, "sqlite:test.db");
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.plugins.directory, PathBuf::from("test_plugins"));
        assert!(!config.plugins.auto_load);
        assert!(matches!(config.logging.level, LogLevel::Debug));
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(Level::from(LogLevel::Error), Level::ERROR);
        assert_eq!(Level::from(LogLevel::Warn), Level::WARN);
        assert_eq!(Level::from(LogLevel::Info), Level::INFO);
        assert_eq!(Level::from(LogLevel::Debug), Level::DEBUG);
        assert_eq!(Level::from(LogLevel::Trace), Level::TRACE);
    }
}
