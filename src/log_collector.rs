use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: u64,
}

static LOGS: Lazy<Mutex<Vec<LogEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn add_log(level: &str, message: &str) {
    let entry = LogEntry {
        level: level.to_string(),
        message: message.to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };
    LOGS.lock().unwrap().push(entry);
}

pub fn get_logs() -> Vec<LogEntry> {
    LOGS.lock().unwrap().clone()
}

pub fn clear_logs() {
    LOGS.lock().unwrap().clear();
}
