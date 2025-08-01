//! 插件消息处理
//!
//! 提供插件间通信的消息类型和处理机制

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 插件消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    /// 消息ID
    pub id: String,
    /// 发送者插件名称
    pub from: String,
    /// 接收者插件名称
    pub to: String,
    /// 消息主题
    pub topic: String,
    /// 消息负载
    pub payload: Vec<u8>,
    /// 消息类型
    pub message_type: String,
    /// 消息元数据
    pub metadata: HashMap<String, String>,
    /// 消息时间戳（毫秒）
    pub timestamp: u64,
    /// 消息过期时间戳（毫秒）
    pub expires_at: Option<u64>,
    /// 消息优先级
    pub priority: MessagePriority,
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// 低优先级
    Low = 0,
    /// 正常优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 紧急优先级
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

/// 消息构建器
#[derive(Debug)]
pub struct MessageBuilder {
    from: String,
    to: Option<String>,
    topic: Option<String>,
    payload: Option<Vec<u8>>,
    message_type: Option<String>,
    metadata: HashMap<String, String>,
    priority: MessagePriority,
    expires_at: Option<u64>,
}

impl MessageBuilder {
    /// 创建新的消息构建器
    pub fn new(from: &str) -> Self {
        Self {
            from: from.to_string(),
            to: None,
            topic: None,
            payload: None,
            message_type: None,
            metadata: HashMap::new(),
            priority: MessagePriority::Normal,
            expires_at: None,
        }
    }

    /// 设置接收者
    pub fn to(mut self, to: &str) -> Self {
        self.to = Some(to.to_string());
        self
    }

    /// 设置主题
    pub fn topic(mut self, topic: &str) -> Self {
        self.topic = Some(topic.to_string());
        self
    }

    /// 设置负载（JSON）
    pub fn payload_json<T: Serialize>(mut self, payload: &T) -> Result<Self, serde_json::Error> {
        self.payload = Some(serde_json::to_vec(payload)?);
        self.message_type = Some("application/json".to_string());
        Ok(self)
    }

    /// 设置负载（字符串）
    pub fn payload_string(mut self, payload: &str) -> Self {
        self.payload = Some(payload.as_bytes().to_vec());
        self.message_type = Some("text/plain".to_string());
        self
    }

    /// 设置负载（字节）
    pub fn payload_bytes(mut self, payload: Vec<u8>) -> Self {
        self.payload = Some(payload);
        self.message_type = Some("application/octet-stream".to_string());
        self
    }

    /// 设置消息类型
    pub fn message_type(mut self, message_type: &str) -> Self {
        self.message_type = Some(message_type.to_string());
        self
    }

    /// 添加元数据
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置优先级
    pub fn priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置过期时间戳（毫秒）
    pub fn expires_at(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// 设置生存时间（秒）
    pub fn ttl(mut self, seconds: u64) -> Self {
        let current_time = crate::utils::time::now_millis();
        self.expires_at = Some(current_time + (seconds * 1000));
        self
    }

    /// 构建消息
    pub fn build(self) -> Result<PluginMessage, String> {
        let to = self.to.ok_or("Missing 'to' field")?;
        let topic = self.topic.unwrap_or_else(|| "default".to_string());
        let payload = self.payload.unwrap_or_default();
        let message_type = self
            .message_type
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Ok(PluginMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: self.from,
            to,
            topic,
            payload,
            message_type,
            metadata: self.metadata,
            timestamp: crate::utils::time::now_millis(),
            expires_at: self.expires_at,
            priority: self.priority,
        })
    }
}

impl PluginMessage {
    /// 创建新的消息构建器
    pub fn builder(from: &str) -> MessageBuilder {
        MessageBuilder::new(from)
    }

