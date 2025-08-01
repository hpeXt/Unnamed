// 模块化的主文件，支持 Tauri v2

// 导入 Tauri API 封装
import { invoke, isTauri, getVersion, listen, PluginMessageBus } from './tauri-api.js';
// 导入引导管理器
import { onboarding } from '../components/onboarding/onboarding.js';
// 导入插件创建器
import { pluginCreator } from '../components/plugin-creator/plugin-creator.js';
// 导入插件选择器
import { pluginSelector } from '../components/plugin-selector/plugin-selector.js';
// 导入调试控制台
import { DebugConsoleManager } from '../components/debug-console/debug-console.js';

console.log('Minimal Kernel Dashboard - Loading (Module)...');

// 等待应用就绪
async function waitForAppReady() {
    console.log('Waiting for app to be ready...');
    let retries = 0;
    const maxRetries = 30; // 最多等待 15 秒
    
    while (retries < maxRetries) {
        try {
            const ready = await invoke('is_app_ready');
            if (ready) {
                console.log('App is ready!');
                return true;
            }
        } catch (error) {
            console.error('Error checking app ready state:', error);
        }
        
        await new Promise(resolve => setTimeout(resolve, 500));
        retries++;
    }
    
    console.warn('App ready check timed out, proceeding anyway');
    return false;
}


// 设置基本事件监听
function setupBasicListeners() {
    const addButton = document.getElementById('add-widget');
    if (addButton) {
        addButton.addEventListener('click', async () => {
            console.log('Add widget clicked');
            if (isTauri()) {
                try {
                    // 检查应用是否就绪
                    const ready = await invoke('is_app_ready');
                    if (!ready) {
                        alert('应用还在初始化中，请稍后再试');
                        return;
                    }
                    
                    // 显示优雅的选择界面
                    const choice = await pluginSelector.show();
                    if (!choice) return;
                    
                    // 如果选择创建新插件
                    if (choice.type === 'create') {
                        pluginCreator.show();
                        return;
                    }
                    
                    if (choice.type === 'webview') {
                        // 创建独立窗口插件
                        console.log('=== Creating plugin container ===');
                        const containerId = await invoke('create_plugin_container', {
                            plugin_id: choice.pluginId,
                            render_mode: 'webview',
                            position: { x: 100, y: 100 },
                            size: { width: 600, height: 400 }
                        });
                        console.log('Container created successfully:', containerId);
                    } else {
                        // 创建内联组件
                        console.log('=== Creating inline widget ===');
                        const widgetId = await invoke('create_inline_widget', {
                            widget_type: choice.widgetType,
                            position: { row: 1, col: 1 },
                            size: { row_span: 1, col_span: 1 },
                            config: choice.config || {}
                        });
                        console.log('Inline widget created successfully:', widgetId);
                        
                        // 刷新内联组件显示
                        await updateInlineWidgets();
                    }
                    
                    // 更新容器列表
                    await updateContainerList();
                } catch (error) {
                    console.error('=== Creation failed ===');
                    console.error('Error details:', error);
                    const errorMessage = error.message || error.toString() || '未知错误';
                    alert('创建失败:\n\n' + errorMessage);
                }
            } else {
                alert('在 Web 模式下运行，无法创建插件容器');
            }
        });
    }
    
    const layoutButton = document.getElementById('layout-settings');
    if (layoutButton) {
        layoutButton.addEventListener('click', async () => {
            console.log('Layout settings clicked');
            if (isTauri()) {
                await showLayoutMenu();
            }
        });
    }
    
    const debugButton = document.getElementById('debug-console');
    if (debugButton) {
        debugButton.addEventListener('click', () => {
            console.log('Debug console clicked');
            new DebugConsoleManager();
        });
    }
}

// 更新容器列表
async function updateContainerList() {
    if (isTauri()) {
        try {
            const containers = await invoke('list_containers');
            console.log('Containers:', containers);
            document.getElementById('plugin-count').textContent = `容器: ${containers.length}`;
        } catch (error) {
            console.error('Failed to update container list:', error);
        }
    }
}

