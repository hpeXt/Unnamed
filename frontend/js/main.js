// 简单的初始化脚本，用于测试应用是否能正常启动

console.log('Minimal Kernel Dashboard - Loading...');

// Tauri API (使用全局对象)
const { invoke } = window.__TAURI__ || {};

// 初始化应用
async function initApp() {
    console.log('Initializing application...');
    
    // 更新时间
    updateTime();
    setInterval(updateTime, 1000);
    
    // 如果 Tauri API 可用，尝试获取插件列表
    if (invoke) {
        try {
            // 尝试获取 UI 插件列表
            const plugins = await invoke('get_plugins');
            console.log('Available UI plugins:', plugins);
            document.getElementById('plugin-count').textContent = `插件: ${plugins.length}`;
        } catch (error) {
            console.error('Failed to get plugins:', error);
            document.getElementById('plugin-count').textContent = '插件: 错误';
        }
    } else {
        console.warn('Tauri API not available');
        document.getElementById('plugin-count').textContent = '插件: N/A';
    }
    
    document.getElementById('memory-usage').textContent = '内存: --';
    
    // 设置基本事件监听
    setupBasicListeners();
    
    console.log('Application initialized successfully');
}

// 设置基本事件监听
function setupBasicListeners() {
    const addButton = document.getElementById('add-widget');
    if (addButton) {
        addButton.addEventListener('click', async () => {
            console.log('Add widget clicked');
            if (invoke) {
                try {
                    // 尝试重新加载插件
                    console.log('Reloading plugins...');
                    const plugins = await invoke('reload_plugins');
                    console.log('Plugins reloaded:', plugins);
                    document.getElementById('plugin-count').textContent = `插件: ${plugins.length}`;
                    alert(`成功加载 ${plugins.length} 个插件！`);
                } catch (error) {
                    console.error('Failed to reload plugins:', error);
                    alert('加载插件失败: ' + error);
                }
            } else {
                alert('Tauri API 不可用！');
            }
        });
    }
    
    const layoutButton = document.getElementById('layout-settings');
    if (layoutButton) {
        layoutButton.addEventListener('click', () => {
            console.log('Layout settings clicked');
            alert('布局设置功能即将推出！');
        });
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

// 页面加载完成后初始化
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initApp);
} else {
    initApp();
}