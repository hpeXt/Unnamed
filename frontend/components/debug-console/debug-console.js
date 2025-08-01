/**
 * 插件调试控制台
 * 提供插件日志查看、筛选和管理功能
 */

export class DebugConsoleManager {
    constructor() {
        this.logs = [];
        this.filters = {
            level: 'all',
            plugin: 'all',
            search: ''
        };
        this.autoRefresh = true;
        this.refreshInterval = 2000; // 2秒刷新一次
        this.refreshTimer = null;
        
        this.init();
    }
    
    async init() {
        await this.createUI();
        await this.loadLogs();
        
        if (this.autoRefresh) {
            this.startAutoRefresh();
        }
    }
    
    createUI() {
        const container = document.createElement('div');
        container.className = 'debug-console-container';
        container.innerHTML = `
            <div class="debug-console">
                <div class="debug-header">
                    <h2>🔧 插件调试控制台</h2>
                    <div class="header-controls">
                        <button class="btn-refresh" title="刷新日志">
                            <span class="icon">🔄</span>
                        </button>
                        <button class="btn-clear" title="清空日志">
                            <span class="icon">🗑️</span>
                        </button>
                        <button class="btn-export" title="导出日志">
                            <span class="icon">💾</span>
                        </button>
                        <label class="auto-refresh">
                            <input type="checkbox" id="auto-refresh" checked>
                            <span>自动刷新</span>
                        </label>
                    </div>
                </div>
                
                <div class="debug-filters">
                    <div class="filter-group">
                        <label>级别：</label>
                        <select id="filter-level">
                            <option value="all">全部</option>
                            <option value="debug">调试</option>
                            <option value="info">信息</option>
                            <option value="warn">警告</option>
                            <option value="error">错误</option>
                        </select>
                    </div>
                    
                    <div class="filter-group">
                        <label>插件：</label>
                        <select id="filter-plugin">
                            <option value="all">全部插件</option>
                        </select>
                    </div>
                    
                    <div class="filter-group search-group">
                        <label>搜索：</label>
                        <input type="text" id="filter-search" placeholder="搜索日志内容...">
                    </div>
                    
                    <div class="log-stats">
                        <span class="stat-item">
                            <span class="stat-label">总计：</span>
                            <span class="stat-value" id="stat-total">0</span>
                        </span>
                        <span class="stat-item">
                            <span class="stat-label">显示：</span>
                            <span class="stat-value" id="stat-shown">0</span>
                        </span>
                    </div>
                </div>
                
                <div class="debug-logs-container">
                    <div class="debug-logs" id="debug-logs">
                        <div class="empty-state">
                            <span class="empty-icon">📋</span>
                            <p>暂无日志记录</p>
                            <p class="empty-hint">插件运行时的日志将显示在这里</p>
                        </div>
                    </div>
                </div>
                
                <div class="debug-footer">
                    <div class="footer-info">
                        <span id="last-update">上次更新: --</span>
                    </div>
                    <div class="footer-actions">
                        <button class="btn-close">关闭</button>
                    </div>
                </div>
            </div>
        `;
        
        document.body.appendChild(container);
        this.container = container;
        this.setupEventListeners();
    }
    
