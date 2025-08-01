use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    #[serde(rename = "type")]
    pub plugin_type: String,
    pub features: Vec<String>,
    pub icon: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePluginResult {
    pub success: bool,
    pub path: String,
    pub message: String,
}

pub async fn create_plugin_from_template(config: PluginConfig) -> Result<CreatePluginResult> {
    // 验证插件名称格式
    if !config.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Ok(CreatePluginResult {
            success: false,
            path: String::new(),
            message: "插件名称只能包含小写字母、数字和连字符".to_string(),
        });
    }

    // 获取项目根目录
    let project_root = std::env::current_dir()
        .context("无法获取当前目录")?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("无法获取项目根目录"))?
        .to_path_buf();

    // 创建插件目录
    let plugin_dir = project_root.join("plugins").join(&config.name);
    
    // 检查插件是否已存在
    if plugin_dir.exists() {
        return Ok(CreatePluginResult {
            success: false,
            path: plugin_dir.to_string_lossy().to_string(),
            message: format!("插件 '{}' 已存在", config.name),
        });
    }

    // 创建目录结构
    fs::create_dir_all(plugin_dir.join("src"))
        .context("创建插件目录失败")?;

    // 生成 Cargo.toml
    let cargo_toml = generate_cargo_toml(&config);
    fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)
        .context("写入 Cargo.toml 失败")?;

    // 生成 src/lib.rs
    let lib_rs = generate_lib_rs(&config);
    fs::write(plugin_dir.join("src/lib.rs"), lib_rs)
        .context("写入 lib.rs 失败")?;

    // 生成 README.md
    let readme = generate_readme(&config);
    fs::write(plugin_dir.join("README.md"), readme)
        .context("写入 README.md 失败")?;

    // 添加到工作空间（如果需要）
    update_workspace_members(&project_root, &config.name)?;

    Ok(CreatePluginResult {
        success: true,
        path: plugin_dir.to_string_lossy().to_string(),
        message: format!("插件 '{}' 创建成功", config.display_name),
    })
}

fn generate_cargo_toml(config: &PluginConfig) -> String {
    format!(r#"[workspace]

[package]
name = "{}"
version = "{}"
edition = "2021"
authors = ["{}"]
description = "{}"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
chrono = "0.4"
"#, 
        config.name,
        config.version,
        config.author,
        config.description
    )
}

fn generate_lib_rs(config: &PluginConfig) -> String {
    match config.plugin_type.as_str() {
        "data-collector" => generate_data_collector_template(config),
        "analyzer" => generate_analyzer_template(config),
        "ui-widget" => generate_ui_widget_template(config),
        _ => generate_basic_template(config),
    }
}

fn generate_data_collector_template(config: &PluginConfig) -> String {
    format!(r#"//! {}
//! 
//! {}
//! 
//! 作者: {}

use extism_pdk::*;
use serde::{{Deserialize, Serialize}};
use serde_json::json;
use chrono::{{DateTime, Utc}};

// 声明主机函数
#[host_fn]
extern "ExtismHost" {{
    fn store_data_host(plugin_id: &str, key: &str, value: &str) -> String;
    fn get_data_host(plugin_id: &str, key: &str) -> String;
    fn log_message_host(level: &str, message: &str) -> String;
}}

const PLUGIN_ID: &str = "{}";

#[derive(Serialize, Deserialize)]
struct HealthData {{
    timestamp: String,
    value: f64,
    unit: String,
    metadata: serde_json::Value,
}}

/// 插件初始化
#[plugin_fn]
pub fn init() -> FnResult<String> {{
    unsafe {{
        log_message_host("info", &format!("{{}} 初始化成功", "{}"))?;
    }}
    Ok(json!({{
        "success": true,
        "message": "插件初始化成功"
    }}).to_string())
}}

/// 收集数据
#[plugin_fn]
pub fn collect_data() -> FnResult<String> {{
    // TODO: 实现数据收集逻辑
    let data = HealthData {{
        timestamp: Utc::now().to_rfc3339(),
        value: 72.0, // 示例数据
        unit: "bpm".to_string(),
        metadata: json!({{
            "source": "manual",
            "quality": "good"
        }}),
    }};
    
    // 存储数据
    let key = format!("data_{{}}", Utc::now().timestamp());
    unsafe {{
        store_data_host(PLUGIN_ID, &key, &serde_json::to_string(&data)?)?;
        log_message_host("info", &format!("数据已收集: {{:?}}", data))?;
    }}
    
    Ok(json!({{
        "success": true,
        "data": data
    }}).to_string())
}}

/// 获取最新数据
#[plugin_fn]
pub fn get_latest_data() -> FnResult<String> {{
    // TODO: 实现获取最新数据的逻辑
    Ok(json!({{
        "success": true,
        "message": "获取数据功能待实现"
    }}).to_string())
}}

/// 获取插件信息
#[plugin_fn]
pub fn info() -> FnResult<String> {{
    Ok(json!({{
        "id": PLUGIN_ID,
        "name": "{}",
        "version": "{}",
        "type": "data-collector",
        "icon": "{}",
        "description": "{}"
    }}).to_string())
}}
"#, 
        config.display_name,
        config.description,
        config.author,
        config.name,
        config.display_name,
        config.display_name,
        config.version,
        config.icon,
        config.description
    )
}

