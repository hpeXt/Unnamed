//! Test Receiver 插件
//! 
//! 用于端到端消息传递测试的接收者插件

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct IncomingMessage {
    from: String,
    to: String,
    payload: Vec<u8>,
    msg_type: Option<String>,
    timestamp: String,
    id: String,
}

#[derive(Serialize, Deserialize)]
struct MessagePayload {
    #[serde(rename = "type")]
    msg_type: String,
    content: String,
    test_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    batch_index: Option<usize>,
    timestamp: String,
}

// 声明主机函数
#[host_fn]
extern "ExtismHost" {
    fn send_message_host(input: String) -> String;
    fn log_message_host(level: String, message: String) -> String;
    fn store_data_host(key: String, value: String) -> String;
    fn get_data_host(key: String) -> String;
}

/// 初始化插件
#[plugin_fn]
pub fn init() -> FnResult<String> {
    unsafe {
        log_message_host("info".to_string(), "Test Receiver plugin initialized".to_string())?;
        
        // 初始化接收计数器
        store_data_host(
            json!({
                "plugin_id": "test_receiver",
                "key": "message_count",
                "value": 0
            }).to_string(),
            "".to_string()
        )?;
    }
    Ok("Test Receiver ready".to_string())
}

/// 处理接收到的消息
#[plugin_fn]
pub fn process_message(input: String) -> FnResult<String> {
    let message: IncomingMessage = serde_json::from_str(&input)?;
    
    // 解析消息负载
    let payload: MessagePayload = serde_json::from_slice(&message.payload)?;
    
    unsafe {
        log_message_host(
            "info".to_string(), 
            format!("Received {} message from {}: {}", payload.msg_type, message.from, payload.content)
        )?;
    }
    
    // 更新消息计数
    let count_key = "message_count";
    let current_count = unsafe {
        let result = get_data_host(json!({
            "plugin_id": "test_receiver",
            "key": count_key
        }).to_string())?;
        
        let data: serde_json::Value = serde_json::from_str(&result)?;
        data["value"].as_i64().unwrap_or(0)
    };
    
    let new_count = current_count + 1;
    unsafe {
        store_data_host(
            json!({
                "plugin_id": "test_receiver",
                "key": count_key,
                "value": new_count
            }).to_string(),
            "".to_string()
        )?;
    }
    
    // 存储接收到的消息
    let msg_key = format!("received_{}_{}", payload.test_id, message.id);
    unsafe {
        store_data_host(
            json!({
                "plugin_id": "test_receiver",
                "key": msg_key,
                "value": json!({
                    "message_id": message.id,
                    "from": message.from,
                    "payload": payload,
                    "received_at": chrono::Utc::now().to_rfc3339(),
                    "sequence": new_count
                })
            }).to_string(),
            "".to_string()
        )?;
    }
    
    // 如果是测试消息，发送确认回复
    if payload.msg_type == "test_message" {
        let reply = json!({
            "from": "test_receiver",
            "to": message.from,
            "payload": serde_json::to_vec(&json!({
                "type": "test_reply",
                "original_id": message.id,
                "test_id": payload.test_id,
                "content": format!("Received: {}", payload.content),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))?
        });
        
        let reply_id = unsafe {
            send_message_host(reply.to_string())?
        };
        
        unsafe {
            log_message_host(
                "info".to_string(),
                format!("Sent reply {} for message {}", reply_id, message.id)
            )?;
        }
    }
    
    Ok(json!({
        "success": true,
        "message_id": message.id,
        "test_id": payload.test_id,
        "sequence": new_count
    }).to_string())
}

/// 获取特定测试的接收消息
#[plugin_fn]
pub fn get_test_messages(test_id: String) -> FnResult<String> {
    let mut messages: Vec<serde_json::Value> = Vec::new();
    
    // 这里简化处理，实际应该查询所有以 test_id 为前缀的键
    // 由于插件 API 限制，这里只返回计数
    let count_key = "message_count";
    let count = unsafe {
        let result = get_data_host(json!({
            "plugin_id": "test_receiver",
            "key": count_key
        }).to_string())?;
        
        let data: serde_json::Value = serde_json::from_str(&result)?;
        data["value"].as_i64().unwrap_or(0)
    };
    
    Ok(json!({
        "test_id": test_id,
        "total_received": count,
        "status": "active"
    }).to_string())
}

/// 获取插件统计信息
#[plugin_fn]
pub fn get_stats() -> FnResult<String> {
    let count = unsafe {
        let result = get_data_host(json!({
            "plugin_id": "test_receiver",
            "key": "message_count"
        }).to_string())?;
        
        let data: serde_json::Value = serde_json::from_str(&result)?;
        data["value"].as_i64().unwrap_or(0)
    };
    
    Ok(json!({
        "plugin": "test_receiver",
        "total_messages_received": count,
        "status": "active",
        "version": "0.1.0"
    }).to_string())
}

/// 重置接收计数器（用于测试清理）
#[plugin_fn]
pub fn reset_stats() -> FnResult<String> {
    unsafe {
        store_data_host(
            json!({
                "plugin_id": "test_receiver",
                "key": "message_count",
                "value": 0
            }).to_string(),
            "".to_string()
        )?;
        
        log_message_host("info".to_string(), "Test Receiver stats reset".to_string())?;
    }
    
    Ok(json!({
        "success": true,
        "message": "Stats reset"
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