    /// 检查消息是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            crate::utils::time::now_millis() > expires_at
        } else {
            false
        }
    }

    /// 获取负载为 JSON
    pub fn payload_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.payload)
    }

    /// 获取负载为字符串
    pub fn payload_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.payload.clone())
    }

    /// 获取负载为字节
    pub fn payload_bytes(&self) -> &[u8] {
        &self.payload
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 创建回复消息
    pub fn reply(&self, from: &str) -> MessageBuilder {
        MessageBuilder::new(from)
            .to(&self.from)
            .topic(&self.topic)
            .metadata("reply_to", &self.id)
    }

    /// 转发消息
    pub fn forward(&self, from: &str, to: &str) -> MessageBuilder {
        MessageBuilder::new(from)
            .to(to)
            .topic(&self.topic)
            .payload_bytes(self.payload.clone())
            .message_type(&self.message_type)
            .priority(self.priority)
            .metadata("forwarded_from", &self.from)
            .metadata("original_id", &self.id)
    }
}

/// 消息处理器 trait
pub trait MessageHandler {
    /// 处理消息
    fn handle_message(&mut self, message: &PluginMessage) -> crate::error::PluginResult<()>;

    /// 获取支持的消息类型
    fn supported_message_types(&self) -> Vec<String> {
        vec!["*".to_string()]
    }

    /// 获取支持的主题
    fn supported_topics(&self) -> Vec<String> {
        vec!["*".to_string()]
    }
}

/// 消息过滤器
#[derive(Debug, Clone)]
pub struct MessageFilter {
    /// 发送者过滤器
    pub from: Option<String>,
    /// 接收者过滤器
    pub to: Option<String>,
    /// 主题过滤器
    pub topic: Option<String>,
    /// 消息类型过滤器
    pub message_type: Option<String>,
    /// 优先级过滤器
    pub min_priority: Option<MessagePriority>,
}

impl MessageFilter {
    /// 创建新的过滤器
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            topic: None,
            message_type: None,
            min_priority: None,
        }
    }

    /// 设置发送者过滤器
    pub fn from(mut self, from: &str) -> Self {
        self.from = Some(from.to_string());
        self
    }

    /// 设置接收者过滤器
    pub fn to(mut self, to: &str) -> Self {
        self.to = Some(to.to_string());
        self
    }

    /// 设置主题过滤器
    pub fn topic(mut self, topic: &str) -> Self {
        self.topic = Some(topic.to_string());
        self
    }

    /// 设置消息类型过滤器
    pub fn message_type(mut self, message_type: &str) -> Self {
        self.message_type = Some(message_type.to_string());
        self
    }

    /// 设置最小优先级过滤器
    pub fn min_priority(mut self, priority: MessagePriority) -> Self {
        self.min_priority = Some(priority);
        self
    }

    /// 检查消息是否匹配过滤器
    pub fn matches(&self, message: &PluginMessage) -> bool {
        if let Some(ref from) = self.from {
            if &message.from != from {
                return false;
            }
        }

        if let Some(ref to) = self.to {
            if &message.to != to {
                return false;
            }
        }

        if let Some(ref topic) = self.topic {
            if &message.topic != topic {
                return false;
            }
        }

        if let Some(ref message_type) = self.message_type {
            if &message.message_type != message_type {
                return false;
            }
        }

        if let Some(min_priority) = self.min_priority {
            if message.priority < min_priority {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_builder() {
        let message = PluginMessage::builder("sender")
            .to("receiver")
            .topic("test")
            .payload_string("Hello, World!")
            .priority(MessagePriority::High)
            .build()
            .unwrap();

        assert_eq!(message.from, "sender");
        assert_eq!(message.to, "receiver");
        assert_eq!(message.topic, "test");
        assert_eq!(message.priority, MessagePriority::High);
        assert_eq!(message.payload_string().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_message_filter() {
        let message = PluginMessage::builder("sender")
            .to("receiver")
            .topic("test")
            .payload_string("test")
            .priority(MessagePriority::High)
            .build()
            .unwrap();

        let filter = MessageFilter::new()
            .from("sender")
            .topic("test")
            .min_priority(MessagePriority::Normal);

        assert!(filter.matches(&message));

        let filter = MessageFilter::new().from("other").topic("test");

        assert!(!filter.matches(&message));
    }

    #[test]
    fn test_message_expiration() {
        let mut message = PluginMessage::builder("sender")
            .to("receiver")
            .topic("test")
            .payload_string("test")
            .build()
            .unwrap();

        assert!(!message.is_expired());

        message.expires_at = Some(crate::utils::time::now_millis() - 1000);
        assert!(message.is_expired());
    }
}
