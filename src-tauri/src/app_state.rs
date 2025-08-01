use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 应用程序状态管理
#[derive(Clone)]
pub struct AppState {
    /// 应用是否已经完全初始化
    ready: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ready: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 标记应用已就绪
    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::Relaxed);
        tracing::info!("Application is now ready");
    }

    /// 检查应用是否已就绪
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
}

/// Tauri 命令：检查应用是否就绪
#[tauri::command]
pub fn is_app_ready(state: tauri::State<AppState>) -> bool {
    state.is_ready()
}
