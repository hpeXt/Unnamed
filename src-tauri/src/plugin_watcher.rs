use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::Arc;
use tauri::AppHandle;
use tracing;

use crate::bridge::KernelBridge;
use once_cell::sync::OnceCell;

pub struct PluginWatcher {
    _watcher: RecommendedWatcher,
}

static WATCHER: OnceCell<PluginWatcher> = OnceCell::new();

impl PluginWatcher {
    pub fn new(app_handle: AppHandle, kernel_bridge: Arc<KernelBridge>) -> notify::Result<Self> {
        let plugin_dir = kernel_bridge.get_plugin_directory(&app_handle)?;

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                if matches!(
                    event.kind,
                    EventKind::Modify(_)
                        | EventKind::Create(_)
                        | EventKind::Remove(_)
                        | EventKind::Any
                ) {
                    let bridge = kernel_bridge.clone();
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = bridge.load_plugins(handle).await {
                            tracing::error!("Plugin hot reload failed: {}", e);
                        } else {
                            tracing::info!("Plugins hot reloaded");
                        }
                    });
                }
            }
        })?;
        watcher.configure(Config::PreciseEvents(true))?;
        watcher.watch(&plugin_dir, RecursiveMode::Recursive)?;

        Ok(Self { _watcher: watcher })
    }
}

pub fn init(app_handle: AppHandle, kernel_bridge: Arc<KernelBridge>) {
    if WATCHER.get().is_some() {
        return;
    }
    match PluginWatcher::new(app_handle, kernel_bridge) {
        Ok(w) => {
            let _ = WATCHER.set(w);
            tracing::info!("Plugin watcher started");
        }
        Err(e) => {
            tracing::error!("Failed to start plugin watcher: {}", e);
        }
    }
}
