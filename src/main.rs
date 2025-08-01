use anyhow::Result;
use clap::Parser;
use minimal_kernel::config::{Cli, Commands, Config};
use minimal_kernel::kernel::Kernel;

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 加载配置
    let config = Config::load_with_cli(cli.clone())?;

    // 初始化日志系统
    config.init_logging()?;

    tracing::info!("Minimal Kernel Starting...");

    // 处理命令行子命令
    if let Some(command) = cli.command {
        handle_command(command, &config).await?;
        return Ok(());
    }

    // 初始化内核
    let kernel = Kernel::new(config).await?;

    tracing::info!("Minimal Kernel Ready!");

    // 运行内核（包含优雅关闭）
    kernel.run().await?;

    Ok(())
}

async fn handle_command(command: Commands, config: &Config) -> Result<()> {
    match command {
        Commands::Run => {
            // 这是默认行为，直接运行内核
            let kernel = Kernel::new(config.clone()).await?;
            kernel.run().await?;
        }
        Commands::ListPlugins => {
            // 列出所有插件
            let kernel = Kernel::new(config.clone()).await?;
            let plugins = kernel.discover_plugins(&config.plugins.directory)?;

            println!("发现的插件:");
            for plugin in plugins {
                let status = if plugin.loaded {
                    "已加载"
                } else {
                    "未加载"
                };
                println!(
                    "  {} - {} ({} bytes) [{}]",
                    plugin.name,
                    plugin.path.display(),
                    plugin.file_size,
                    status
                );
            }
        }
        Commands::PluginInfo { name } => {
            // 显示插件信息
            let mut kernel = Kernel::new(config.clone()).await?;

            // 尝试加载插件（如果还没加载）
            if !kernel.list_loaded_plugins().contains(&name.as_str()) {
                let plugins = kernel.discover_plugins(&config.plugins.directory)?;
                if let Some(plugin) = plugins.iter().find(|p| p.name == name) {
                    kernel.load_plugin(&name, plugin.path.to_str().unwrap())?;
                }
            }

            // 获取插件信息
            if let Ok(info) = kernel.call_plugin_string(&name, "info", "") {
                println!("插件信息: {info}");
            } else {
                println!("无法获取插件 '{name}' 的信息");
            }
        }
        Commands::ResetConfig => {
            // 重置配置
            let default_config = Config::default();
            if let Some(config_path) = Config::get_user_config_path() {
                default_config.save_to_file(&config_path)?;
                println!("配置已重置到: {}", config_path.display());
            } else {
                println!("无法确定配置文件路径");
            }
        }
    }

    Ok(())
}
