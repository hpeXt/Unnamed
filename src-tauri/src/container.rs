use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContainer {
    pub id: String,
    pub plugin_id: String,
    pub render_mode: RenderMode,
    pub webview_label: Option<String>,
    pub position: ContainerPosition,
    pub size: ContainerSize,
    pub status: ContainerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    WebView,
    Inline, // 新增：主窗口内组件
    Canvas,
    Native,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ContainerPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ContainerSize {
    pub width: f64,
    pub height: f64,
}

// 新增：网格位置（用于内联组件）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GridPosition {
    pub row: u32,
    pub col: u32,
}

// 新增：网格大小（占用格子数）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GridSize {
    pub row_span: u32,
    pub col_span: u32,
}

// 新增：内联组件数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineWidget {
    pub id: String,
    pub plugin_id: String,
    pub widget_type: String,
    pub position: GridPosition,
    pub size: GridSize,
    pub config: serde_json::Value,
    pub status: ContainerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    Loading,
    Active,
    Paused,
    Error(String),
}

pub struct ContainerManager {
    containers: Arc<RwLock<HashMap<String, PluginContainer>>>,
    inline_widgets: Arc<RwLock<HashMap<String, InlineWidget>>>, // 新增：内联组件管理
    app_handle: Arc<RwLock<Option<AppHandle>>>,
}