// 更新时间
function updateTime() {
    const now = new Date();
    const timeStr = now.toLocaleTimeString('zh-CN', {
        hour12: false,
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
    });
    const timeElement = document.getElementById('system-time');
    if (timeElement) {
        timeElement.textContent = timeStr;
    }
}

// 设置内核消息监听器（调试用）
async function setupKernelMessageListener() {
    if (isTauri()) {
        try {
            const unlisten = await listen('kernel-message', (event) => {
                console.log('[Kernel Message]', event.payload);
            });
            
            // 保存 unlisten 函数以便后续清理
            window.__kernelMessageUnlisten = unlisten;
        } catch (error) {
            console.error('Failed to setup kernel message listener:', error);
        }
    }
}

// 示例：发送测试消息到插件
window.testPluginCommunication = async function() {
    if (isTauri()) {
        try {
            // 发送消息到系统监控插件
            await invoke('send_to_plugin', {
                pluginId: 'ui-system-monitor',
                message: {
                    type: 'test',
                    data: 'Hello from main app!'
                }
            });
            console.log('Test message sent to ui-system-monitor');
        } catch (error) {
            console.error('Failed to send test message:', error);
        }
    }
};

// 开发工具：重置引导流程（用于测试）
window.resetOnboarding = function() {
    if (confirm('确定要重置引导流程吗？这将清除所有用户偏好设置。')) {
        localStorage.removeItem('onboarding_completed');
        localStorage.removeItem('user_preferences');
        localStorage.removeItem('user_theme');
        location.reload();
    }
};

// 开发工具：显示当前用户偏好设置
window.showPreferences = function() {
    const prefs = localStorage.getItem('user_preferences');
    if (prefs) {
        console.log('用户偏好设置:', JSON.parse(prefs));
    } else {
        console.log('尚未设置用户偏好');
    }
};

// 开发工具：显示插件调试控制台
window.showDebugConsole = function() {
    new DebugConsoleManager();
};


// 更新内联组件显示
async function updateInlineWidgets() {
    if (!isTauri()) return;
    
    try {
        const widgets = await invoke('list_inline_widgets');
        console.log('Inline widgets:', widgets);
        
        const gridContainer = document.getElementById('dashboard-grid');
        if (!gridContainer) return;
        
        // 清除欢迎信息
        const welcomeDiv = gridContainer.querySelector('div[style*="padding: 40px"]');
        if (welcomeDiv) {
            welcomeDiv.remove();
        }
        
        // 渲染每个内联组件
        for (const widget of widgets) {
            renderInlineWidget(widget);
        }
    } catch (error) {
        console.error('Failed to update inline widgets:', error);
    }
}

// 渲染单个内联组件
function renderInlineWidget(widget) {
    const gridContainer = document.getElementById('dashboard-grid');
    
    // 检查是否已存在
    let widgetDiv = document.getElementById(`widget-${widget.id}`);
    if (!widgetDiv) {
        widgetDiv = document.createElement('div');
        widgetDiv.id = `widget-${widget.id}`;
        widgetDiv.className = 'inline-widget';
        gridContainer.appendChild(widgetDiv);
    }
    
    // 设置网格位置
    widgetDiv.setAttribute('data-grid-col', widget.position.col);
    widgetDiv.setAttribute('data-grid-row', widget.position.row);
    widgetDiv.setAttribute('data-col-span', widget.size.col_span);
    widgetDiv.setAttribute('data-row-span', widget.size.row_span);
    
    // 设置内容
    widgetDiv.innerHTML = `
        <div class="inline-widget-header">
            <span class="inline-widget-title">${getWidgetTitle(widget.widget_type)}</span>
            <div class="inline-widget-actions">
                <button onclick="removeInlineWidget('${widget.id}')" title="关闭">×</button>
            </div>
        </div>
        <div class="inline-widget-content" id="widget-content-${widget.id}">
            ${renderWidgetContent(widget)}
        </div>
    `;
}

