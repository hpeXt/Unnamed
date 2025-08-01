// 插件创建向导
export class PluginCreator {
    constructor() {
        this.currentStep = 0;
        this.pluginConfig = {
            name: '',
            displayName: '',
            description: '',
            author: '',
            version: '0.1.0',
            type: 'data-collector', // data-collector, analyzer, ui-widget
            features: [],
            icon: '📊'
        };
        
        // 加载样式
        this.loadStyles();
    }
    
    // 加载样式文件
    loadStyles() {
        // 检查是否已经加载
        if (document.querySelector('link[href*="plugin-creator.css"]')) {
            return;
        }
        
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = './plugin-creator/plugin-creator.css';
        document.head.appendChild(link);
    }
    
    // 显示创建向导
    async show() {
        this.createModal();
        this.showStep(0);
    }
    
    // 创建模态框
    createModal() {
        const modal = document.createElement('div');
        modal.className = 'plugin-creator-modal';
        modal.innerHTML = `
            <div class="plugin-creator-container">
                <div class="plugin-creator-header">
                    <h2>创建新插件</h2>
                    <button class="close-btn" id="close-creator">×</button>
                </div>
                
                <div class="plugin-creator-progress">
                    <div class="progress-step active" data-step="0">基本信息</div>
                    <div class="progress-step" data-step="1">插件类型</div>
                    <div class="progress-step" data-step="2">功能选择</div>
                    <div class="progress-step" data-step="3">确认创建</div>
                </div>
                
                <div class="plugin-creator-content" id="creator-content">
                    <!-- 动态内容 -->
                </div>
                
                <div class="plugin-creator-footer">
                    <button class="btn btn-secondary" id="prev-step" style="visibility: hidden;">上一步</button>
                    <button class="btn btn-primary" id="next-step">下一步</button>
                </div>
            </div>
        `;
        
        document.body.appendChild(modal);
        this.modal = modal;
        this.bindEvents();
    }
    
    // 绑定事件
    bindEvents() {
        document.getElementById('close-creator').addEventListener('click', () => this.close());
        document.getElementById('prev-step').addEventListener('click', () => this.previousStep());
        document.getElementById('next-step').addEventListener('click', () => this.nextStep());
        
        // 点击背景关闭
        this.modal.addEventListener('click', (e) => {
            if (e.target === this.modal) {
                this.close();
            }
        });
    }
    
    // 显示步骤内容
    showStep(step) {
        this.currentStep = step;
        const content = document.getElementById('creator-content');
        
        // 更新进度指示
        document.querySelectorAll('.progress-step').forEach((el, index) => {
            el.classList.toggle('active', index <= step);
        });
        
        // 更新按钮
        document.getElementById('prev-step').style.visibility = step === 0 ? 'hidden' : 'visible';
        document.getElementById('next-step').textContent = step === 3 ? '创建插件' : '下一步';
        
        // 加载步骤内容
        switch(step) {
            case 0:
                content.innerHTML = this.getBasicInfoStep();
                this.bindBasicInfoEvents();
                break;
            case 1:
                content.innerHTML = this.getPluginTypeStep();
                this.bindPluginTypeEvents();
                break;
            case 2:
                content.innerHTML = this.getFeaturesStep();
                this.bindFeaturesEvents();
                break;
            case 3:
                content.innerHTML = this.getConfirmStep();
                break;
        }
    }
    
    // 基本信息步骤
    getBasicInfoStep() {
        return `
            <div class="form-section">
                <h3>插件基本信息</h3>
                
                <div class="form-group">
                    <label>插件名称 (英文，用于代码)</label>
                    <input type="text" id="plugin-name" class="form-control" 
                           placeholder="my-health-tracker" value="${this.pluginConfig.name}"
                           pattern="[a-z0-9-]+" required>
                    <small>只能包含小写字母、数字和连字符</small>
                </div>
                
                <div class="form-group">
                    <label>显示名称</label>
                    <input type="text" id="plugin-display-name" class="form-control" 
                           placeholder="我的健康追踪器" value="${this.pluginConfig.displayName}" required>
                </div>
                
                <div class="form-group">
                    <label>描述</label>
                    <textarea id="plugin-description" class="form-control" rows="3"
                              placeholder="追踪和分析个人健康数据...">${this.pluginConfig.description}</textarea>
                </div>
                
                <div class="form-group">
                    <label>作者</label>
                    <input type="text" id="plugin-author" class="form-control" 
                           value="${this.pluginConfig.author || 'hpeXt'}" required>
                </div>
                
                <div class="form-group">
                    <label>图标</label>
                    <div class="icon-selector">
                        ${this.getIconOptions()}
                    </div>
                </div>
            </div>
        `;
    }
    
