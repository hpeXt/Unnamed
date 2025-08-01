//! Test Sender 插件
//! 
//! 用于端到端消息传递测试的发送者插件

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct SendRequest {
    to: String,
    message: String,
    test_id: String,
}

#[derive(Serialize, Deserialize)]
struct BatchSendRequest {
    to: String,
    messages: Vec<String>,
    test_id: String,
}

// 声明主机函数
#[host_fn]
extern "ExtismHost" {
    fn send_message_host(input: String) -> String;
    fn log_message_host(level: String, message: String) -> String;
    fn store_data_host(key: String, value: String) -> String;
}

/// 初始化插件
#[plugin_fn]
pub fn init() -> FnResult<String> {
    unsafe {
        log_message_host("info".to_string(), "Test Sender plugin initialized".to_string())?;
    }
    Ok("Test Sender ready".to_string())
}

/// 发送单个测试消息
#[plugin_fn]
pub fn send_test_message(input: String) -> FnResult<String> {
    let request: SendRequest = serde_json::from_str(&input)?;
    
    // 记录发送日志
    unsafe {
        log_message_host(
            "info".to_string(), 
            format!("Sending test message to {}: {}", request.to, request.message)
        )?;
    }
    
    // 构造消息
    let msg = json!({
        "from": "test_sender",
        "to": request.to,
        "payload": serde_json::to_vec(&json!({
            "type": "test_message",
            "content": request.message,
            "test_id": request.test_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))?
    });
    
    // 发送消息
    let msg_id = unsafe {
        send_message_host(msg.to_string())?
    };
    
    // 存储发送记录
    let record_key = format!("sent_{}_{}", request.test_id, msg_id);
    unsafe {
        store_data_host(
            json!({
                "plugin_id": "test_sender",
                "key": record_key,
                "value": json!({
                    "message_id": msg_id,
                    "to": request.to,
                    "content": request.message,
                    "test_id": request.test_id,
                    "sent_at": chrono::Utc::now().to_rfc3339()
                })
            }).to_string(),
            "".to_string()
        )?;
    }
    
    Ok(json!({
        "success": true,
        "message_id": msg_id,
        "test_id": request.test_id
    }).to_string())
}

/// 批量发送测试消息
#[plugin_fn]
pub fn send_batch_messages(input: String) -> FnResult<String> {
    let request: BatchSendRequest = serde_json::from_str(&input)?;
    let mut results = Vec::new();
    
    unsafe {
        log_message_host(
            "info".to_string(), 
            format!("Sending {} messages to {}", request.messages.len(), request.to)
        )?;
    }
    
    for (index, message) in request.messages.iter().enumerate() {
        let msg = json!({
            "from": "test_sender",
            "to": request.to.clone(),
            "payload": serde_json::to_vec(&json!({
                "type": "batch_test_message",
                "content": message,
                "test_id": request.test_id.clone(),
                "batch_index": index,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))?
        });
        
        let msg_id = unsafe {
            send_message_host(msg.to_string())?
        };
        
        results.push(json!({
            "index": index,
            "message_id": msg_id,
            "content": message
        }));
    }
    
    // 存储批量发送记录
    let batch_key = format!("batch_{}", request.test_id);
    unsafe {
        store_data_host(
            json!({
                "plugin_id": "test_sender",
                "key": batch_key,
                "value": json!({
                    "test_id": request.test_id,
                    "to": request.to,
                    "count": request.messages.len(),
                    "results": results,
                    "sent_at": chrono::Utc::now().to_rfc3339()
                })
            }).to_string(),
            "".to_string()
        )?;
    }
    
    Ok(json!({
        "success": true,
        "test_id": request.test_id,
        "sent_count": results.len(),
        "results": results
    }).to_string())
}

/// 测试发送到不存在的插件
#[plugin_fn]
pub fn send_to_nonexistent(input: String) -> FnResult<String> {
    let test_id = input;
    
    unsafe {
        log_message_host(
            "info".to_string(), 
            "Testing send to nonexistent plugin".to_string()
        )?;
    }
    
    // 尝试发送到不存在的插件
    let msg = json!({
        "from": "test_sender",
        "to": "nonexistent_plugin",
        "payload": serde_json::to_vec(&json!({
            "type": "error_test",
            "test_id": test_id
        }))?
    });
    
    // 这应该会失败或返回错误
    let result = unsafe {
        send_message_host(msg.to_string())?
    };
    
    Ok(json!({
        "test_id": test_id,
        "result": result
    }).to_string())
}

/// 获取插件状态
#[plugin_fn]
pub fn get_status() -> FnResult<String> {
    Ok(json!({
        "plugin": "test_sender",
        "status": "active",
        "version": "0.1.0",
        "capabilities": [
            "send_test_message",
            "send_batch_messages",
            "send_to_nonexistent"
        ]
    }).to_string())
}

// 添加 chrono 依赖的 mock
mod chrono {
    pub struct Utc;
    
    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }
    
    pub struct DateTime;
    
    impl DateTime {
        pub fn to_rfc3339(&self) -> String {
            "2025-01-01T00:00:00Z".to_string()
        }
    }
}