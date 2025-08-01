//! Hello World 插件
//!
//! 最小化内核的第一个示例插件

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct MessageRequest {
    to: String,
    content: String,
}

// 声明主机函数 - 注意需要匹配主机端注册的函数名
#[host_fn]
extern "ExtismHost" {
    fn store_data_host(plugin_id: &str, key: &str, value: &str) -> String;
    fn get_data_host(plugin_id: &str, key: &str) -> String;
    fn delete_data_host(plugin_id: &str, key: &str) -> String;
    fn list_keys_host(plugin_id: &str) -> String;
    fn send_message_host(from: &str, to: &str, payload: &str) -> String;
    fn log_message_host(level: &str, message: &str) -> String;
}

/// 简单的问候函数
#[plugin_fn]
pub fn greet() -> FnResult<String> {
    // 使用主机日志功能
    unsafe {
        log_message_host("info", "Hello plugin started")?;
    }
    Ok("Hello from plugin!".to_string())
}

/// 带参数的问候函数
#[plugin_fn]
pub fn greet_name(name: String) -> FnResult<String> {
    Ok(format!("Hello, {}! Welcome to Minimal Kernel.", name))
}

/// 获取插件信息
#[plugin_fn]
pub fn info() -> FnResult<String> {
    Ok(serde_json::json!({
        "name": "hello",
        "version": "0.1.0",
        "description": "Hello World plugin for minimal kernel"
    })
    .to_string())
}

/// 发送消息给其他插件
#[plugin_fn]
pub fn send_greeting(input: String) -> FnResult<String> {
    let request: MessageRequest = serde_json::from_str(&input)?;

    info!("准备发送问候到 {}", request.to);

    // 记录日志
    unsafe {
        log_message_host("info", &format!("Sending greeting to {}", request.to))?;
    }

    // 调用主机函数发送消息
    let msg_id = unsafe { send_message_host("hello", &request.to, &request.content)? };

    Ok(format!("已发送消息到 {}，消息ID: {}", request.to, msg_id))
}

/// 演示数据存储功能
#[plugin_fn]
pub fn save_greeting(name: String) -> FnResult<String> {
    let plugin_id = "hello";
    let key = format!("greeting_{}", name);
    let value = json!({
        "greeting": format!("Hello, {}!", name),
        "timestamp": "2024-07-17"
    });

    // 存储数据
    unsafe {
        store_data_host(plugin_id, &key, &value.to_string())?;
    }

    Ok(format!("已保存问候语给 {}", name))
}

/// 演示数据读取功能
#[plugin_fn]
pub fn load_greeting(name: String) -> FnResult<String> {
    let plugin_id = "hello";
    let key = format!("greeting_{}", name);

    // 读取数据
    let data = unsafe { get_data_host(plugin_id, &key)? };

    // 解析返回的 JSON
    #[derive(Deserialize)]
    struct GetDataResponse {
        success: bool,
        value: Option<serde_json::Value>,
    }

    let response: GetDataResponse = serde_json::from_str(&data)?;
    if response.success && response.value.is_some() {
        Ok(format!("Retrieved: {}", response.value.unwrap()))
    } else {
        Ok(format!("No greeting found for {}", name))
    }
}

/// 演示列出所有键
#[plugin_fn]
pub fn list_greetings() -> FnResult<String> {
    let plugin_id = "hello";

    // 列出所有键
    let keys_json = unsafe { list_keys_host(plugin_id)? };

    // 解析返回的 JSON
    #[derive(Deserialize)]
    struct ListKeysResponse {
        success: bool,
        keys: Vec<String>,
    }

    let response: ListKeysResponse = serde_json::from_str(&keys_json)?;
    if response.success {
        Ok(format!("Stored greetings: {:?}", response.keys))
    } else {
        Ok("Failed to list greetings".to_string())
    }
}
