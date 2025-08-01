// 系统监控插件 - 使用 Tauri API
class SystemMonitorPlugin {
    constructor() {
        this.cpuHistory = [];
        this.memoryHistory = [];
        this.maxDataPoints = 50;
        this.unlisten = null;
    }

    async initialize() {
        console.log('System Monitor Plugin initializing with Tauri API...');
        
        // 订阅系统数据 - 使用 pluginAPI
        if (window.pluginAPI) {
            try {
                // 订阅系统统计主题
                this.unlisten = await window.pluginAPI.subscribe('system.stats', (message) => {
                    console.log('Received system stats:', message);
                    this.handleMessage(message);
                });
                
                console.log('Successfully subscribed to system.stats');
            } catch (error) {
                console.error('Failed to subscribe:', error);
            }
        }
        
        // 暂时保留模拟数据（等真实插件就绪后移除）
        this.startMockData();
        
        // 初始化图表
        this.initChart();
    }
    
    destroy() {
        // 清理监听器
        if (this.unlisten) {
            this.unlisten();
            this.unlisten = null;
        }
    }

    handleMessage(message) {
        // 处理来自内核的消息
        if (message.topic === 'system.stats' && message.payload) {
            this.updateStats(message.payload);
        }
    }

    updateStats(data) {
        // 更新 CPU 使用率
        if (data.cpu !== undefined) {
            document.getElementById('cpu-usage').textContent = `${data.cpu.toFixed(1)}%`;
            this.cpuHistory.push({
                time: new Date(),
                value: data.cpu
            });
            if (this.cpuHistory.length > this.maxDataPoints) {
                this.cpuHistory.shift();
            }
        }
        
        // 更新内存使用
        if (data.memory !== undefined) {
            document.getElementById('memory-usage').textContent = `${data.memory.toFixed(1)}%`;
            this.memoryHistory.push({
                time: new Date(),
                value: data.memory
            });
            if (this.memoryHistory.length > this.maxDataPoints) {
                this.memoryHistory.shift();
            }
        }
        
        // 更新插件数量
        if (data.pluginCount !== undefined) {
            document.getElementById('plugin-count').textContent = data.pluginCount;
        }
        
        // 更新运行时间
        if (data.uptime !== undefined) {
            const hours = Math.floor(data.uptime / 3600);
            const minutes = Math.floor((data.uptime % 3600) / 60);
            document.getElementById('uptime').textContent = `${hours}h ${minutes}m`;
        }
        
        // 更新图表
        this.updateChart();
    }

    startMockData() {
        // 开发时的模拟数据
        setInterval(() => {
            const mockData = {
                cpu: 20 + Math.random() * 60,
                memory: 30 + Math.random() * 40,
                pluginCount: 3,
                uptime: Date.now() / 1000
            };
            this.updateStats(mockData);
        }, 1000);
    }

    initChart() {
        const canvas = document.getElementById('cpu-chart');
        const ctx = canvas.getContext('2d');
        
        // 设置画布大小
        const resizeCanvas = () => {
            const container = canvas.parentElement;
            canvas.width = container.clientWidth;
            canvas.height = container.clientHeight;
        };
        resizeCanvas();
        window.addEventListener('resize', resizeCanvas);
    }

    updateChart() {
        const canvas = document.getElementById('cpu-chart');
        const ctx = canvas.getContext('2d');
        const width = canvas.width;
        const height = canvas.height;
        
        // 清空画布
        ctx.fillStyle = '#2A2A2A';
        ctx.fillRect(0, 0, width, height);
        
        // 绘制网格
        ctx.strokeStyle = '#3A3A3A';
        ctx.lineWidth = 1;
        
        // 水平网格线
        for (let i = 0; i <= 4; i++) {
            const y = (height / 4) * i;
            ctx.beginPath();
            ctx.moveTo(0, y);
            ctx.lineTo(width, y);
            ctx.stroke();
        }
        
        // 绘制 CPU 曲线
        if (this.cpuHistory.length > 1) {
            ctx.strokeStyle = '#00D9FF';
            ctx.lineWidth = 2;
            ctx.beginPath();
            
            this.cpuHistory.forEach((point, index) => {
                const x = (width / (this.maxDataPoints - 1)) * index;
                const y = height - (point.value / 100) * height;
                
                if (index === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
            });
            
            ctx.stroke();
        }
        
        // 绘制内存曲线
        if (this.memoryHistory.length > 1) {
            ctx.strokeStyle = '#00FF88';
            ctx.lineWidth = 2;
            ctx.beginPath();
            
            this.memoryHistory.forEach((point, index) => {
                const x = (width / (this.maxDataPoints - 1)) * index;
                const y = height - (point.value / 100) * height;
                
                if (index === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
            });
            
            ctx.stroke();
        }
        
        // 绘制图例
        ctx.font = '12px -apple-system, BlinkMacSystemFont, sans-serif';
        ctx.fillStyle = '#00D9FF';
        ctx.fillRect(10, 10, 12, 12);
        ctx.fillStyle = '#FFFFFF';
        ctx.fillText('CPU', 28, 20);
        
        ctx.fillStyle = '#00FF88';
        ctx.fillRect(80, 10, 12, 12);
        ctx.fillStyle = '#FFFFFF';
        ctx.fillText('内存', 98, 20);
    }
}

// 初始化插件
const plugin = new SystemMonitorPlugin();
document.addEventListener('DOMContentLoaded', () => {
    plugin.initialize();
});