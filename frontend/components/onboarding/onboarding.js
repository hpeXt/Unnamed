// 引导流程管理器
export class OnboardingManager {
    constructor() {
        this.currentStep = 0;
        this.totalSteps = 3;
        this.preferences = {
            theme: 'light',
            language: 'zh-CN',
            enableAnalytics: false
        };
        
        // 检查是否已完成引导
        this.isCompleted = localStorage.getItem('onboarding_completed') === 'true';
    }
    
    // 初始化引导界面
    async init() {
        // 检查是否已完成引导
        if (this.isCompleted) {
            return false;
        }
        
        // 开发模式下跳过引导
        const urlParams = new URLSearchParams(window.location.search);
        if (urlParams.get('skipOnboarding') === 'true') {
            console.log('Skipping onboarding in development mode');
            this.complete();
            return false;
        }
        
        // 加载样式
        await this.loadStyles();
        
        // 创建引导容器
        this.createContainer();
        
        // 显示第一步
        this.showStep(0);
        
        return true;
    }
    
    // 加载样式文件
    async loadStyles() {
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = './onboarding/onboarding.css';
        document.head.appendChild(link);
        
        // 等待样式加载完成
        return new Promise((resolve) => {
            link.onload = resolve;
        });
    }
    
    // 创建引导容器
    createContainer() {
        const overlay = document.createElement('div');
        overlay.className = 'onboarding-overlay';
        overlay.id = 'onboarding-overlay';
        
        const container = document.createElement('div');
        container.className = 'onboarding-container';
        
        container.innerHTML = `
            <div class="onboarding-header">
                <div class="onboarding-progress">
                    ${Array.from({length: this.totalSteps}, (_, i) => 
                        `<div class="progress-dot ${i === 0 ? 'active' : ''}" data-step="${i}"></div>`
                    ).join('')}
                </div>
                <button class="btn btn-text" id="skip-btn">跳过</button>
            </div>
            
            <div class="onboarding-content" id="onboarding-content">
                <!-- 内容将动态插入 -->
            </div>
            
            <div class="onboarding-footer">
                <button class="btn btn-secondary" id="prev-btn" style="visibility: hidden;">上一步</button>
                <button class="btn btn-primary" id="next-btn">下一步</button>
            </div>
        `;
        
        overlay.appendChild(container);
        document.body.appendChild(overlay);
        
        // 绑定事件
        this.bindEvents();
    }
    
    // 绑定事件处理
    bindEvents() {
        document.getElementById('skip-btn').addEventListener('click', () => this.skip());
        document.getElementById('prev-btn').addEventListener('click', () => this.previousStep());
        document.getElementById('next-btn').addEventListener('click', () => this.nextStep());
    }
    
    // 显示指定步骤
    showStep(step) {
        this.currentStep = step;
        const content = document.getElementById('onboarding-content');
        
        // 更新进度指示器
        document.querySelectorAll('.progress-dot').forEach((dot, index) => {
            dot.classList.toggle('active', index === step);
        });
        
        // 更新按钮状态
        const prevBtn = document.getElementById('prev-btn');
        const nextBtn = document.getElementById('next-btn');
        
        prevBtn.style.visibility = step === 0 ? 'hidden' : 'visible';
        nextBtn.textContent = step === this.totalSteps - 1 ? '开始使用' : '下一步';
        
        // 加载步骤内容
        content.innerHTML = '';
        content.appendChild(this.getStepContent(step));
        
        // 添加动画效果
        content.querySelector('.step-content').classList.add('slide-enter');
    }
    
    // 获取步骤内容
    getStepContent(step) {
        const container = document.createElement('div');
        container.className = 'step-content';
        
        switch(step) {
            case 0:
                container.innerHTML = this.getWelcomeContent();
                break;
            case 1:
                container.innerHTML = this.getFeaturesContent();
                break;
            case 2:
                container.innerHTML = this.getSetupContent();
                // 绑定设置表单事件
                setTimeout(() => this.bindSetupEvents(), 0);
                break;
        }
        
        return container;
    }
    
    // 欢迎页内容
    getWelcomeContent() {
        return `
            <div class="welcome-icon">🚀</div>
            <h1 class="welcome-title">欢迎使用 Minimal Kernel</h1>
            <p class="welcome-subtitle">
                一个基于插件架构的本地数据处理平台<br>
                让您完全掌控自己的健康数据
            </p>
        `;
    }
    
