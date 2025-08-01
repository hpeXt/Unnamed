// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod container;
mod bridge;
mod system_monitor;
mod app_state;
mod plugin_creator;

use container::{ContainerManager, RenderMode, ContainerPosition, ContainerSize, GridPosition, GridSize};
use bridge::KernelBridge;
use system_monitor::SystemMonitor;
use app_state::{AppState, is_app_ready};
use plugin_creator::{PluginConfig, CreatePluginResult};
use tauri::{State, Manager};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GridLayout {
    columns: u32,
    rows: u32,
    widgets: Vec<WidgetConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WidgetConfig {
    plugin_id: String,
    grid_area: GridArea,
}

#[derive(Debug, Serialize, Deserialize)]
struct GridArea {
    col: u32,
    row: u32,
    col_span: u32,
    row_span: u32,
}

// Tauri 命令：创建插件容器
#[tauri::command]
async fn create_plugin_container(
    plugin_id: String,
    render_mode: String,
    position: Option<ContainerPosition>,
    size: Option<ContainerSize>,
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<String, String> {
    // 解析渲染模式
    let mode = match render_mode.as_str() {
        "webview" => RenderMode::WebView,
        "canvas" => RenderMode::Canvas,
        "native" => RenderMode::Native,
        _ => return Err(format!("Invalid render mode: {}", render_mode)),
    };
    
    tracing::info!("Creating container for plugin: {}", plugin_id);
    let result = container_manager.create_container(&plugin_id, mode, position, size).await;
    match result {
        Ok(container_id) => {
            tracing::info!("Container created successfully: {}", container_id);
            Ok(container_id)
        }
        Err(e) => {
            tracing::error!("Failed to create container: {}", e);
            Err(format!("Failed to create container: {}", e))
        }
    }
}

// Tauri 命令：删除插件容器
#[tauri::command]
async fn remove_plugin_container(
    container_id: String,
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<(), String> {
    container_manager.remove_container(&container_id)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：列出所有容器
#[tauri::command]
async fn list_containers(
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<Vec<container::PluginContainer>, String> {
    Ok(container_manager.list_containers().await)
}

// Tauri 命令：从模板创建插件
#[tauri::command]
async fn create_plugin_from_template(
    config: PluginConfig,
) -> Result<CreatePluginResult, String> {
    plugin_creator::create_plugin_from_template(config)
        .await
        .map_err(|e| format!("创建插件失败: {}", e))
}

// Tauri 命令：保存当前布局
#[tauri::command]
async fn save_layout(
    name: String,
    kernel_bridge: State<'_, Arc<KernelBridge>>,
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<String, String> {
    // 获取当前所有内联组件
    let inline_widgets = container_manager.list_inline_widgets().await;
    
    // 转换为布局请求格式
    let mut widgets = Vec::new();
    for widget in inline_widgets {
        widgets.push(minimal_kernel::storage::layout::CreateWidgetRequest {
            widget_type: widget.widget_type,
            plugin_id: Some(widget.plugin_id),
            position_col: widget.position.col as i32,
            position_row: widget.position.row as i32,
            size_col_span: widget.size.col_span as i32,
            size_row_span: widget.size.row_span as i32,
            config: Some(widget.config),
        });
    }
    
    // 保存布局
    kernel_bridge.save_layout(name, widgets)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：列出所有布局
#[tauri::command]
async fn list_layouts(
    kernel_bridge: State<'_, Arc<KernelBridge>>,
) -> Result<Vec<serde_json::Value>, String> {
    kernel_bridge.list_layouts()
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：应用布局
#[tauri::command]
async fn apply_layout(
    layout_id: i64,
    kernel_bridge: State<'_, Arc<KernelBridge>>,
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<(), String> {
    // 获取布局详情
    let layout_widgets = kernel_bridge.get_layout_widgets(layout_id)
        .await
        .map_err(|e| e.to_string())?;
    
    // 清除现有内联组件
    let current_widgets = container_manager.list_inline_widgets().await;
    for widget in current_widgets {
        container_manager.remove_inline_widget(&widget.id).await.ok();
    }
    
    // 创建新组件
    for widget in layout_widgets {
        container_manager.create_inline_widget(
            &widget.widget_type,
            GridPosition {
                row: widget.position_row as u32,
                col: widget.position_col as u32,
            },
            GridSize {
                row_span: widget.size_row_span as u32,
                col_span: widget.size_col_span as u32,
            },
            widget.config.unwrap_or_default()
        ).await.ok();
    }
    
    Ok(())
}

// Tauri 命令：获取插件列表
#[tauri::command]
async fn get_plugins(
    kernel_bridge: State<'_, Arc<KernelBridge>>,
) -> Result<Vec<String>, String> {
    kernel_bridge.get_ui_plugins()
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：发送消息到插件
#[tauri::command]
async fn send_to_plugin(
    plugin_id: String,
    message: serde_json::Value,
    kernel_bridge: State<'_, Arc<KernelBridge>>,
) -> Result<(), String> {
    kernel_bridge.send_message(&plugin_id, message)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：订阅数据
#[tauri::command]
async fn subscribe_data(
    topic: String,
    plugin_id: String,
    kernel_bridge: State<'_, Arc<KernelBridge>>,
) -> Result<(), String> {
    kernel_bridge.subscribe(&topic, &plugin_id)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：重新加载插件
#[tauri::command]
async fn reload_plugins(
    app_handle: tauri::AppHandle,
    kernel_bridge: State<'_, Arc<KernelBridge>>,
) -> Result<Vec<String>, String> {
    kernel_bridge.load_plugins(app_handle)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：取消订阅
#[tauri::command]
async fn unsubscribe_data(
    topic: String,
    plugin_id: String,
    kernel_bridge: State<'_, Arc<KernelBridge>>,
) -> Result<(), String> {
    kernel_bridge.unsubscribe(&topic, &plugin_id)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：获取 UI 插件订阅信息
#[tauri::command]
async fn get_ui_subscriptions(
    kernel_bridge: State<'_, Arc<KernelBridge>>,
) -> Result<Vec<bridge::UISubscription>, String> {
    kernel_bridge.get_ui_subscriptions()
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：注销 UI 插件
#[tauri::command]
async fn unregister_ui_plugin(
    plugin_id: String,
    kernel_bridge: State<'_, Arc<KernelBridge>>,
) -> Result<(), String> {
    kernel_bridge.unregister_ui_plugin(&plugin_id)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：创建内联组件
#[tauri::command]
async fn create_inline_widget(
    widget_type: String,
    position: GridPosition,
    size: GridSize,
    config: serde_json::Value,
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<String, String> {
    container_manager.create_inline_widget(&widget_type, position, size, config)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：删除内联组件
#[tauri::command]
async fn remove_inline_widget(
    widget_id: String,
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<(), String> {
    container_manager.remove_inline_widget(&widget_id)
        .await
        .map_err(|e| e.to_string())
}

// Tauri 命令：列出所有内联组件
#[tauri::command]
async fn list_inline_widgets(
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<Vec<container::InlineWidget>, String> {
    Ok(container_manager.list_inline_widgets().await)
}

// Tauri 命令：更新内联组件配置
#[tauri::command]
async fn update_inline_widget(
    widget_id: String,
    config: serde_json::Value,
    container_manager: State<'_, Arc<ContainerManager>>,
) -> Result<(), String> {
    container_manager.update_inline_widget(&widget_id, config)
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    // 创建内核桥接
    let kernel_bridge = Arc::new(KernelBridge::new());
    let kernel_bridge_for_setup = kernel_bridge.clone();

    tauri::Builder::default()
        .setup(move |app| {
            // 获取 app handle 用于后续操作
            let app_handle = app.handle().clone();
            let app_handle_for_container = app.handle().clone();
            
            // 创建应用状态
            let app_state = AppState::new();
            app.manage(app_state.clone());
            
            // 创建容器管理器
            let container_manager = Arc::new(ContainerManager::new());
            
            // 同步设置 app handle - 使用 block_on 确保立即完成
            let container_manager_clone = container_manager.clone();
            tauri::async_runtime::block_on(async move {
                container_manager_clone.set_app_handle(app_handle_for_container).await;
            });
            
            // 将容器管理器放入应用状态
            app.manage(container_manager);
            
            // 创建系统监控器
            let system_monitor = Arc::new(SystemMonitor::new(app_handle.clone()));
            app.manage(system_monitor.clone());
            
            // 初始化内核
            let kernel_bridge_clone = kernel_bridge_for_setup.clone();
            let app_handle_for_listener = app_handle.clone();
            let app_state_for_init = app_state.clone();
            tauri::async_runtime::spawn(async move {
                match kernel_bridge_clone.initialize().await {
                    Ok(_) => {
                        tracing::info!("Kernel initialized successfully");
                        
                        // 加载插件
                        match kernel_bridge_clone.load_plugins(app_handle).await {
                            Ok(plugins) => {
                                tracing::info!("Loaded {} plugins: {:?}", plugins.len(), plugins);
                            },
                            Err(e) => {
                                tracing::error!("Failed to load plugins: {}", e);
                            }
                        }
                        
                        // 启动消息监听器
                        match kernel_bridge_clone.start_message_listener(app_handle_for_listener).await {
                            Ok(_) => {
                                tracing::info!("Message listener started successfully");
                            },
                            Err(e) => {
                                tracing::error!("Failed to start message listener: {}", e);
                            }
                        }

                        // 启动插件目录监视
                        match kernel_bridge_clone.start_plugin_watcher(app_handle_for_listener).await {
                            Ok(_) => tracing::info!("Plugin watcher started"),
                            Err(e) => tracing::error!("Failed to start plugin watcher: {}", e),
                        }
                        
                        // 标记应用已就绪
                        app_state_for_init.set_ready();
                    },
                    Err(e) => {
                        tracing::error!("Failed to initialize kernel: {}", e);
                        tracing::error!("Note: The app will continue without kernel functionality");
                        // 即使内核初始化失败，UI 部分仍然可用，所以也标记为就绪
                        app_state_for_init.set_ready();
                    }
                }
            });
            
            Ok(())
        })
        .manage(kernel_bridge)
        .invoke_handler(tauri::generate_handler![
            create_plugin_container,
            remove_plugin_container,
            list_containers,
            create_plugin_from_template,
            save_layout,
            list_layouts,
            apply_layout,
            get_plugins,
            send_to_plugin,
            subscribe_data,
            unsubscribe_data,
            reload_plugins,
            get_ui_subscriptions,
            unregister_ui_plugin,
            is_app_ready,
            create_inline_widget,
            remove_inline_widget,
            list_inline_widgets,
            update_inline_widget,
            system_monitor::get_system_stats,
            system_monitor::get_processes,
            system_monitor::start_system_monitoring,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}