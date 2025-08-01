// 系统监控 UI 插件
import { SystemMonitor } from '../../tauri-api.js';

export class SystemMonitorPlugin {
    constructor(container) {
        this.container = container;
        this.systemMonitor = new SystemMonitor();
        this.charts = {};
        this.init();
    }
    
    async init() {
        // 创建插件 UI
        this.container.innerHTML = `
            <div class="system-monitor-plugin">
                <h3>系统监控</h3>
                <div class="stats-grid">
                    <div class="stat-card">
                        <h4>CPU 使用率</h4>
                        <div class="stat-value" id="cpu-usage">--</div>
                        <canvas id="cpu-chart" width="200" height="50"></canvas>
                    </div>
                    <div class="stat-card">
                        <h4>内存使用率</h4>
                        <div class="stat-value" id="memory-usage">--</div>
                        <div class="memory-details">
                            <span id="memory-used">--</span> / <span id="memory-total">--</span>
                        </div>
                        <canvas id="memory-chart" width="200" height="50"></canvas>
                    </div>
                    <div class="stat-card">
                        <h4>磁盘使用率</h4>
                        <div class="stat-value" id="disk-usage">--</div>
                        <div class="disk-details">
                            <span id="disk-used">--</span> / <span id="disk-total">--</span>
                        </div>
                    </div>
                    <div class="stat-card">
                        <h4>进程数</h4>
                        <div class="stat-value" id="process-count">--</div>
                    </div>
                    <div class="stat-card">
                        <h4>网络流量</h4>
                        <div class="network-details">
                            <div>↓ <span id="network-rx">--</span></div>
                            <div>↑ <span id="network-tx">--</span></div>
                        </div>
                    </div>
                </div>
                <div class="process-list">
                    <h4>进程列表 (Top 10)</h4>
                    <div id="process-table"></div>
                </div>
            </div>
        `;
        
        this.addStyles();
        this.initCharts();
        
        // 开始监控
        await this.startMonitoring();
    }
    
    addStyles() {
        const style = document.createElement('style');
        style.textContent = `
            .system-monitor-plugin {
                padding: 10px;
                font-family: 'Consolas', 'Monaco', monospace;
            }
            
            .system-monitor-plugin h3 {
                margin: 0 0 15px 0;
                color: #2196F3;
            }
            
            .stats-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                gap: 15px;
                margin-bottom: 20px;
            }
            
            .stat-card {
                background: rgba(255, 255, 255, 0.05);
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 8px;
                padding: 15px;
            }
            
            .stat-card h4 {
                margin: 0 0 10px 0;
                font-size: 14px;
                color: #888;
            }
            
            .stat-value {
                font-size: 24px;
                font-weight: bold;
                color: #4CAF50;
                margin-bottom: 10px;
            }
            
            .memory-details, .disk-details, .network-details {
                font-size: 12px;
                color: #666;
                margin-top: 5px;
            }
            
            .process-list {
                background: rgba(255, 255, 255, 0.05);
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 8px;
                padding: 15px;
            }
            
            .process-list h4 {
                margin: 0 0 10px 0;
                color: #888;
            }
            
            #process-table {
                font-size: 12px;
            }
            
            .process-row {
                display: grid;
                grid-template-columns: 50px 1fr 80px 80px;
                padding: 5px 0;
                border-bottom: 1px solid rgba(255, 255, 255, 0.05);
            }
            
            .process-header {
                font-weight: bold;
                color: #888;
            }
            
            canvas {
                margin-top: 10px;
            }
        `;
        document.head.appendChild(style);
    }
    
    initCharts() {
        // 初始化 CPU 图表
        const cpuCanvas = document.getElementById('cpu-chart');
        const cpuCtx = cpuCanvas.getContext('2d');
        this.charts.cpu = {
            ctx: cpuCtx,
            data: new Array(50).fill(0),
            max: 100
        };
        
        // 初始化内存图表
        const memCanvas = document.getElementById('memory-chart');
        const memCtx = memCanvas.getContext('2d');
        this.charts.memory = {
            ctx: memCtx,
            data: new Array(50).fill(0),
            max: 100
        };
    }
    