// 获取组件标题
function getWidgetTitle(widgetType) {
    const titles = {
        'heart-rate': '心率监控',
        'cpu-usage': 'CPU 使用率',
        'memory-usage': '内存使用率'
    };
    return titles[widgetType] || widgetType;
}

// 渲染组件内容
function renderWidgetContent(widget) {
    // 根据组件类型渲染不同的内容
    switch (widget.widget_type) {
        case 'heart-rate':
            return `
                <div style="text-align: center;">
                    <div style="font-size: 48px; color: var(--accent-danger);">❤️</div>
                    <div style="font-size: 36px; margin: 16px 0;">75</div>
                    <div style="color: var(--text-secondary);">BPM</div>
                </div>
            `;
        case 'cpu-usage':
            return `
                <div style="text-align: center;">
                    <div style="font-size: 36px; color: var(--accent-primary);">25%</div>
                    <div style="color: var(--text-secondary);">CPU 使用率</div>
                </div>
            `;
        case 'memory-usage':
            return `
                <div style="text-align: center;">
                    <div style="font-size: 36px; color: var(--accent-success);">4.2 GB</div>
                    <div style="color: var(--text-secondary);">内存使用</div>
                </div>
            `;
        default:
            return '<div>组件加载中...</div>';
    }
}

// 删除内联组件
window.removeInlineWidget = async function(widgetId) {
    if (!isTauri()) return;
    
    try {
        await invoke('remove_inline_widget', { widget_id: widgetId });
        const widgetDiv = document.getElementById(`widget-${widgetId}`);
        if (widgetDiv) {
            widgetDiv.remove();
        }
    } catch (error) {
        console.error('Failed to remove widget:', error);
        alert('删除组件失败：' + error.message);
    }
};

// 监听内联组件事件
async function setupInlineWidgetListeners() {
    if (!isTauri()) return;
    
    try {
        // 监听创建事件
        await listen('create-inline-widget', (event) => {
            console.log('Widget created:', event.payload);
            renderInlineWidget(event.payload);
        });
        
        // 监听更新事件
        await listen('update-inline-widget', (event) => {
            const { id, config } = event.payload;
            console.log('Widget updated:', id, config);
            // 更新组件内容
            const contentDiv = document.getElementById(`widget-content-${id}`);
            if (contentDiv) {
                // 这里可以根据配置更新内容
            }
        });
        
        // 监听删除事件
        await listen('remove-inline-widget', (event) => {
            const widgetId = event.payload;
            const widgetDiv = document.getElementById(`widget-${widgetId}`);
            if (widgetDiv) {
                widgetDiv.remove();
            }
        });
    } catch (error) {
        console.error('Failed to setup inline widget listeners:', error);
    }
}

// 在初始化时设置内联组件监听器
async function initApp() {
    console.log('=== Initializing application ===');
    console.log('Tauri available:', isTauri());
    console.log('Window.__TAURI__:', window.__TAURI__);
    
    if (isTauri() && window.__TAURI__) {
        console.log('Tauri API details:', {
            core: !!window.__TAURI__.core,
            invoke: !!window.__TAURI__.core?.invoke,
            event: !!window.__TAURI__.event,
            listen: !!window.__TAURI__.event?.listen
        });
    }
    
    // 检查并启动引导流程
    const onboardingShown = await onboarding.init();
    if (onboardingShown) {
        console.log('Showing onboarding flow for first-time user');
        // 监听引导完成事件
        window.addEventListener('onboarding-completed', (event) => {
            console.log('Onboarding completed with preferences:', event.detail);
            // 引导完成后继续初始化
            continueAppInit();
        });
        return; // 等待引导完成
    }
    
    // 如果不需要引导，直接继续初始化
    await continueAppInit();
}

