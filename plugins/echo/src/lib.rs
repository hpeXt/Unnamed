//! Echo 插件
//! 
//! 接收消息并回显

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct EchoMessage {
    from: String,
    content: String,
}

// 声明主机函数
#[host_fn]
extern "ExtismHost" {
    fn send_message_host(input: String) -> String;
    fn log_message_host(level: String, message: String) -> String;
    fn subscribe_topic_host(input: String) -> String;
}

/// 初始化插件
#[plugin_fn]
pub fn init() -> FnResult<String> {
    info!("Echo 插件已初始化");
    
    // 订阅 "echo" 主题
    let subscribe_req = json!({
        "plugin_id": "echo",
        "topic": "echo_topic"
    });
    
    unsafe {
        subscribe_topic_host(subscribe_req.to_string())?;
        log_message_host("info".to_string(), "Echo plugin initialized and subscribed to echo_topic".to_string())?;
    }
    
    Ok("Echo plugin initialized".to_string())
}

/// 处理消息
#[plugin_fn]
pub fn process_message(input: String) -> FnResult<String> {
    let msg: EchoMessage = serde_json::from_str(&input)?;
    
    // 记录收到的消息
    info!("收到来自 {} 的消息: {}", msg.from, msg.content);
    
    unsafe {
        log_message_host(
            "info".to_string(),
            format!("Received message from {}: {}", msg.from, msg.content)
        )?;
    }
    
    // 准备回显消息
    let echo_content = format!("Echo: {}", msg.content);
    
    // 准备调用主机函数的参数
    let reply = json!({
        "from": "echo",
        "to": msg.from,
        "payload": echo_content
    });
    
    // 调用主机函数发送回复
    let msg_id = unsafe { 
        send_message_host(reply.to_string())?
    };
    
    Ok(format!("已回显消息给 {}，消息ID: {}", msg.from, msg_id))
}

/// 批量回显功能
#[plugin_fn]
pub fn echo_multiple(input: String) -> FnResult<String> {
    #[derive(Deserialize)]
    struct BatchRequest {
        messages: Vec<EchoMessage>,
    }
    
    let batch: BatchRequest = serde_json::from_str(&input)?;
    let mut results = Vec::new();
    
    for msg in batch.messages {
        // 处理每个消息
        let echo_content = format!("Echo: {}", msg.content);
        let reply = json!({
            "from": "echo",
            "to": msg.from,
            "payload": echo_content
        });
        
        let msg_id = unsafe { 
            send_message_host(reply.to_string())?
        };
        
        results.push(format!("Echoed to {}: {}", msg.from, msg_id));
    }
    
    Ok(json!({
        "status": "success",
        "count": results.len(),
        "results": results
    }).to_string())
}