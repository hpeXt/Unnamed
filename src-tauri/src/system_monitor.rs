use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{AppHandle, Emitter};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_usage: f32,
    pub disk_total: u64,
    pub disk_used: u64,
    pub process_count: usize,
    pub network_rx: u64,
    pub network_tx: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
}

pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
    app_handle: AppHandle,
}

impl SystemMonitor {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            app_handle,
        }
    }

    /// 获取系统统计信息
    pub async fn get_system_stats(&self) -> Result<SystemStats, String> {
        let mut system = self.system.write().await;
        system.refresh_all();

        // CPU 使用率（简化实现，使用平均值）
        let cpu_count = system.cpus().len() as f32;
        let cpu_usage = if cpu_count > 0.0 {
            system.cpus().iter()
                .map(|cpu| cpu.cpu_usage())
                .sum::<f32>() / cpu_count
        } else {
            0.0
        };

        // 内存信息
        let memory_total = system.total_memory();
        let memory_used = system.used_memory();
        let memory_usage = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        // 磁盘信息 (暂时使用固定值，sysinfo 0.30 API 不同)
        let disk_total = 1_000_000_000_000u64; // 1TB
        let disk_used = 500_000_000_000u64;    // 500GB
        let disk_usage = 50.0;

        // 进程数量
        let process_count = system.processes().len();

        // 网络信息（暂时使用固定值）
        let network_rx = 0u64;
        let network_tx = 0u64;

        Ok(SystemStats {
            cpu_usage,
            memory_usage,
            memory_total,
            memory_used,
            disk_usage,
            disk_total,
            disk_used,
            process_count,
            network_rx,
            network_tx,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        })
    }

    /// 获取进程列表
    pub async fn get_processes(&self) -> Result<Vec<ProcessInfo>, String> {
        let mut system = self.system.write().await;
        system.refresh_processes();

        let mut processes = Vec::new();
        for (pid, process) in system.processes() {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
            });
        }

        // 按内存使用量排序
        processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
        
        // 只返回前 20 个进程
        processes.truncate(20);

        Ok(processes)
    }

    /// 开始定期监控
    pub async fn start_monitoring(self: Arc<Self>, interval_ms: u64) {
        let monitor = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));
            loop {
                interval.tick().await;
                
                match monitor.get_system_stats().await {
                    Ok(stats) => {
                        // 发送系统统计信息到前端
                        let _ = monitor.app_handle.emit("system-stats", &stats);
                    }
                    Err(e) => {
                        eprintln!("Failed to get system stats: {}", e);
                    }
                }
            }
        });
    }
}

// Tauri 命令
#[tauri::command]
pub async fn get_system_stats(
    state: tauri::State<'_, Arc<SystemMonitor>>,
) -> Result<SystemStats, String> {
    state.get_system_stats().await
}

#[tauri::command]
pub async fn get_processes(
    state: tauri::State<'_, Arc<SystemMonitor>>,
) -> Result<Vec<ProcessInfo>, String> {
    state.get_processes().await
}

#[tauri::command]
pub async fn start_system_monitoring(
    state: tauri::State<'_, Arc<SystemMonitor>>,
    interval_ms: u64,
) -> Result<(), String> {
    let monitor = state.inner().clone();
    monitor.start_monitoring(interval_ms).await;
    Ok(())
}