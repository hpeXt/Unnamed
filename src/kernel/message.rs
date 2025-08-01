//! 消息类型定义
//! 
//! 定义插件间通信的消息格式

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// 插件间消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息ID（唯一标识）
    pub id: String,
    
    /// 发送者插件名称
    pub from: String,
    
    /// 接收者插件名称（对于主题消息，可能为空）
    pub to: String,
    
    /// 消息负载（二进制数据）
    pub payload: Vec<u8>,
    
    /// 消息类型（可选，用于路由）
    pub msg_type: Option<String>,
    
    /// 主题名称（用于发布-订阅模式）
    pub topic: Option<String>,
    
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// 创建新消息（点对点）
    pub fn new(from: String, to: String, payload: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            payload,
            msg_type: None,
            topic: None,
            timestamp: Utc::now(),
        }
    }
    
    /// 创建主题消息（发布-订阅）
    pub fn new_topic(from: String, topic: String, payload: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to: String::new(), // 主题消息不需要特定接收者
            payload,
            msg_type: None,
            topic: Some(topic),
            timestamp: Utc::now(),
        }
    }
    
    /// 设置消息类型
    pub fn with_type(mut self, msg_type: String) -> Self {
        self.msg_type = Some(msg_type);
        self
    }
    
    /// 设置主题
    pub fn with_topic(mut self, topic: String) -> Self {
        self.topic = Some(topic);
        self
    }
    
    /// 检查是否为主题消息
    pub fn is_topic_message(&self) -> bool {
        self.topic.is_some()
    }
}

/// 消息发送结果
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageResult {
    /// 成功发送
    Success,
    
    /// 目标插件不存在
    PluginNotFound(String),
    
    /// 发送失败
    Failed(String),
}