    // 图标选项
    getIconOptions() {
        const icons = ['📊', '💊', '🏃', '❤️', '🧠', '🍎', '💤', '🏥', '📈', '🔬'];
        return icons.map(icon => `
            <button class="icon-option ${this.pluginConfig.icon === icon ? 'selected' : ''}" 
                    data-icon="${icon}">${icon}</button>
        `).join('');
    }
    
    // 插件类型步骤
    getPluginTypeStep() {
        const types = [
            {
                id: 'data-collector',
                name: '数据采集器',
                icon: '📥',
                description: '定期采集和存储健康数据'
            },
            {
                id: 'analyzer',
                name: '数据分析器',
                icon: '📊',
                description: '分析已有数据，生成洞察报告'
            },
            {
                id: 'ui-widget',
                name: 'UI 组件',
                icon: '🎨',
                description: '创建可视化仪表盘组件'
            }
        ];
        
        return `
            <div class="form-section">
                <h3>选择插件类型</h3>
                <div class="plugin-type-grid">
                    ${types.map(type => `
                        <div class="plugin-type-card ${this.pluginConfig.type === type.id ? 'selected' : ''}" 
                             data-type="${type.id}">
                            <div class="type-icon">${type.icon}</div>
                            <div class="type-name">${type.name}</div>
                            <div class="type-description">${type.description}</div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    }
    
    // 功能选择步骤
    getFeaturesStep() {
        const featuresByType = {
            'data-collector': [
                { id: 'storage', name: '数据存储', checked: true },
                { id: 'schedule', name: '定时任务', checked: true },
                { id: 'api-fetch', name: 'API 数据获取', checked: false },
                { id: 'file-import', name: '文件导入', checked: false }
            ],
            'analyzer': [
                { id: 'storage-read', name: '读取存储数据', checked: true },
                { id: 'statistics', name: '统计分析', checked: true },
                { id: 'ml-analysis', name: '机器学习分析', checked: false },
                { id: 'report-gen', name: '报告生成', checked: false }
            ],
            'ui-widget': [
                { id: 'chart', name: '图表显示', checked: true },
                { id: 'realtime', name: '实时更新', checked: false },
                { id: 'interactive', name: '交互功能', checked: false },
                { id: 'export', name: '数据导出', checked: false }
            ]
        };
        
        const features = featuresByType[this.pluginConfig.type] || [];
        
        return `
            <div class="form-section">
                <h3>选择插件功能</h3>
                <div class="features-list">
                    ${features.map(feature => `
                        <label class="feature-item">
                            <input type="checkbox" id="feature-${feature.id}" 
                                   ${feature.checked ? 'checked' : ''}
                                   ${feature.checked ? 'disabled' : ''}>
                            <span>${feature.name}</span>
                            ${feature.checked ? '<small>(必需)</small>' : ''}
                        </label>
                    `).join('')}
                </div>
                
                <div class="code-preview">
                    <h4>代码预览</h4>
                    <pre><code>${this.generateCodePreview()}</code></pre>
                </div>
            </div>
        `;
    }
    
    // 确认步骤
    getConfirmStep() {
        return `
            <div class="form-section">
                <h3>确认创建</h3>
                
                <div class="summary-card">
                    <div class="summary-icon">${this.pluginConfig.icon}</div>
                    <h4>${this.pluginConfig.displayName}</h4>
                    <p>${this.pluginConfig.description}</p>
                    
                    <div class="summary-details">
                        <div><strong>名称:</strong> ${this.pluginConfig.name}</div>
                        <div><strong>类型:</strong> ${this.getTypeName(this.pluginConfig.type)}</div>
                        <div><strong>作者:</strong> ${this.pluginConfig.author}</div>
                        <div><strong>版本:</strong> ${this.pluginConfig.version}</div>
                        <div><strong>功能:</strong> ${this.pluginConfig.features.join(', ') || '基础功能'}</div>
                    </div>
                </div>
                
                <div class="confirm-actions">
                    <p>插件将创建在: <code>plugins/${this.pluginConfig.name}/</code></p>
                    <p class="hint">创建后可以立即在插件列表中看到并使用。</p>
                </div>
            </div>
        `;
    }
    
    // 获取类型名称
    getTypeName(type) {
        const names = {
            'data-collector': '数据采集器',
            'analyzer': '数据分析器',
            'ui-widget': 'UI 组件'
        };
        return names[type] || type;
    }
    