    drawChart(chart) {
        const { ctx, data, max } = chart;
        const width = ctx.canvas.width;
        const height = ctx.canvas.height;
        
        // 清除画布
        ctx.clearRect(0, 0, width, height);
        
        // 绘制网格线
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
        ctx.beginPath();
        for (let i = 0; i <= 4; i++) {
            const y = (height / 4) * i;
            ctx.moveTo(0, y);
            ctx.lineTo(width, y);
        }
        ctx.stroke();
        
        // 绘制数据线
        ctx.strokeStyle = '#4CAF50';
        ctx.lineWidth = 2;
        ctx.beginPath();
        
        const stepX = width / (data.length - 1);
        data.forEach((value, index) => {
            const x = index * stepX;
            const y = height - (value / max) * height;
            
            if (index === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        });
        
        ctx.stroke();
    }
    
    async startMonitoring() {
        // 获取初始数据
        try {
            const stats = await this.systemMonitor.getStats();
            this.updateStats(stats);
            
            const processes = await this.systemMonitor.getProcesses();
            this.updateProcessList(processes);
        } catch (error) {
            console.error('Failed to get initial system stats:', error);
        }
        
        // 开始实时监控
        await this.systemMonitor.startMonitoring(1000, (stats) => {
            this.updateStats(stats);
        });
        
        // 定期更新进程列表
        this.processInterval = setInterval(async () => {
            try {
                const processes = await this.systemMonitor.getProcesses();
                this.updateProcessList(processes);
            } catch (error) {
                console.error('Failed to get processes:', error);
            }
        }, 5000);
    }
    
    updateStats(stats) {
        // 更新 CPU
        document.getElementById('cpu-usage').textContent = stats.cpu_usage.toFixed(1) + '%';
        this.charts.cpu.data.push(stats.cpu_usage);
        this.charts.cpu.data.shift();
        this.drawChart(this.charts.cpu);
        
        // 更新内存
        document.getElementById('memory-usage').textContent = stats.memory_usage.toFixed(1) + '%';
        document.getElementById('memory-used').textContent = this.formatBytes(stats.memory_used);
        document.getElementById('memory-total').textContent = this.formatBytes(stats.memory_total);
        this.charts.memory.data.push(stats.memory_usage);
        this.charts.memory.data.shift();
        this.drawChart(this.charts.memory);
        
        // 更新磁盘
        document.getElementById('disk-usage').textContent = stats.disk_usage.toFixed(1) + '%';
        document.getElementById('disk-used').textContent = this.formatBytes(stats.disk_used);
        document.getElementById('disk-total').textContent = this.formatBytes(stats.disk_total);
        
        // 更新进程数
        document.getElementById('process-count').textContent = stats.process_count;
        
        // 更新网络
        document.getElementById('network-rx').textContent = this.formatBytes(stats.network_rx) + '/s';
        document.getElementById('network-tx').textContent = this.formatBytes(stats.network_tx) + '/s';
    }
    
    updateProcessList(processes) {
        const table = document.getElementById('process-table');
        
        // 创建表格
        let html = `
            <div class="process-row process-header">
                <div>PID</div>
                <div>名称</div>
                <div>CPU %</div>
                <div>内存</div>
            </div>
        `;
        
        // 只显示前10个进程
        processes.slice(0, 10).forEach(process => {
            html += `
                <div class="process-row">
                    <div>${process.pid}</div>
                    <div>${process.name}</div>
                    <div>${process.cpu_usage.toFixed(1)}%</div>
                    <div>${this.formatBytes(process.memory_usage)}</div>
                </div>
            `;
        });
        
        table.innerHTML = html;
    }
    
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + sizes[i];
    }
    
    destroy() {
        // 停止监控
        this.systemMonitor.stopMonitoring();
        
        // 清除定时器
        if (this.processInterval) {
            clearInterval(this.processInterval);
        }
    }
}