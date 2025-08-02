// 主入口文件 - 使用动态导入优化初始加载

// 加载状态管理
const loadingState = {
  modules: new Map(),
  startTime: performance.now(),
  
  track(name, promise) {
    this.modules.set(name, { loading: true, startTime: performance.now() });
    return promise
      .then(module => {
        const loadTime = performance.now() - this.modules.get(name).startTime;
        this.modules.set(name, { loading: false, loaded: true, loadTime });
        console.log(`Module ${name} loaded in ${loadTime.toFixed(2)}ms`);
        return module;
      })
      .catch(err => {
        this.modules.set(name, { loading: false, error: err });
        console.error(`Failed to load module ${name}:`, err);
        throw err;
      });
  },
  
  getLoadingSummary() {
    const totalTime = performance.now() - this.startTime;
    const modules = Array.from(this.modules.entries()).map(([name, state]) => ({
      name,
      ...state
    }));
    return { totalTime, modules };
  }
};

// 显示加载指示器
function showLoadingIndicator() {
  const app = document.getElementById('app');
  if (app) {
    app.classList.add('loading');
    const indicator = app.querySelector('.loading-indicator');
    if (indicator) {
      indicator.style.display = 'block';
    }
  }
}

// 隐藏加载指示器
function hideLoadingIndicator() {
  const app = document.getElementById('app');
  if (app) {
    app.classList.remove('loading');
    app.classList.add('loaded');
    const indicator = app.querySelector('.loading-indicator');
    if (indicator) {
      indicator.style.display = 'none';
    }
  }
}

// 错误边界
function showErrorBoundary(error) {
  const app = document.getElementById('app');
  if (app) {
    app.innerHTML = `
      <div class="error-boundary">
        <h1>😔 应用加载失败</h1>
        <p>${error.message}</p>
        <details>
          <summary>错误详情</summary>
          <pre>${error.stack}</pre>
        </details>
        <button onclick="location.reload()">重新加载</button>
      </div>
    `;
  }
}

// 主初始化函数
async function initializeApp() {
  console.log('Starting application initialization...');
  showLoadingIndicator();
  
  try {
    // 并行加载所有必需的模块
    const modulePromises = {
      tauriApi: loadingState.track('tauri-api', 
        import('@tauri-apps/api/core').then(m => ({ 
          invoke: m.invoke 
        }))
      ),
      tauriEvent: loadingState.track('tauri-event', 
        import('@tauri-apps/api/event').then(m => ({ 
          listen: m.listen, 
          emit: m.emit 
        }))
      ),
      tauriApp: loadingState.track('tauri-app', 
        import('@tauri-apps/api/app').then(m => ({ 
          getVersion: m.getVersion 
        }))
      ),
      utils: loadingState.track('utils', 
        import('../js/utils.js')
      ),
      components: loadingState.track('components', 
        Promise.all([
          import('../components/onboarding/onboarding.js'),
          import('../components/plugin-selector/plugin-selector.js'),
          import('../components/plugin-creator/plugin-creator.js'),
          import('../components/debug-console/debug-console.js')
        ])
      )
    };
    
    // 等待所有模块加载完成
    const modules = await Promise.all(
      Object.entries(modulePromises).map(async ([key, promise]) => {
        try {
          const module = await promise;
          return { key, module };
        } catch (error) {
          console.error(`Failed to load ${key}:`, error);
          return { key, error };
        }
      })
    );
    
    // 解构加载的模块
    const loadedModules = {};
    const failedModules = [];
    
    modules.forEach(({ key, module, error }) => {
      if (error) {
        failedModules.push({ key, error });
      } else {
        loadedModules[key] = module;
      }
    });
    
    // 如果有关键模块加载失败，显示错误
    if (failedModules.length > 0) {
      console.error('Failed to load modules:', failedModules);
      // 继续运行，但功能可能受限
    }
    
    // 导入主应用模块
    const { initApp } = await loadingState.track('main-module', 
      import('../js/main-module.js')
    );
    
    // 打印加载统计
    const summary = loadingState.getLoadingSummary();
    console.log(`All modules loaded in ${summary.totalTime.toFixed(2)}ms`, summary);
    
    // 初始化应用
    await initApp(loadedModules);
    
    // 隐藏加载指示器
    hideLoadingIndicator();
    
  } catch (error) {
    console.error('Failed to initialize application:', error);
    showErrorBoundary(error);
  }
}

// 全局错误处理
window.addEventListener('error', (event) => {
  console.error('Global error:', event.error);
  // 只在初始化阶段显示错误边界
  if (document.getElementById('app').classList.contains('loading')) {
    showErrorBoundary(event.error);
  }
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled promise rejection:', event.reason);
  // 只在初始化阶段显示错误边界
  if (document.getElementById('app').classList.contains('loading')) {
    showErrorBoundary(new Error(event.reason));
  }
});

// 确保在正确的时机初始化
if (document.readyState === 'complete') {
  // 如果页面已经加载完成，稍微延迟以确保 Tauri API 就绪
  setTimeout(initializeApp, 50);
} else {
  // 使用 load 事件确保所有资源都已加载
  window.addEventListener('load', () => {
    // 额外的小延迟确保 Tauri API 完全就绪
    setTimeout(initializeApp, 50);
  });
}

// 导出给开发工具使用
window.__APP_DEBUG__ = {
  loadingState,
  reinitialize: initializeApp
};