//! 简单依赖解析器
//!
//! 使用基础的HashMap和图遍历算法，避免复杂的依赖

use super::plugin_loader::PluginInfo;
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet, VecDeque};

/// 简单的依赖解析器
#[derive(Debug, Default)]
pub struct DependencyResolver {
    /// 插件依赖图：插件名 -> 依赖列表
    dependencies: HashMap<String, Vec<String>>,
    /// 插件信息映射
    plugins: HashMap<String, PluginInfo>,
}

impl DependencyResolver {
    /// 创建新的依赖解析器
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加插件信息
    pub fn add_plugin(&mut self, plugin: PluginInfo) {
        let deps = plugin.all_dependencies();
        self.dependencies.insert(plugin.name.clone(), deps);
        self.plugins.insert(plugin.name.clone(), plugin);
    }

    /// 添加多个插件
    pub fn add_plugins(&mut self, plugins: Vec<PluginInfo>) {
        for plugin in plugins {
            self.add_plugin(plugin);
        }
    }

    /// 解析依赖顺序（拓扑排序）
    pub fn resolve_order(&self, target_plugins: &[String]) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for plugin_name in target_plugins {
            if !visited.contains(plugin_name) {
                self.visit_plugin(plugin_name, &mut visited, &mut visiting, &mut result)?;
            }
        }

        Ok(result)
    }

    /// 深度优先搜索访问插件（递归版本）
    fn visit_plugin(
        &self,
        plugin_name: &str,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<()> {
        if visiting.contains(plugin_name) {
            return Err(anyhow!("发现循环依赖: {}", plugin_name));
        }

        if visited.contains(plugin_name) {
            return Ok(());
        }

        visiting.insert(plugin_name.to_string());

        // 访问所有依赖
        if let Some(deps) = self.dependencies.get(plugin_name) {
            for dep in deps {
                if !self.plugins.contains_key(dep) {
                    tracing::warn!("未找到依赖插件: {} (需要 {})", dep, plugin_name);
                    continue;
                }
                self.visit_plugin(dep, visited, visiting, result)?;
            }
        }

        visiting.remove(plugin_name);
        visited.insert(plugin_name.to_string());
        result.push(plugin_name.to_string());

        Ok(())
    }

    /// 检查是否存在循环依赖
    pub fn check_circular_dependencies(&self) -> Result<()> {
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut dummy_result = Vec::new();

        for plugin_name in self.plugins.keys() {
            if !visited.contains(plugin_name) {
                self.visit_plugin(plugin_name, &mut visited, &mut visiting, &mut dummy_result)?;
            }
        }

        Ok(())
    }

    /// 获取指定插件的所有依赖（递归）
    pub fn get_all_dependencies(&self, plugin_name: &str) -> Result<Vec<String>> {
        let mut deps = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(plugin_name.to_string());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(plugin_deps) = self.dependencies.get(&current) {
                for dep in plugin_deps {
                    if !visited.contains(dep) {
                        queue.push_back(dep.clone());
                        deps.push(dep.clone());
                    }
                }
            }
        }

        Ok(deps)
    }

    /// 检查依赖是否满足
    pub fn check_dependencies_satisfied(
        &self,
        plugin_name: &str,
        available_plugins: &[String],
    ) -> bool {
        if let Some(deps) = self.dependencies.get(plugin_name) {
            deps.iter().all(|dep| available_plugins.contains(dep))
        } else {
            true // 没有依赖
        }
    }

    /// 获取插件统计信息
    pub fn get_stats(&self) -> (usize, usize, usize) {
        let total_plugins = self.plugins.len();
        let total_dependencies: usize = self.dependencies.values().map(|deps| deps.len()).sum();
        let plugins_with_deps = self
            .dependencies
            .values()
            .filter(|deps| !deps.is_empty())
            .count();

        (total_plugins, total_dependencies, plugins_with_deps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn create_test_plugin(name: &str, deps: Vec<String>) -> PluginInfo {
        PluginInfo {
            name: name.to_string(),
            path: PathBuf::from(format!("{}.wasm", name)),
            file_size: 1024,
            modified: SystemTime::now(),
            loaded: false,
            version: "1.0.0".to_string(),
            description: format!("{} 测试插件", name),
            author: None,
            dependencies: deps,
            optional_dependencies: Vec::new(),
            tags: Vec::new(),
            min_kernel_version: None,
        }
    }

    #[test]
    fn test_simple_dependency_resolution() {
        let mut resolver = DependencyResolver::new();

        // A -> B -> C
        resolver.add_plugin(create_test_plugin("C", vec![]));
        resolver.add_plugin(create_test_plugin("B", vec!["C".to_string()]));
        resolver.add_plugin(create_test_plugin("A", vec!["B".to_string()]));

        let order = resolver.resolve_order(&["A".to_string()]).unwrap();
        assert_eq!(order, vec!["C", "B", "A"]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut resolver = DependencyResolver::new();

        // A -> B -> A (循环)
        resolver.add_plugin(create_test_plugin("A", vec!["B".to_string()]));
        resolver.add_plugin(create_test_plugin("B", vec!["A".to_string()]));

        let result = resolver.resolve_order(&["A".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_plugins() {
        let mut resolver = DependencyResolver::new();

        resolver.add_plugin(create_test_plugin("base", vec![]));
        resolver.add_plugin(create_test_plugin("plugin1", vec!["base".to_string()]));
        resolver.add_plugin(create_test_plugin("plugin2", vec!["base".to_string()]));

        let order = resolver
            .resolve_order(&["plugin1".to_string(), "plugin2".to_string()])
            .unwrap();
        assert!(order.contains(&"base".to_string()));
        assert!(order.contains(&"plugin1".to_string()));
        assert!(order.contains(&"plugin2".to_string()));

        // base 应该在 plugin1 和 plugin2 之前
        let base_pos = order.iter().position(|x| x == "base").unwrap();
        let plugin1_pos = order.iter().position(|x| x == "plugin1").unwrap();
        let plugin2_pos = order.iter().position(|x| x == "plugin2").unwrap();

        assert!(base_pos < plugin1_pos);
        assert!(base_pos < plugin2_pos);
    }
}