fn generate_analyzer_template(config: &PluginConfig) -> String {
    format!(r#"//! {}
//! 
//! {}
//! 
//! 作者: {}

use extism_pdk::*;
use serde::{{Deserialize, Serialize}};
use serde_json::json;

// 声明主机函数
#[host_fn]
extern "ExtismHost" {{
    fn get_data_host(plugin_id: &str, key: &str) -> String;
    fn list_keys_host(plugin_id: &str) -> String;
    fn log_message_host(level: &str, message: &str) -> String;
}}

const PLUGIN_ID: &str = "{}";

#[derive(Serialize, Deserialize)]
struct AnalysisResult {{
    summary: String,
    insights: Vec<String>,
    recommendations: Vec<String>,
    statistics: serde_json::Value,
}}

/// 插件初始化
#[plugin_fn]
pub fn init() -> FnResult<String> {{
    unsafe {{
        log_message_host("info", &format!("{{}} 初始化成功", "{}"))?;
    }}
    Ok(json!({{
        "success": true,
        "message": "分析器初始化成功"
    }}).to_string())
}}

/// 分析数据
#[plugin_fn]
pub fn analyze(input: String) -> FnResult<String> {{
    // 解析输入参数
    #[derive(Deserialize)]
    struct AnalyzeRequest {{
        target_plugin: String,
        time_range: Option<String>,
    }}
    
    let request: AnalyzeRequest = serde_json::from_str(&input)?;
    
    unsafe {{
        log_message_host("info", &format!("开始分析 {{}} 的数据", request.target_plugin))?;
    }}
    
    // TODO: 实现数据分析逻辑
    let result = AnalysisResult {{
        summary: "数据分析完成".to_string(),
        insights: vec![
            "平均值处于正常范围".to_string(),
            "检测到轻微上升趋势".to_string(),
        ],
        recommendations: vec![
            "建议保持当前状态".to_string(),
        ],
        statistics: json!({{
            "count": 100,
            "average": 75.5,
            "min": 60,
            "max": 90
        }}),
    }};
    
    Ok(json!({{
        "success": true,
        "result": result
    }}).to_string())
}}

/// 生成报告
#[plugin_fn]
pub fn generate_report(input: String) -> FnResult<String> {{
    // TODO: 实现报告生成逻辑
    Ok(json!({{
        "success": true,
        "message": "报告生成功能待实现"
    }}).to_string())
}}

/// 获取插件信息
#[plugin_fn]
pub fn info() -> FnResult<String> {{
    Ok(json!({{
        "id": PLUGIN_ID,
        "name": "{}",
        "version": "{}",
        "type": "analyzer",
        "icon": "{}",
        "description": "{}"
    }}).to_string())
}}
"#,
        config.display_name,
        config.description,
        config.author,
        config.name,
        config.display_name,
        config.display_name,
        config.version,
        config.icon,
        config.description
    )
}

