// Tauri API wrapper for v2
// 这个文件提供了一个简单的 Tauri API 封装

// 尝试获取 Tauri API
let tauriAPI = null;

// 检查是否在 Tauri 环境中
if (window.__TAURI__) {
    tauriAPI = window.__TAURI__;
} else {
    console.warn('Tauri API not available - running in web mode');
}

// 导出 invoke 函数
export async function invoke(cmd, args) {
    if (tauriAPI && tauriAPI.core && tauriAPI.core.invoke) {
        return await tauriAPI.core.invoke(cmd, args);
    } else {
        console.warn(`Cannot invoke command '${cmd}' - Tauri API not available`);
        throw new Error('Tauri API not available');
    }
}

// 检查是否在 Tauri 环境中
export function isTauri() {
    return tauriAPI !== null;
}

// 获取应用版本
export async function getVersion() {
    if (tauriAPI && tauriAPI.app) {
        return await tauriAPI.app.getVersion();
    }
    return 'Unknown';
}

// 导出 Tauri v2 事件 API
export async function listen(eventName, handler) {
    if (tauriAPI && tauriAPI.event && tauriAPI.event.listen) {
        return await tauriAPI.event.listen(eventName, handler);
    } else {
        console.warn(`Cannot listen to event '${eventName}' - Tauri API not available`);
        return () => {}; // 返回空的 unlisten 函数
    }
}

export async function once(eventName, handler) {
    if (tauriAPI && tauriAPI.event && tauriAPI.event.once) {
        return await tauriAPI.event.once(eventName, handler);
    } else {
        console.warn(`Cannot listen once to event '${eventName}' - Tauri API not available`);
        return () => {};
    }
}

export async function emit(eventName, payload) {
    if (tauriAPI && tauriAPI.event && tauriAPI.event.emit) {
        return await tauriAPI.event.emit(eventName, payload);
    } else {
        console.warn(`Cannot emit event '${eventName}' - Tauri API not available`);
    }
}

export async function emitTo(target, eventName, payload) {
    if (tauriAPI && tauriAPI.event && tauriAPI.event.emitTo) {
        return await tauriAPI.event.emitTo(target, eventName, payload);
    } else {
        console.warn(`Cannot emit to target '${target}' - Tauri API not available`);
    }
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
        return invoke('get_system_stats');
    }
    
    /**
     * 获取进程列表
     * @returns {Promise<Array>} 进程列表
     */
    async getProcesses() {
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
    if (isTauri()) {
        console.log('Tauri API modules:', {
            core: !!tauriAPI.core,
            event: !!tauriAPI.event,
            app: !!tauriAPI.app,
            window: !!tauriAPI.window
        });
    }
}