    setupEventListeners() {
        // 刷新按钮
        this.container.querySelector('.btn-refresh').addEventListener('click', () => {
            this.loadLogs();
        });
        
        // 清空按钮
        this.container.querySelector('.btn-clear').addEventListener('click', () => {
            this.clearLogs();
        });
        
        // 导出按钮
        this.container.querySelector('.btn-export').addEventListener('click', () => {
            this.exportLogs();
        });
        
        // 自动刷新切换
        this.container.querySelector('#auto-refresh').addEventListener('change', (e) => {
            this.autoRefresh = e.target.checked;
            if (this.autoRefresh) {
                this.startAutoRefresh();
            } else {
                this.stopAutoRefresh();
            }
        });
        
        // 级别筛选
        this.container.querySelector('#filter-level').addEventListener('change', (e) => {
            this.filters.level = e.target.value;
            this.applyFilters();
        });
        
        // 插件筛选
        this.container.querySelector('#filter-plugin').addEventListener('change', (e) => {
            this.filters.plugin = e.target.value;
            this.applyFilters();
        });
        
        // 搜索框
        this.container.querySelector('#filter-search').addEventListener('input', (e) => {
            this.filters.search = e.target.value.toLowerCase();
            this.applyFilters();
        });
        
        // 关闭按钮
        this.container.querySelector('.btn-close').addEventListener('click', () => {
            this.close();
        });
        
        // ESC 键关闭
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.container) {
                this.close();
            }
        });
    }
    
    async loadLogs() {
        try {
            const logs = await window.__TAURI__.core.invoke('get_plugin_logs');
            this.logs = logs;
            
            // 更新插件筛选器选项
            this.updatePluginFilter();
            
            // 应用筛选并显示
            this.applyFilters();
            
            // 更新最后更新时间
            this.updateLastUpdateTime();
            
            // 滚动到底部
            this.scrollToBottom();
        } catch (error) {
            console.error('加载日志失败:', error);
            this.showError('加载日志失败: ' + error);
        }
    }
    
    async clearLogs() {
        if (!confirm('确定要清空所有日志吗？')) {
            return;
        }
        
        try {
            await window.__TAURI__.core.invoke('clear_plugin_logs');
            this.logs = [];
            this.applyFilters();
            this.showSuccess('日志已清空');
        } catch (error) {
            console.error('清空日志失败:', error);
            this.showError('清空日志失败: ' + error);
        }
    }
    
    exportLogs() {
        const filteredLogs = this.getFilteredLogs();
        const content = filteredLogs.map(log => {
            const time = new Date(log.timestamp).toLocaleString();
            return `[${time}] [${log.level.toUpperCase()}] ${log.message}`;
        }).join('\n');
        
        const blob = new Blob([content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `plugin-logs-${Date.now()}.txt`;
        a.click();
        URL.revokeObjectURL(url);
        
        this.showSuccess('日志已导出');
    }
    
    updatePluginFilter() {
        const plugins = new Set();
        this.logs.forEach(log => {
            // 从日志消息中提取插件 ID（假设格式为 [PLUGIN_ID] ...）
            const match = log.message.match(/^\[([^\]]+)\]/);
            if (match) {
                plugins.add(match[1]);
            }
        });
        
        const select = this.container.querySelector('#filter-plugin');
        const currentValue = select.value;
        
        // 清空现有选项（保留"全部插件"）
        select.innerHTML = '<option value="all">全部插件</option>';
        
        // 添加插件选项
        Array.from(plugins).sort().forEach(plugin => {
            const option = document.createElement('option');
            option.value = plugin;
            option.textContent = plugin;
            select.appendChild(option);
        });
        
        // 恢复之前的选择
        if (currentValue && select.querySelector(`option[value="${currentValue}"]`)) {
            select.value = currentValue;
        }
    }
    
    getFilteredLogs() {
        return this.logs.filter(log => {
            // 级别筛选
            if (this.filters.level !== 'all' && log.level !== this.filters.level) {
                return false;
            }
            
            // 插件筛选
            if (this.filters.plugin !== 'all') {
                const match = log.message.match(/^\[([^\]]+)\]/);
                if (!match || match[1] !== this.filters.plugin) {
                    return false;
                }
            }
            
            // 搜索筛选
            if (this.filters.search && !log.message.toLowerCase().includes(this.filters.search)) {
                return false;
            }
            
            return true;
        });
    }
    
    applyFilters() {
        const filteredLogs = this.getFilteredLogs();
        const logsContainer = this.container.querySelector('#debug-logs');
        
        if (filteredLogs.length === 0) {
            logsContainer.innerHTML = `
                <div class="empty-state">
                    <span class="empty-icon">🔍</span>
                    <p>没有匹配的日志</p>
                    <p class="empty-hint">尝试调整筛选条件</p>
                </div>
            `;
        } else {
            logsContainer.innerHTML = filteredLogs.map(log => this.renderLog(log)).join('');
        }
        
        // 更新统计
        this.container.querySelector('#stat-total').textContent = this.logs.length;
        this.container.querySelector('#stat-shown').textContent = filteredLogs.length;
    }
    
    renderLog(log) {
        const time = new Date(log.timestamp).toLocaleTimeString();
        const levelClass = `log-level-${log.level}`;
        const levelIcon = this.getLevelIcon(log.level);
        
        return `
            <div class="log-entry ${levelClass}">
                <span class="log-time">${time}</span>
                <span class="log-level" title="${log.level}">
                    ${levelIcon}
                </span>
                <span class="log-message">${this.escapeHtml(log.message)}</span>
            </div>
        `;
    }
    
    getLevelIcon(level) {
        const icons = {
            debug: '🐛',
            info: 'ℹ️',
            warn: '⚠️',
            error: '❌'
        };
        return icons[level] || '📝';
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    updateLastUpdateTime() {
        const now = new Date().toLocaleTimeString();
        this.container.querySelector('#last-update').textContent = `上次更新: ${now}`;
    }
    
    scrollToBottom() {
        const logsContainer = this.container.querySelector('.debug-logs');
        logsContainer.scrollTop = logsContainer.scrollHeight;
    }
    
    startAutoRefresh() {
        this.stopAutoRefresh();
        this.refreshTimer = setInterval(() => {
            this.loadLogs();
        }, this.refreshInterval);
    }
    
    stopAutoRefresh() {
        if (this.refreshTimer) {
            clearInterval(this.refreshTimer);
            this.refreshTimer = null;
        }
    }
    
    showSuccess(message) {
        this.showNotification(message, 'success');
    }
    
    showError(message) {
        this.showNotification(message, 'error');
    }
    
    showNotification(message, type) {
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.textContent = message;
        
        this.container.appendChild(notification);
        
        setTimeout(() => {
            notification.classList.add('show');
        }, 10);
        
        setTimeout(() => {
            notification.classList.remove('show');
            setTimeout(() => {
                notification.remove();
            }, 300);
        }, 3000);
    }
    
    close() {
        this.stopAutoRefresh();
        this.container.classList.add('closing');
        setTimeout(() => {
            this.container.remove();
        }, 300);
    }
    
    show() {
        this.container.style.display = 'flex';
        setTimeout(() => {
            this.container.classList.add('show');
        }, 10);
    }
}

// 全局函数，方便调用
window.showDebugConsole = () => {
    new DebugConsoleManager();
};