fn generate_ui_widget_template(config: &PluginConfig) -> String {
    format!(r#"//! {}
//! 
//! {}
//! 
//! 作者: {}

use extism_pdk::*;
use serde::{{Deserialize, Serialize}};
use serde_json::json;

// 声明主机函数
#[host_fn]
extern "ExtismHost" {{
    fn subscribe_data_host(plugin_id: &str, target: &str) -> String;
    fn send_message_host(from: &str, to: &str, payload: &str) -> String;
    fn log_message_host(level: &str, message: &str) -> String;
}}

const PLUGIN_ID: &str = "{}";

#[derive(Serialize, Deserialize)]
struct WidgetData {{
    title: String,
    value: serde_json::Value,
    unit: Option<String>,
    chart_data: Option<Vec<f64>>,
}}

/// 插件初始化
#[plugin_fn]
pub fn init() -> FnResult<String> {{
    unsafe {{
        log_message_host("info", &format!("{{}} UI 组件初始化成功", "{}"))?;
    }}
    
    // TODO: 订阅需要的数据源
    
    Ok(json!({{
        "success": true,
        "message": "UI 组件初始化成功"
    }}).to_string())
}}

/// 获取组件数据
#[plugin_fn]
pub fn get_widget_data() -> FnResult<String> {{
    // TODO: 实现获取组件显示数据的逻辑
    let data = WidgetData {{
        title: "{}".to_string(),
        value: json!(75),
        unit: Some("单位".to_string()),
        chart_data: Some(vec![60.0, 65.0, 70.0, 75.0, 72.0]),
    }};
    
    Ok(json!({{
        "success": true,
        "data": data
    }}).to_string())
}}

/// 处理用户交互
#[plugin_fn]
pub fn handle_interaction(input: String) -> FnResult<String> {{
    #[derive(Deserialize)]
    struct InteractionEvent {{
        event_type: String,
        payload: serde_json::Value,
    }}
    
    let event: InteractionEvent = serde_json::from_str(&input)?;
    
    unsafe {{
        log_message_host("info", &format!("处理交互事件: {{}}", event.event_type))?;
    }}
    
    // TODO: 根据事件类型处理用户交互
    
    Ok(json!({{
        "success": true,
        "message": "交互处理成功"
    }}).to_string())
}}

/// 获取插件信息
#[plugin_fn]
pub fn info() -> FnResult<String> {{
    Ok(json!({{
        "id": PLUGIN_ID,
        "name": "{}",
        "version": "{}",
        "type": "ui-widget",
        "icon": "{}",
        "description": "{}"
    }}).to_string())
}}
"#,
        config.display_name,
        config.description,
        config.author,
        config.name,
        config.display_name,
        config.display_name,
        config.display_name,
        config.version,
        config.icon,
        config.description
    )
}

fn generate_basic_template(config: &PluginConfig) -> String {
    // 基础模板，类似 hello 插件
    generate_data_collector_template(config)
}

fn generate_readme(config: &PluginConfig) -> String {
    format!(r#"# {} {}

{}

## 功能特性

- 类型: {}
- 版本: {}
- 作者: {}

## 开发指南

### 构建插件

```bash
cargo build --target wasm32-unknown-unknown --release
```

### 测试插件

```bash
# 在项目根目录运行
cargo run -- --plugin-dir plugins/{}
```

## API 文档

### 导出函数

- `init()` - 插件初始化
- `info()` - 获取插件信息

## 许可证

MIT License
"#,
        config.icon,
        config.display_name,
        config.description,
        config.plugin_type,
        config.version,
        config.author,
        config.name
    )
}

fn update_workspace_members(project_root: &PathBuf, plugin_name: &str) -> Result<()> {
    // 读取根 Cargo.toml
    let cargo_toml_path = project_root.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        // 如果根目录没有 Cargo.toml，不需要更新
        return Ok(());
    }

    let content = fs::read_to_string(&cargo_toml_path)?;
    
    // 检查是否已经包含该插件
    let plugin_path = format!("plugins/{}", plugin_name);
    if content.contains(&plugin_path) {
        return Ok(());
    }

    // 查找 workspace.members 部分
    if let Some(members_start) = content.find("members = [") {
        if let Some(members_end) = content[members_start..].find(']') {
            let members_end = members_start + members_end;
            
            // 在 members 数组末尾添加新插件
            let before = &content[..members_end];
            let after = &content[members_end..];
            
            // 查找最后一个逗号的位置，确保格式正确
            let insert_pos = before.rfind(',').map(|p| p + 1).unwrap_or(members_end);
            let indent = "    "; // 4 spaces
            
            let new_content = format!(
                "{}\n{}\"plugins/{}\",{}",
                &content[..insert_pos],
                indent,
                plugin_name,
                &content[insert_pos..]
            );
            
            fs::write(cargo_toml_path, new_content)?;
        }
    }

    Ok(())
}