//! 插件清单文件解析
//!
//! 支持 manifest.toml 格式的插件元数据

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 插件清单结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// 插件基本信息
    pub plugin: PluginInfo,
    /// 依赖信息
    #[serde(default)]
    pub dependencies: Dependencies,
    /// 元数据
    #[serde(default)]
    pub metadata: Metadata,
}

/// 插件基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 作者
    #[serde(default)]
    pub author: Option<String>,
}

/// 依赖信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dependencies {
    /// 必需依赖
    #[serde(default)]
    pub requires: Vec<String>,
    /// 可选依赖
    #[serde(default)]
    pub optional: Vec<String>,
}

/// 元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
    /// 最小内核版本要求
    #[serde(default)]
    pub min_kernel_version: Option<String>,
    /// 自定义字段
    #[serde(flatten)]
    pub custom: HashMap<String, toml::Value>,
}

impl PluginManifest {
    /// 从文件加载清单
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| anyhow!("读取清单文件失败: {}", e))?;

        Self::parse_manifest(&content)
    }

    /// 从字符串解析清单
    pub fn parse_manifest(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| anyhow!("解析清单文件失败: {}", e))
    }

    /// 创建默认清单
    pub fn default_for_plugin(name: &str) -> Self {
        Self {
            plugin: PluginInfo {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: format!("{name} 插件"),
                author: None,
            },
            dependencies: Dependencies::default(),
            metadata: Metadata::default(),
        }
    }

    /// 获取所有依赖（必需 + 可选）
    pub fn all_dependencies(&self) -> Vec<String> {
        let mut deps = self.dependencies.requires.clone();
        deps.extend(self.dependencies.optional.clone());
        deps
    }

    /// 检查是否兼容指定的内核版本
    pub fn is_compatible_with_kernel(&self, kernel_version: &str) -> bool {
        if let Some(min_version) = &self.metadata.min_kernel_version {
            // 这里应该使用 semver 比较，为简单起见使用字符串比较
            min_version.as_str() <= kernel_version
        } else {
            true // 没有版本要求，认为兼容
        }
    }
}

/// 查找并读取插件清单文件
///
/// 搜索顺序：
/// 1. wasm 文件同目录的 manifest.toml
/// 2. wasm 文件父目录的 manifest.toml
/// 3. wasm 文件所在插件目录的 manifest.toml
pub fn find_and_read_manifest(wasm_path: &Path) -> Result<PluginManifest> {
    let mut search_paths = vec![
        // 同目录
        wasm_path
            .parent()
            .unwrap_or(wasm_path)
            .join("manifest.toml"),
    ];

    // 父目录
    if let Some(parent) = wasm_path.parent().and_then(|p| p.parent()) {
        search_paths.push(parent.join("manifest.toml"));
    }

    // 插件目录根
    if let Some(root) = wasm_path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
    {
        search_paths.push(root.join("manifest.toml"));
    }

    for path in search_paths {
        if path.exists() {
            tracing::debug!("找到清单文件: {}", path.display());
            return PluginManifest::from_file(&path);
        }
    }

    // 如果没找到清单文件，从文件名创建默认清单
    let plugin_name = wasm_path
        .file_stem()
        .ok_or_else(|| anyhow!("无效的插件路径: {}", wasm_path.display()))?
        .to_string_lossy()
        .to_string();

    tracing::debug!("未找到清单文件，使用默认配置: {}", plugin_name);
    Ok(PluginManifest::default_for_plugin(&plugin_name))
}

/// 生成示例清单文件内容
pub fn generate_example_manifest(plugin_name: &str) -> String {
    format!(
        r#"# Minimal Kernel 插件清单文件

[plugin]
name = "{plugin_name}"
version = "0.1.0"
description = "{plugin_name} 插件的简要描述"
author = "Your Name"

[dependencies]
# 必需的插件依赖
requires = []
# 可选的插件依赖
optional = []

[metadata]
# 插件标签，用于分类和搜索
tags = ["example", "demo"]
# 支持的最小内核版本
min_kernel_version = "0.1.0"

# 自定义元数据字段
[metadata.custom]
license = "MIT"
homepage = "https://github.com/your-username/{plugin_name}"
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_manifest() {
        let manifest_content = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
description = "测试插件"
author = "Test Author"

[dependencies]
requires = ["base-plugin"]
optional = ["extra-plugin"]

[metadata]
tags = ["test", "example"]
min_kernel_version = "0.1.0"
"#;

        let manifest = PluginManifest::parse_manifest(manifest_content).unwrap();

        assert_eq!(manifest.plugin.name, "test-plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert_eq!(manifest.dependencies.requires, vec!["base-plugin"]);
        assert_eq!(manifest.dependencies.optional, vec!["extra-plugin"]);
        assert_eq!(manifest.metadata.tags, vec!["test", "example"]);
    }

    #[test]
    fn test_find_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");
        let target_dir = plugin_dir.join("target/wasm32-unknown-unknown/release");
        fs::create_dir_all(&target_dir).unwrap();

        // 创建清单文件
        let manifest_path = plugin_dir.join("manifest.toml");
        let manifest_content = generate_example_manifest("test-plugin");
        fs::write(&manifest_path, manifest_content).unwrap();

        // 创建 wasm 文件
        let wasm_path = target_dir.join("test_plugin.wasm");
        fs::write(&wasm_path, b"fake wasm content").unwrap();

        // 测试查找清单
        let manifest = find_and_read_manifest(&wasm_path).unwrap();
        assert_eq!(manifest.plugin.name, "test-plugin");
    }

    #[test]
    fn test_default_manifest() {
        let manifest = PluginManifest::default_for_plugin("my-plugin");
        assert_eq!(manifest.plugin.name, "my-plugin");
        assert_eq!(manifest.plugin.version, "0.1.0");
        assert!(manifest.dependencies.requires.is_empty());
    }

    #[test]
    fn test_version_compatibility() {
        let mut manifest = PluginManifest::default_for_plugin("test");

        // 无版本要求
        assert!(manifest.is_compatible_with_kernel("0.1.0"));

        // 有版本要求
        manifest.metadata.min_kernel_version = Some("0.1.0".to_string());
        assert!(manifest.is_compatible_with_kernel("0.1.0"));
        assert!(manifest.is_compatible_with_kernel("0.2.0"));
        assert!(!manifest.is_compatible_with_kernel("0.0.9"));
    }
}