impl ContainerManager {
    pub fn new() -> Self {
        Self {
            containers: Arc::new(RwLock::new(HashMap::new())),
            inline_widgets: Arc::new(RwLock::new(HashMap::new())),
            app_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_app_handle(&self, app_handle: AppHandle) {
        let mut handle = self.app_handle.write().await;
        *handle = Some(app_handle);
    }

    pub async fn create_container(
        &self,
        plugin_id: &str,
        render_mode: RenderMode,
        position: Option<ContainerPosition>,
        size: Option<ContainerSize>,
    ) -> Result<String> {
        let container_id = Uuid::new_v4().to_string();

        let position = position.unwrap_or(ContainerPosition { x: 100.0, y: 100.0 });
        let size = size.unwrap_or(ContainerSize {
            width: 400.0,
            height: 300.0,
        });

        let mut container = PluginContainer {
            id: container_id.clone(),
            plugin_id: plugin_id.to_string(),
            render_mode: render_mode.clone(),
            webview_label: None,
            position,
            size,
            status: ContainerStatus::Loading,
        };

        match render_mode {
            RenderMode::WebView => {
                let webview_label = format!("plugin-{}-{}", plugin_id, &container_id[..8]);
                tracing::info!("Creating WebView window with label: {}", webview_label);

                // 创建实际的 WebView 窗口
                let app_handle_guard = self.app_handle.read().await;
                if let Some(app_handle) = app_handle_guard.as_ref() {
                    tracing::info!("App handle is available, creating window");
                    self.create_webview_window(
                        app_handle,
                        &webview_label,
                        plugin_id,
                        position,
                        size,
                    )
                    .await?;

                    container.webview_label = Some(webview_label);
                    container.status = ContainerStatus::Active;
                } else {
                    tracing::error!("App handle not set!");
                    return Err(anyhow!("App handle not set"));
                }
            }
            RenderMode::Inline => {
                // 内联模式不需要创建窗口，只需返回容器ID
                tracing::info!("Creating inline widget container for plugin: {}", plugin_id);
                container.status = ContainerStatus::Active;
            }
            RenderMode::Canvas => {
                // Canvas 模式暂时只记录状态
                container.status = ContainerStatus::Active;
            }
            RenderMode::Native => {
                return Err(anyhow!("Native render mode not implemented yet"));
            }
        }

        self.containers
            .write()
            .await
            .insert(container_id.clone(), container);
        Ok(container_id)
    }

    async fn create_webview_window(
        &self,
        app_handle: &AppHandle,
        label: &str,
        plugin_id: &str,
        position: ContainerPosition,
        size: ContainerSize,
    ) -> Result<()> {
        // 获取插件的 HTML 路径
        let plugin_url = self.get_plugin_url(plugin_id)?;
        tracing::info!(
            "Creating WebView window for plugin {} with URL: {}",
            plugin_id,
            plugin_url
        );

        // 创建新的 WebView 窗口
        let window =
            WebviewWindowBuilder::new(app_handle, label, WebviewUrl::App(plugin_url.into()))
                .title(format!("Plugin: {}", plugin_id))
                .position(position.x, position.y)
                .inner_size(size.width, size.height)
                .resizable(true)
                .decorations(true)
                .always_on_top(false)
                .build()?;

        // 注入插件 API 和 Tauri API
        window.eval(&format!(
            r#"
            window.__PLUGIN_ID__ = '{}';
            window.__CONTAINER_ID__ = '{}';
            
            // 创建插件 API 对象
            window.pluginAPI = {{
                pluginId: '{}',
                containerId: '{}',
                
                // Tauri 核心 API
                invoke: window.__TAURI__.core.invoke,
                
                // 消息订阅（使用后端的订阅管理）
                subscribe: async function(topic, callback) {{
                    // 告诉后端订阅
                    await window.__TAURI__.core.invoke('subscribe_data', {{
                        topic: topic,
                        pluginId: '{}'
                    }});
                    
                    // 监听内核消息
                    return window.__TAURI__.event.listen('kernel-message', (event) => {{
                        const message = event.payload;
                        if (message.topic === topic || message.to === '{}') {{
                            callback(message);
                        }}
                    }});
                }},
                
                // 发送消息到其他插件
                send: async function(targetPluginId, data) {{
                    await window.__TAURI__.core.invoke('send_to_plugin', {{
                        pluginId: targetPluginId,
                        message: data
                    }});
                }},
                
                // 前端广播（仅在前端插件间）
                broadcast: async function(topic, data) {{
                    await window.__TAURI__.event.emit('plugin-broadcast-' + topic, {{
                        from: '{}',
                        topic: topic,
                        data: data,
                        timestamp: Date.now()
                    }});
                }},
                
                // 监听前端广播
                onBroadcast: async function(topic, callback) {{
                    return window.__TAURI__.event.listen('plugin-broadcast-' + topic, (event) => {{
                        callback(event.payload);
                    }});
                }}
            }};
            
            console.log('Plugin {} loaded with Tauri API support');
            "#,
            plugin_id, label, plugin_id, label, plugin_id, plugin_id, plugin_id, plugin_id
        ))?;

        Ok(())
    }

    fn get_plugin_url(&self, plugin_id: &str) -> Result<String> {
        // 对于开发模式，使用本地文件路径
        // 对于生产模式，应该使用打包的资源路径
        if cfg!(debug_assertions) {
            // 开发模式：使用正确的相对路径
            // Tauri 运行在 src-tauri 目录，插件在上级目录的 plugins 文件夹
            let current_dir = std::env::current_dir()?;
            let plugin_path = current_dir
                .parent()
                .ok_or_else(|| anyhow!("Cannot find parent directory"))?
                .join("plugins")
                .join(format!("ui-{}", plugin_id))
                .join("index.html");

            if plugin_path.exists() {
                Ok(format!("file://{}", plugin_path.to_string_lossy()))
            } else {
                Err(anyhow!("Plugin HTML not found: {}", plugin_path.display()))
            }
        } else {
            // 生产模式：使用相对路径（会被 Tauri 解析）
            Ok(format!("../plugins/ui-{}/index.html", plugin_id))
        }
    }

    pub async fn get_container(&self, container_id: &str) -> Option<PluginContainer> {
        self.containers.read().await.get(container_id).cloned()
    }

    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        let container = self
            .containers
            .write()
            .await
            .remove(container_id)
            .ok_or_else(|| anyhow!("Container not found: {}", container_id))?;

        // 如果是 WebView，关闭窗口
        if let Some(webview_label) = container.webview_label {
            let app_handle_guard = self.app_handle.read().await;
            if let Some(app_handle) = app_handle_guard.as_ref() {
                if let Some(window) = app_handle.get_webview_window(&webview_label) {
                    window.close()?;
                }
            }
        }

        Ok(())
    }

    pub async fn list_containers(&self) -> Vec<PluginContainer> {
        self.containers.read().await.values().cloned().collect()
    }

    pub async fn update_container_status(
        &self,
        container_id: &str,
        status: ContainerStatus,
    ) -> Result<()> {
        let mut containers = self.containers.write().await;
        let container = containers
            .get_mut(container_id)
            .ok_or_else(|| anyhow!("Container not found: {}", container_id))?;
        container.status = status;
        Ok(())
    }

    pub async fn resize_container(&self, container_id: &str, size: ContainerSize) -> Result<()> {
        let mut containers = self.containers.write().await;
        let container = containers
            .get_mut(container_id)
            .ok_or_else(|| anyhow!("Container not found: {}", container_id))?;

        container.size = size.clone();

        // 如果是 WebView，调整窗口大小
        if let Some(webview_label) = &container.webview_label {
            let app_handle_guard = self.app_handle.read().await;
            if let Some(app_handle) = app_handle_guard.as_ref() {
                if let Some(window) = app_handle.get_webview_window(webview_label) {
                    window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: size.width as u32,
                        height: size.height as u32,
                    }))?;
                }
            }
        }

        Ok(())
    }

    // 新增：创建内联组件
    pub async fn create_inline_widget(
        &self,
        widget_type: &str,
        position: GridPosition,
        size: GridSize,
        config: serde_json::Value,
    ) -> Result<String> {
        let widget_id = Uuid::new_v4().to_string();

        let widget = InlineWidget {
            id: widget_id.clone(),
            plugin_id: format!("widget-{}", widget_type),
            widget_type: widget_type.to_string(),
            position,
            size,
            config,
            status: ContainerStatus::Active,
        };

        // 先保存widget的克隆用于emit
        let widget_clone = widget.clone();

        self.inline_widgets
            .write()
            .await
            .insert(widget_id.clone(), widget);

        // 通知前端创建组件
        if let Some(app_handle) = self.app_handle.read().await.as_ref() {
            app_handle.emit("create-inline-widget", &widget_clone)?;
        }

        Ok(widget_id)
    }

    // 新增：删除内联组件
    pub async fn remove_inline_widget(&self, widget_id: &str) -> Result<()> {
        self.inline_widgets
            .write()
            .await
            .remove(widget_id)
            .ok_or_else(|| anyhow!("Widget not found: {}", widget_id))?;

        // 通知前端删除组件
        if let Some(app_handle) = self.app_handle.read().await.as_ref() {
            app_handle.emit("remove-inline-widget", &widget_id)?;
        }

        Ok(())
    }

    // 新增：获取所有内联组件
    pub async fn list_inline_widgets(&self) -> Vec<InlineWidget> {
        self.inline_widgets.read().await.values().cloned().collect()
    }

    // 新增：更新内联组件配置
    pub async fn update_inline_widget(
        &self,
        widget_id: &str,
        config: serde_json::Value,
    ) -> Result<()> {
        let mut widgets = self.inline_widgets.write().await;
        let widget = widgets
            .get_mut(widget_id)
            .ok_or_else(|| anyhow!("Widget not found: {}", widget_id))?;

        widget.config = config.clone();

        // 通知前端更新组件
        if let Some(app_handle) = self.app_handle.read().await.as_ref() {
            app_handle.emit(
                "update-inline-widget",
                &serde_json::json!({
                    "id": widget_id,
                    "config": config
                }),
            )?;
        }

        Ok(())
    }
}
