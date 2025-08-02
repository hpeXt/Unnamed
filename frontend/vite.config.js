import { defineConfig } from 'vite'

export default defineConfig({
  // 确保相对路径正确
  base: './',
  
  build: {
    // 输出目录
    outDir: 'dist',
    
    // 清空输出目录
    emptyOutDir: true,
    
    // 入口文件
    rollupOptions: {
      input: 'index.html',
      output: {
        // 不分割代码，生成单个文件
        manualChunks: undefined,
        inlineDynamicImports: true
      }
    },
    
    // 生成 sourcemap 方便调试
    sourcemap: true,
    
    // 压缩选项
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: false, // 保留 console.log 方便调试
        drop_debugger: true
      }
    }
  },
  
  server: {
    // 开发服务器端口
    port: 3000,
    
    // 自动打开浏览器
    open: false
  }
})