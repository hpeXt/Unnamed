//! 主机函数定义
//! 
//! 提供给插件调用的函数

use extism::*;
use tokio::sync::mpsc;
use crate::kernel::message::Message;
use crate::kernel::message_bus::MessageBusHandle;
use crate::storage::Storage;
use crate::identity::IdentityManager;
use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;

/// 共享应用状态
#[derive(Clone)]
pub struct HostContext {
    pub storage: Option<Arc<Storage>>,
    pub msg_sender: mpsc::Sender<Message>,
    pub identity: Option<Arc<IdentityManager>>,
    pub message_bus: Option<MessageBusHandle>,
}

impl HostContext {
    pub fn new(
        storage: Option<Arc<Storage>>, 
        msg_sender: mpsc::Sender<Message>,
        identity: Option<Arc<IdentityManager>>,
        message_bus: Option<MessageBusHandle>
    ) -> Self {
        Self {
            storage,
            msg_sender,
            identity,
            message_bus,
        }
    }
}

// 使用 BTreeMap 来包装上下文（官方推荐模式）
pub type ContextStore = Arc<Mutex<BTreeMap<String, Arc<Mutex<HostContext>>>>>;

// 定义主机函数（基于官方文档的 KV store 示例）
host_fn!(store_data(user_data: ContextStore; plugin_id: String, key: String, value: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(storage) = &ctx.storage {
            // 解析 JSON 值
            let json_value: serde_json::Value = serde_json::from_str(&value)?;
            
            // 使用 block_on 在同步上下文中执行异步操作
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                storage.store_data(&plugin_id, &key, &json_value).await
            })?;
            
            Ok("success".to_string())
        } else {
            Err(extism::Error::msg("Storage not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(get_data(user_data: ContextStore; plugin_id: String, key: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(storage) = &ctx.storage {
            let runtime = tokio::runtime::Handle::current();
            let value = runtime.block_on(async {
                storage.get_data(&plugin_id, &key).await
            })?;
            
            // 将结果序列化为 JSON
            let result = serde_json::json!({
                "success": true,
                "value": value
            });
            
            Ok(result.to_string())
        } else {
            Err(extism::Error::msg("Storage not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(delete_data(user_data: ContextStore; plugin_id: String, key: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(storage) = &ctx.storage {
            let runtime = tokio::runtime::Handle::current();
            let deleted = runtime.block_on(async {
                storage.delete_data(&plugin_id, &key).await
            })?;
            
            // 将结果序列化为 JSON
            let result = serde_json::json!({
                "success": true,
                "deleted": deleted
            });
            
            Ok(result.to_string())
        } else {
            Err(extism::Error::msg("Storage not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(list_keys(user_data: ContextStore; plugin_id: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(storage) = &ctx.storage {
            let runtime = tokio::runtime::Handle::current();
            let keys = runtime.block_on(async {
                storage.list_keys(&plugin_id).await
            })?;
            
            // 将结果序列化为 JSON
            let result = serde_json::json!({
                "success": true,
                "keys": keys
            });
            
            Ok(result.to_string())
        } else {
            Err(extism::Error::msg("Storage not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(send_message(user_data: ContextStore; from: String, to: String, payload: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        
        // 将 payload 转换为字节
        let payload_bytes = payload.into_bytes();
        let msg = Message::new(from, to, payload_bytes);
        let msg_id = msg.id.clone();
        
        ctx.msg_sender.try_send(msg)
            .map_err(|e| extism::Error::msg(format!("Failed to send message: {}", e)))?;
        
        Ok(msg_id)
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

// 简单的日志函数（不需要用户数据）
host_fn!(log_message(level: String, message: String) -> String {
    match level.as_str() {
        "error" => eprintln!("[PLUGIN ERROR] {}", message),
        "warn" => eprintln!("[PLUGIN WARN] {}", message),
        "info" => println!("[PLUGIN INFO] {}", message),
        "debug" => println!("[PLUGIN DEBUG] {}", message),
        _ => println!("[PLUGIN] {}", message),
    }
    
    Ok("logged".to_string())
});

// 身份管理相关主机函数
host_fn!(sign_message(user_data: ContextStore; plugin_id: String, message: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(identity) = &ctx.identity {
            let runtime = tokio::runtime::Handle::current();
            let signature = runtime.block_on(async {
                identity.sign_for_plugin(&plugin_id, message.as_bytes()).await
            })?;
            
            // 将签名转换为十六进制字符串
            let signature_hex = hex::encode(&signature);
            
            let result = serde_json::json!({
                "success": true,
                "signature": signature_hex
            });
            
            Ok(result.to_string())
        } else {
            Err(extism::Error::msg("Identity manager not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(verify_signature(user_data: ContextStore; plugin_id: String, message: String, signature: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(identity) = &ctx.identity {
            // 将十六进制签名转换为字节
            let signature_bytes = hex::decode(&signature)
                .map_err(|e| extism::Error::msg(format!("Invalid signature hex: {}", e)))?;
            
            let runtime = tokio::runtime::Handle::current();
            let is_valid = runtime.block_on(async {
                identity.verify_plugin_signature(&plugin_id, message.as_bytes(), &signature_bytes).await
            })?;
            
            let result = serde_json::json!({
                "success": true,
                "valid": is_valid
            });
            
            Ok(result.to_string())
        } else {
            Err(extism::Error::msg("Identity manager not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(get_plugin_address(user_data: ContextStore; plugin_id: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(identity) = &ctx.identity {
            let runtime = tokio::runtime::Handle::current();
            let address = runtime.block_on(async {
                identity.get_plugin_address(&plugin_id).await
            })?;
            
            let result = serde_json::json!({
                "success": true,
                "address": address.to_string()
            });
            
            Ok(result.to_string())
        } else {
            Err(extism::Error::msg("Identity manager not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(subscribe_topic(user_data: ContextStore; plugin_id: String, topic: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(bus) = &ctx.message_bus {
            let success = bus.subscribe_topic(&plugin_id, &topic);
            
            let result = serde_json::json!({
                "success": success,
                "plugin_id": plugin_id,
                "topic": topic,
                "message": if success {
                    "订阅成功"
                } else {
                    "订阅失败，可能已经订阅过此主题"
                }
            });
            
            Ok(result.to_string())
        } else {
            Err(extism::Error::msg("Message bus not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(unsubscribe_topic(user_data: ContextStore; plugin_id: String, topic: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        if let Some(bus) = &ctx.message_bus {
            let success = bus.unsubscribe_topic(&plugin_id, &topic);
            
            let result = serde_json::json!({
                "success": success,
                "plugin_id": plugin_id,
                "topic": topic,
                "message": if success {
                    "取消订阅成功"
                } else {
                    "取消订阅失败，可能未订阅此主题"
                }
            });
            
            Ok(result.to_string())
        } else {
            Err(extism::Error::msg("Message bus not initialized"))
        }
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

host_fn!(publish_message(user_data: ContextStore; plugin_id: String, topic: String, payload: String) -> String {
    let store = user_data.get()?;
    let store = store.lock().unwrap();
    let inner_store = store.lock().unwrap();
    
    if let Some(ctx_arc) = inner_store.get("context") {
        let ctx = ctx_arc.lock().unwrap();
        
        // 创建主题消息
        let payload_bytes = payload.into_bytes();
        let msg = Message::new_topic(plugin_id.clone(), topic.clone(), payload_bytes);
        let msg_id = msg.id.clone();
        
        // 发送消息
        ctx.msg_sender.try_send(msg)
            .map_err(|e| extism::Error::msg(format!("Failed to send topic message: {}", e)))?;
        
        let result = serde_json::json!({
            "success": true,
            "message_id": msg_id,
            "topic": topic,
            "from": plugin_id
        });
        
        Ok(result.to_string())
    } else {
        Err(extism::Error::msg("Context not found"))
    }
});

// 时间相关主机函数 - 不需要用户数据
host_fn!(get_timestamp_host() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| extism::Error::msg(format!("Time error: {}", e)))?
        .as_secs();
    
    Ok(timestamp.to_string())
});

host_fn!(get_timestamp_millis_host() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| extism::Error::msg(format!("Time error: {}", e)))?
        .as_millis() as u64;
    
    Ok(timestamp.to_string())
});

/// 为 PluginBuilder 创建上下文存储
pub fn create_context_store(context: Arc<Mutex<HostContext>>) -> UserData<ContextStore> {
    let mut store = BTreeMap::new();
    store.insert("context".to_string(), context);
    UserData::new(Arc::new(Mutex::new(store)))
}

/// 使用 PluginBuilder 创建带有主机函数的插件
pub fn build_plugin_with_host_functions(
    manifest: Manifest,
    context_store: UserData<ContextStore>,
) -> Result<Plugin, extism::Error> {
    PluginBuilder::new(manifest)
        .with_wasi(true)
        .with_function(
            "store_data_host",
            [PTR],
            [PTR],
            context_store.clone(),
            store_data,
        )
        .with_function(
            "get_data_host",
            [PTR],
            [PTR],
            context_store.clone(),
            get_data,
        )
        .with_function(
            "delete_data_host",
            [PTR],
            [PTR],
            context_store.clone(),
            delete_data,
        )
        .with_function(
            "list_keys_host",
            [PTR],
            [PTR],
            context_store.clone(),
            list_keys,
        )
        .with_function(
            "send_message_host",
            [PTR],
            [PTR],
            context_store.clone(),
            send_message,
        )
        .with_function(
            "log_message_host",
            [PTR],
            [PTR],
            UserData::new(()),
            log_message,
        )
        .with_function(
            "sign_message_host",
            [PTR],
            [PTR],
            context_store.clone(),
            sign_message,
        )
        .with_function(
            "verify_signature_host",
            [PTR],
            [PTR],
            context_store.clone(),
            verify_signature,
        )
        .with_function(
            "get_plugin_address_host",
            [PTR],
            [PTR],
            context_store.clone(),
            get_plugin_address,
        )
        .with_function(
            "subscribe_topic_host",
            [PTR],
            [PTR],
            context_store.clone(),
            subscribe_topic,
        )
        .with_function(
            "unsubscribe_topic_host",
            [PTR],
            [PTR],
            context_store.clone(),
            unsubscribe_topic,
        )
        .with_function(
            "publish_message_host",
            [PTR],
            [PTR],
            context_store.clone(),
            publish_message,
        )
        .with_function(
            "get_timestamp_host",
            [],
            [PTR],
            UserData::new(()),
            get_timestamp_host,
        )
        .with_function(
            "get_timestamp_millis_host",
            [],
            [PTR],
            UserData::new(()),
            get_timestamp_millis_host,
        )
        .build()
}