    // 功能介绍内容
    getFeaturesContent() {
        const features = [
            {
                icon: '🔌',
                title: '插件系统',
                description: '通过 WebAssembly 插件无限扩展功能'
            },
            {
                icon: '🔒',
                title: '隐私优先',
                description: '所有数据本地存储，您拥有完全控制权'
            },
            {
                icon: '📊',
                title: '数据可视化',
                description: '直观的图表和仪表盘展示您的健康数据'
            },
            {
                icon: '🤖',
                title: '智能分析',
                description: 'AI 驱动的健康趋势分析和个性化建议'
            }
        ];
        
        return `
            <h2 style="margin-bottom: 32px;">核心功能</h2>
            <div class="features-grid">
                ${features.map(feature => `
                    <div class="feature-card">
                        <div class="feature-icon">${feature.icon}</div>
                        <h3 class="feature-title">${feature.title}</h3>
                        <p class="feature-description">${feature.description}</p>
                    </div>
                `).join('')}
            </div>
        `;
    }
    
    // 快速设置内容
    getSetupContent() {
        return `
            <h2 style="margin-bottom: 32px;">快速设置</h2>
            <form class="setup-form">
                <div class="form-group">
                    <label class="form-label">选择主题</label>
                    <div class="theme-selector">
                        <div class="theme-option ${this.preferences.theme === 'light' ? 'selected' : ''}" data-theme="light">
                            <div class="theme-preview light"></div>
                            <span>浅色</span>
                        </div>
                        <div class="theme-option ${this.preferences.theme === 'dark' ? 'selected' : ''}" data-theme="dark">
                            <div class="theme-preview dark"></div>
                            <span>深色</span>
                        </div>
                    </div>
                </div>
                
                <div class="form-group">
                    <label class="form-label">语言</label>
                    <select class="form-control" id="language-select">
                        <option value="zh-CN" ${this.preferences.language === 'zh-CN' ? 'selected' : ''}>简体中文</option>
                        <option value="en-US" ${this.preferences.language === 'en-US' ? 'selected' : ''}>English</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <div class="checkbox-group">
                        <input type="checkbox" id="analytics-checkbox" ${this.preferences.enableAnalytics ? 'checked' : ''}>
                        <label for="analytics-checkbox">帮助改进产品（匿名收集使用数据）</label>
                    </div>
                </div>
            </form>
        `;
    }
    
    // 绑定设置表单事件
    bindSetupEvents() {
        // 主题选择
        document.querySelectorAll('.theme-option').forEach(option => {
            option.addEventListener('click', (e) => {
                document.querySelectorAll('.theme-option').forEach(opt => opt.classList.remove('selected'));
                option.classList.add('selected');
                this.preferences.theme = option.dataset.theme;
                
                // 立即应用主题
                this.applyTheme(this.preferences.theme);
            });
        });
        
        // 语言选择
        document.getElementById('language-select').addEventListener('change', (e) => {
            this.preferences.language = e.target.value;
        });
        
        // 分析选项
        document.getElementById('analytics-checkbox').addEventListener('change', (e) => {
            this.preferences.enableAnalytics = e.target.checked;
        });
    }
    
    // 应用主题
    applyTheme(theme) {
        document.documentElement.setAttribute('data-theme', theme);
        localStorage.setItem('user_theme', theme);
    }
    
    // 下一步
    nextStep() {
        if (this.currentStep < this.totalSteps - 1) {
            this.showStep(this.currentStep + 1);
        } else {
            this.complete();
        }
    }
    
    // 上一步
    previousStep() {
        if (this.currentStep > 0) {
            this.showStep(this.currentStep - 1);
        }
    }
    
    // 跳过引导
    skip() {
        if (confirm('确定要跳过引导吗？您可以在设置中重新查看。')) {
            this.complete();
        }
    }
    
    // 完成引导
    complete() {
        // 保存偏好设置
        localStorage.setItem('user_preferences', JSON.stringify(this.preferences));
        localStorage.setItem('onboarding_completed', 'true');
        
        // 应用设置
        this.applyTheme(this.preferences.theme);
        
        // 移除引导界面
        const overlay = document.getElementById('onboarding-overlay');
        overlay.style.opacity = '0';
        setTimeout(() => {
            overlay.remove();
            
            // 触发完成事件
            window.dispatchEvent(new CustomEvent('onboarding-completed', {
                detail: this.preferences
            }));
        }, 300);
    }
    
    // 重置引导（用于测试或在设置中重新查看）
    static reset() {
        localStorage.removeItem('onboarding_completed');
        localStorage.removeItem('user_preferences');
        location.reload();
    }
}

// 导出单例
export const onboarding = new OnboardingManager();