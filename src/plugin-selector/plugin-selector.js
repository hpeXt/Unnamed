// 插件选择器 - 更优雅的界面替代 prompt
export class PluginSelector {
    constructor() {
        this.loadStyles();
    }
    
    // 加载样式
    loadStyles() {
        if (document.querySelector('link[href*="plugin-selector.css"]')) {
            return;
        }
        
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = './plugin-selector/plugin-selector.css';
        document.head.appendChild(link);
    }
    
    // 显示选择器
    show() {
        return new Promise((resolve) => {
            this.resolve = resolve;
            this.createModal();
        });
    }
    
    // 创建模态框
    createModal() {
        const modal = document.createElement('div');
        modal.className = 'plugin-selector-modal';
        modal.innerHTML = `
            <div class="plugin-selector-container">
                <div class="plugin-selector-header">
                    <h3>选择要添加的插件</h3>
                    <button class="close-btn" id="close-selector">×</button>
                </div>
                
                <div class="plugin-selector-content">
                    <div class="plugin-option create-new" data-action="create">
                        <div class="option-icon">➕</div>
                        <div class="option-info">
                            <div class="option-name">创建新插件</div>
                            <div class="option-description">使用向导创建自定义插件</div>
                        </div>
                    </div>
                    
                    <div class="plugin-section">
                        <h4>系统插件</h4>
                        <div class="plugin-option" data-type="webview" data-plugin-id="system-monitor">
                            <div class="option-icon">📊</div>
                            <div class="option-info">
                                <div class="option-name">系统监控</div>
                                <div class="option-description">独立窗口显示系统资源使用情况</div>
                            </div>
                            <div class="option-badge">独立窗口</div>
                        </div>
                    </div>
                    
                    <div class="plugin-section">
                        <h4>健康监测</h4>
                        <div class="plugin-option" data-type="inline" data-widget-type="heart-rate">
                            <div class="option-icon">❤️</div>
                            <div class="option-info">
                                <div class="option-name">心率监控</div>
                                <div class="option-description">实时显示心率数据</div>
                            </div>
                            <div class="option-badge badge-inline">内联组件</div>
                        </div>
                    </div>
                    
                    <div class="plugin-section">
                        <h4>性能监控</h4>
                        <div class="plugin-option" data-type="inline" data-widget-type="cpu-usage">
                            <div class="option-icon">🔥</div>
                            <div class="option-info">
                                <div class="option-name">CPU 使用率</div>
                                <div class="option-description">监控处理器使用情况</div>
                            </div>
                            <div class="option-badge badge-inline">内联组件</div>
                        </div>
                        
                        <div class="plugin-option" data-type="inline" data-widget-type="memory-usage">
                            <div class="option-icon">💾</div>
                            <div class="option-info">
                                <div class="option-name">内存使用率</div>
                                <div class="option-description">监控内存占用情况</div>
                            </div>
                            <div class="option-badge badge-inline">内联组件</div>
                        </div>
                    </div>
                </div>
            </div>
        `;
        
        document.body.appendChild(modal);
        this.modal = modal;
        this.bindEvents();
    }
    
    // 绑定事件
    bindEvents() {
        // 关闭按钮
        document.getElementById('close-selector').addEventListener('click', () => {
            this.close();
            this.resolve(null);
        });
        
        // 点击背景关闭
        this.modal.addEventListener('click', (e) => {
            if (e.target === this.modal) {
                this.close();
                this.resolve(null);
            }
        });
        
        // 插件选项点击
        this.modal.querySelectorAll('.plugin-option').forEach(option => {
            option.addEventListener('click', () => {
                const action = option.dataset.action;
                
                if (action === 'create') {
                    // 关闭选择器并打开创建器
                    this.close();
                    this.resolve({ type: 'create' });
                } else {
                    // 返回选中的插件信息
                    const result = {
                        type: option.dataset.type,
                        pluginId: option.dataset.pluginId,
                        widgetType: option.dataset.widgetType,
                        name: option.querySelector('.option-name').textContent
                    };
                    
                    this.close();
                    this.resolve(result);
                }
            });
        });
    }
    
    // 关闭选择器
    close() {
        if (this.modal) {
            this.modal.remove();
            this.modal = null;
        }
    }
}

// 导出单例
export const pluginSelector = new PluginSelector();