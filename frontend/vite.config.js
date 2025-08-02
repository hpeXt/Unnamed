import { defineConfig } from 'vite'

// 自定义 Tauri 集成插件
function tauriIntegrationPlugin() {
  return {
    name: 'tauri-integration',
    
    transformIndexHtml(html) {
      // 在开发模式下注入调试工具
      if (process.env.NODE_ENV === 'development') {
        return html.replace(
          '</body>',
          '<script src="/src/dev-tools.js"></script></body>'
        );
      }
      return html;
    },
    
    configureServer(server) {
      // 配置 HMR 以支持 Tauri 特性
      server.ws.on('connection', () => {
        console.log('Tauri dev tools connected');
      });
    }
  };
}

export default defineConfig({
  // 基础路径配置
  base: './',
  
  // 插件配置
  plugins: [tauriIntegrationPlugin()],
  
  // 依赖优化配置
  optimizeDeps: {
    include: ['@tauri-apps/api'], // 显式包含 Tauri API
    esbuildOptions: {
      target: 'esnext' // 使用最新的 ES 特性
    }
  },
  
  // 构建配置
  build: {
    // 目标现代浏览器（Tauri WebView）
    target: 'esnext',
    
    // 输出目录
    outDir: 'dist',
    
    // 清空输出目录
    emptyOutDir: true,
    
    // Rollup 选项
    rollupOptions: {
      input: 'index.html',
      output: {
        // 智能代码分割
        manualChunks(id) {
          // 将 Tauri API 单独分块
          if (id.includes('@tauri-apps/api')) {
            return 'tauri-api';
          }
          // 将组件分块
          if (id.includes('/components/')) {
            const componentName = id.split('/components/')[1].split('/')[0];
            return `component-${componentName}`;
          }
          // 将第三方依赖分块
          if (id.includes('node_modules')) {
            return 'vendor';
          }
        },
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash].[ext]'
      }
    },
    
    // 使用 esbuild 进行更快的压缩
    minify: 'esbuild',
    
    // 生成但不引用 sourcemap（用于调试但不影响加载）
    sourcemap: 'hidden',
    
    // CSS 代码分割
    cssCodeSplit: true,
    
    // 启用 Rollup 的监视器
    watch: process.env.NODE_ENV === 'development' ? {} : null,
    
    // 块大小警告限制（KB）
    chunkSizeWarningLimit: 1000,
    
    // 资源内联限制（4KB）
    assetsInlineLimit: 4096
  },
  
  // 开发服务器配置
  server: {
    // 端口
    port: 3000,
    
    // 启用 CORS
    cors: true,
    
    // 监听所有地址（便于 Tauri 访问）
    host: true,
    
    // 自动打开浏览器
    open: false,
    
    // HMR 配置
    hmr: {
      overlay: true
    },
    
    // 文件系统监视选项
    watch: {
      usePolling: false,
      interval: 100
    }
  },
  
  // 预览服务器配置
  preview: {
    port: 4173,
    host: true
  },
  
  // 定义全局常量
  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version)
  },
  
  // 解析配置
  resolve: {
    alias: {
      '@': '/src',
      '@components': '/components',
      '@utils': '/js/utils'
    }
  }
})