// 继续应用初始化（引导完成后或跳过引导时调用）
async function continueAppInit() {
    console.log('=== Continuing app initialization ===');
    
    // 应用保存的主题偏好
    const savedTheme = localStorage.getItem('user_theme');
    if (savedTheme) {
        document.documentElement.setAttribute('data-theme', savedTheme);
        console.log('Applied saved theme:', savedTheme);
    }
    
    // 更新时间
    updateTime();
    setInterval(updateTime, 1000);
    
    // 如果在 Tauri 环境中，获取应用信息
    if (isTauri()) {
        try {
            // 等待应用就绪
            await waitForAppReady();
            // 获取应用版本
            const version = await getVersion();
            console.log('App version:', version);
            
            // 尝试获取 UI 插件列表
            const plugins = await invoke('get_plugins');
            console.log('Available UI plugins:', plugins);
            document.getElementById('plugin-count').textContent = `插件: ${plugins.length}`;
            
            // 设置内联组件监听器
            await setupInlineWidgetListeners();
            
            // 加载现有的内联组件
            await updateInlineWidgets();
        } catch (error) {
            console.error('Failed to initialize Tauri features:', error);
            document.getElementById('plugin-count').textContent = '插件: 错误';
        }
    } else {
        console.warn('Running in web mode - Tauri features disabled');
        document.getElementById('plugin-count').textContent = '插件: Web模式';
    }
    
    document.getElementById('memory-usage').textContent = '内存: --';
    
    // 设置基本事件监听
    setupBasicListeners();
    
    // 设置内核消息监听（用于调试）
    setupKernelMessageListener();
    
    console.log('Application initialized successfully');
}

// 显示布局菜单
async function showLayoutMenu() {
    const choice = prompt(
        '布局管理：\n\n' +
        '1. 保存当前布局\n' +
        '2. 加载已保存的布局\n' +
        '3. 查看所有布局\n\n' +
        '请输入数字（1-3）：'
    );
    
    try {
        switch (choice) {
            case '1':
                await saveCurrentLayout();
                break;
            case '2':
                await loadSavedLayout();
                break;
            case '3':
                await showAllLayouts();
                break;
            default:
                break;
        }
    } catch (error) {
        console.error('Layout operation failed:', error);
        alert('操作失败：' + error.message);
    }
}

// 保存当前布局
async function saveCurrentLayout() {
    const name = prompt('请输入布局名称：');
    if (!name) return;
    
    try {
        const layoutId = await invoke('save_layout', { name });
        alert(`布局已保存，ID: ${layoutId}`);
    } catch (error) {
        console.error('Failed to save layout:', error);
        throw error;
    }
}

// 加载已保存的布局
async function loadSavedLayout() {
    try {
        const layouts = await invoke('list_layouts');
        if (layouts.length === 0) {
            alert('没有已保存的布局');
            return;
        }
        
        const layoutList = layouts.map((l, i) => 
            `${i + 1}. ${l.name} (${new Date(l.created_at).toLocaleDateString()})`
        ).join('\n');
        
        const choice = prompt(
            '选择要加载的布局：\n\n' + layoutList + '\n\n请输入数字：'
        );
        
        if (choice && !isNaN(choice)) {
            const index = parseInt(choice) - 1;
            if (index >= 0 && index < layouts.length) {
                await invoke('apply_layout', { layout_id: layouts[index].id });
                alert('布局已应用');
                // 刷新内联组件显示
                await updateInlineWidgets();
            }
        }
    } catch (error) {
        console.error('Failed to load layout:', error);
        throw error;
    }
}

// 显示所有布局
async function showAllLayouts() {
    try {
        const layouts = await invoke('list_layouts');
        if (layouts.length === 0) {
            alert('没有已保存的布局');
            return;
        }
        
        const layoutInfo = layouts.map(l => 
            `名称: ${l.name}\n` +
            `描述: ${l.description || '无'}\n` +
            `创建时间: ${new Date(l.created_at).toLocaleString()}\n` +
            `网格: ${l.grid_columns}x${l.grid_rows}\n` +
            `默认: ${l.is_default ? '是' : '否'}`
        ).join('\n\n---\n\n');
        
        alert('所有布局：\n\n' + layoutInfo);
    } catch (error) {
        console.error('Failed to list layouts:', error);
        throw error;
    }
}

// 页面加载完成后初始化
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initApp);
} else {
    initApp();
}