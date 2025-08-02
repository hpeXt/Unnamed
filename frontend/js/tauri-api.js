// Tauri API wrapper for v2
// 直接从 @tauri-apps/api 模块导入，这是标准做法

// 导入 Tauri API 模块
export { invoke } from '@tauri-apps/api/core';
export { listen, once, emit, emitTo } from '@tauri-apps/api/event';
export { getVersion } from '@tauri-apps/api/app';

// 检查是否在 Tauri 环境中
export function isTauri() {
    // 在 Tauri 环境中，window.__TAURI_INTERNALS__ 总是存在的
    return typeof window !== 'undefined' && window.__TAURI_INTERNALS__ !== undefined;
}

// 插件消息总线类 - 基于 Tauri API
export class PluginMessageBus {
    constructor(pluginId) {
        this.pluginId = pluginId;
        this.unlisteners = [];
        this.messageHandlers = new Map();
    }
    
    /**
     * 订阅主题并监听消息
     * @param {string} topic - 要订阅的主题
     * @param {function} callback - 消息处理回调
     * @returns {Promise<function>} - 取消监听函数
     */
    async subscribe(topic, callback) {
        // 导入需要的函数
        const { invoke } = await import('@tauri-apps/api/core');
        const { listen } = await import('@tauri-apps/api/event');
        
        // 1. 告诉后端订阅这个主题
        try {
            await invoke('subscribe_data', { 
                topic, 
                pluginId: this.pluginId 
            });
        } catch (error) {
            console.error(`Failed to subscribe to topic '${topic}':`, error);
        }
        
        // 2. 监听内核消息
        const unlisten = await listen('kernel-message', (event) => {
            const message = event.payload;
            // 过滤出该插件关心的消息
            if (message.topic === topic || message.to === this.pluginId) {
                callback(message);
            }
        });
        
        this.unlisteners.push(unlisten);
        this.messageHandlers.set(topic, callback);
        
        return unlisten;
    }
    
    /**
     * 取消订阅主题
     * @param {string} topic - 要取消订阅的主题
     */
    async unsubscribe(topic) {
        const { invoke } = await import('@tauri-apps/api/core');
        
        try {
            await invoke('unsubscribe_data', {
                topic,
                pluginId: this.pluginId
            });
        } catch (error) {
            console.error(`Failed to unsubscribe from topic '${topic}':`, error);
        }
        
        this.messageHandlers.delete(topic);
    }
    
    /**
     * 发送消息到其他插件
     * @param {string} targetPluginId - 目标插件 ID
     * @param {object} data - 消息数据
     */
    async send(targetPluginId, data) {
        const { invoke } = await import('@tauri-apps/api/core');
        
        try {
            await invoke('send_to_plugin', {
                pluginId: targetPluginId,
                message: data
            });
        } catch (error) {
            console.error(`Failed to send message to '${targetPluginId}':`, error);
        }
    }
    
    /**
     * 广播消息到主题
     * @param {string} topic - 主题
     * @param {object} data - 消息数据
     */
    async broadcast(topic, data) {
        const { emit } = await import('@tauri-apps/api/event');
        
        // 使用 Tauri 的 emit 功能进行前端广播
        await emit(`plugin-broadcast-${topic}`, {
            from: this.pluginId,
            topic,
            data,
            timestamp: Date.now()
        });
    }
    
    /**
     * 监听广播消息
     * @param {string} topic - 主题
     * @param {function} callback - 消息处理回调
     */
    async onBroadcast(topic, callback) {
        const { listen } = await import('@tauri-apps/api/event');
        
        const unlisten = await listen(`plugin-broadcast-${topic}`, (event) => {
            callback(event.payload);
        });
        this.unlisteners.push(unlisten);
        return unlisten;
    }
    
    /**
     * 清理所有监听器
     */
    cleanup() {
        this.unlisteners.forEach(unlisten => {
            try {
                unlisten();
            } catch (error) {
                console.error('Error cleaning up listener:', error);
            }
        });
        this.unlisteners = [];
        this.messageHandlers.clear();
    }
}

// 系统监控 API
export class SystemMonitor {
    constructor() {
        this.monitoring = false;
    }
    
    /**
     * 获取系统统计信息
     * @returns {Promise<object>} 系统统计数据
     */
    async getStats() {
        const { invoke } = await import('@tauri-apps/api/core');
        return invoke('get_system_stats');
    }
    
    /**
     * 获取进程列表
     * @returns {Promise<Array>} 进程列表
     */
    async getProcesses() {
        const { invoke } = await import('@tauri-apps/api/core');
        return invoke('get_processes');
    }
    
    /**
     * 开始系统监控
     * @param {number} intervalMs - 监控间隔（毫秒）
     * @param {function} callback - 监控数据回调函数
     */
    async startMonitoring(intervalMs = 1000, callback) {
        if (this.monitoring) {
            console.warn('System monitoring already started');
            return;
        }
        
        this.monitoring = true;
        
        const { invoke } = await import('@tauri-apps/api/core');
        const { listen } = await import('@tauri-apps/api/event');
        
        // 启动后端监控
        await invoke('start_system_monitoring', { intervalMs });
        
        // 监听系统统计事件
        this.unlisten = await listen('system-stats', (event) => {
            if (callback) {
                callback(event.payload);
            }
        });
    }
    
    /**
     * 停止系统监控
     */
    stopMonitoring() {
        if (this.unlisten) {
            this.unlisten();
            this.unlisten = null;
        }
        this.monitoring = false;
    }
}

// 用于调试的辅助函数
export function debugTauriAPI() {
    console.log('Tauri API available:', isTauri());
    console.log('Window.__TAURI_INTERNALS__:', window.__TAURI_INTERNALS__);
    if (window.__TAURI__) {
        console.log('Window.__TAURI__ (legacy) exists:', !!window.__TAURI__);
    }
}