    // 生成代码预览
    generateCodePreview() {
        return `use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[plugin_fn]
pub fn init() -> FnResult<String> {
    info!("${this.pluginConfig.displayName} 初始化成功");
    Ok("Plugin initialized".to_string())
}

#[plugin_fn]
pub fn process() -> FnResult<String> {
    // 您的插件逻辑
    Ok("Processing...".to_string())
}`;
    }
    
    // 绑定基本信息事件
    bindBasicInfoEvents() {
        document.getElementById('plugin-name').addEventListener('input', (e) => {
            this.pluginConfig.name = e.target.value;
        });
        
        document.getElementById('plugin-display-name').addEventListener('input', (e) => {
            this.pluginConfig.displayName = e.target.value;
        });
        
        document.getElementById('plugin-description').addEventListener('input', (e) => {
            this.pluginConfig.description = e.target.value;
        });
        
        document.getElementById('plugin-author').addEventListener('input', (e) => {
            this.pluginConfig.author = e.target.value;
        });
        
        // 图标选择
        document.querySelectorAll('.icon-option').forEach(btn => {
            btn.addEventListener('click', (e) => {
                document.querySelectorAll('.icon-option').forEach(b => b.classList.remove('selected'));
                btn.classList.add('selected');
                this.pluginConfig.icon = btn.dataset.icon;
            });
        });
    }
    
    // 绑定插件类型事件
    bindPluginTypeEvents() {
        document.querySelectorAll('.plugin-type-card').forEach(card => {
            card.addEventListener('click', () => {
                document.querySelectorAll('.plugin-type-card').forEach(c => c.classList.remove('selected'));
                card.classList.add('selected');
                this.pluginConfig.type = card.dataset.type;
            });
        });
    }
    
    // 绑定功能选择事件
    bindFeaturesEvents() {
        const checkboxes = document.querySelectorAll('input[type="checkbox"]:not(:disabled)');
        checkboxes.forEach(checkbox => {
            checkbox.addEventListener('change', () => {
                this.updateSelectedFeatures();
                // 更新代码预览
                document.querySelector('.code-preview code').textContent = this.generateCodePreview();
            });
        });
    }
    
    // 更新选中的功能
    updateSelectedFeatures() {
        const features = [];
        document.querySelectorAll('input[type="checkbox"]:checked').forEach(cb => {
            const featureId = cb.id.replace('feature-', '');
            features.push(featureId);
        });
        this.pluginConfig.features = features;
    }
    
    // 验证当前步骤
    validateStep() {
        switch(this.currentStep) {
            case 0:
                if (!this.pluginConfig.name || !this.pluginConfig.displayName) {
                    alert('请填写必填项');
                    return false;
                }
                if (!/^[a-z0-9-]+$/.test(this.pluginConfig.name)) {
                    alert('插件名称只能包含小写字母、数字和连字符');
                    return false;
                }
                return true;
            default:
                return true;
        }
    }
    
    // 下一步
    nextStep() {
        if (this.currentStep < 3) {
            if (!this.validateStep()) return;
            this.showStep(this.currentStep + 1);
        } else {
            this.createPlugin();
        }
    }
    
    // 上一步
    previousStep() {
        if (this.currentStep > 0) {
            this.showStep(this.currentStep - 1);
        }
    }
    
    // 创建插件
    async createPlugin() {
        try {
            // 显示加载状态
            document.getElementById('next-step').disabled = true;
            document.getElementById('next-step').textContent = '创建中...';
            
            // 调用后端创建插件
            const result = await window.__TAURI__.core.invoke('create_plugin_from_template', {
                config: this.pluginConfig
            });
            
            console.log('插件创建成功:', result);
            
            // 显示成功消息
            alert(`插件 "${this.pluginConfig.displayName}" 创建成功！\n\n位置: plugins/${this.pluginConfig.name}/`);
            
            // 关闭创建器
            this.close();
            
            // 刷新插件列表
            if (window.updatePluginList) {
                window.updatePluginList();
            }
            
        } catch (error) {
            console.error('创建插件失败:', error);
            alert('创建插件失败: ' + error.message);
            
            // 恢复按钮状态
            document.getElementById('next-step').disabled = false;
            document.getElementById('next-step').textContent = '创建插件';
        }
    }
    
    // 关闭创建器
    close() {
        if (this.modal) {
            this.modal.remove();
            this.modal = null;
        }
    }
}

// 导出单例
export const pluginCreator = new PluginCreator();