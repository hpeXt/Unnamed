//! 重新设计的消息总线系统
//!
//! 将消息系统分为两部分：
//! - MessageBusHandle: 可克隆的发送端
//! - MessageRouter: 独占的接收端

use anyhow::Result;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;

use super::message::{Message, MessageResult};

/// 消息总线句柄 - 可克隆，用于发送消息和管理插件通道
#[derive(Clone)]
pub struct MessageBusHandle {
    /// 主消息发送器
    sender: mpsc::Sender<Message>,
    /// 插件通道映射
    plugin_channels: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
    /// 主题订阅映射
    topic_subscriptions: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// 关闭信号发送器
    shutdown_tx: mpsc::Sender<()>,
}

/// 消息路由器 - 独占接收端，负责路由消息
pub struct MessageRouter {
    /// 主消息接收器
    receiver: mpsc::Receiver<Message>,
    /// 插件通道映射（与 Handle 共享）
    plugin_channels: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
    /// 主题订阅映射（与 Handle 共享）
    topic_subscriptions: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// 关闭信号接收器
    shutdown_rx: mpsc::Receiver<()>,
}

/// 创建消息总线系统
pub fn create_message_bus(buffer_size: usize) -> (MessageBusHandle, MessageRouter) {
    let (sender, receiver) = mpsc::channel(buffer_size);
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let plugin_channels = Arc::new(RwLock::new(HashMap::new()));
    let topic_subscriptions = Arc::new(RwLock::new(HashMap::new()));

    let handle = MessageBusHandle {
        sender,
        plugin_channels: plugin_channels.clone(),
        topic_subscriptions: topic_subscriptions.clone(),
        shutdown_tx,
    };

    let router = MessageRouter {
        receiver,
        plugin_channels,
        topic_subscriptions,
        shutdown_rx,
    };

    (handle, router)
}

impl MessageBusHandle {
    /// 获取主消息发送器
    pub fn get_sender(&self) -> mpsc::Sender<Message> {
        self.sender.clone()
    }

    /// 获取关闭信号发送器
    pub fn get_shutdown_sender(&self) -> mpsc::Sender<()> {
        self.shutdown_tx.clone()
    }

    /// 为插件注册通道
    pub fn register_plugin(&self, plugin_id: String) -> mpsc::Receiver<Message> {
        let (tx, rx) = mpsc::channel(100);
        self.plugin_channels.write().insert(plugin_id, tx);
        rx
    }

    /// 注销插件
    pub fn unregister_plugin(&self, plugin_id: &str) {
        self.plugin_channels.write().remove(plugin_id);

        // 从所有主题订阅中移除该插件
        let mut subscriptions = self.topic_subscriptions.write();
        for (_, subscribers) in subscriptions.iter_mut() {
            subscribers.remove(plugin_id);
        }
        // 清理空的主题
        subscriptions.retain(|_, subscribers| !subscribers.is_empty());
    }

    /// 订阅主题
    pub fn subscribe_topic(&self, plugin_id: &str, topic: &str) -> bool {
        let mut subscriptions = self.topic_subscriptions.write();
        let subscribers = subscriptions.entry(topic.to_string()).or_default();
        subscribers.insert(plugin_id.to_string())
    }

    /// 取消订阅主题
    pub fn unsubscribe_topic(&self, plugin_id: &str, topic: &str) -> bool {
        let mut subscriptions = self.topic_subscriptions.write();
        if let Some(subscribers) = subscriptions.get_mut(topic) {
            let removed = subscribers.remove(plugin_id);
            // 如果该主题没有订阅者了，就删除这个主题
            if subscribers.is_empty() {
                subscriptions.remove(topic);
            }
            removed
        } else {
            false
        }
    }

    /// 获取主题的订阅者列表
    pub fn get_topic_subscribers(&self, topic: &str) -> Vec<String> {
        let subscriptions = self.topic_subscriptions.read();
        subscriptions
            .get(topic)
            .map(|subscribers| subscribers.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// 发送消息到消息总线
    pub async fn send_message(&self, message: Message) -> Result<()> {
        self.sender
            .send(message)
            .await
            .map_err(|_| anyhow::anyhow!("消息总线已关闭"))?;
        Ok(())
    }

    /// 发送关闭信号
    pub async fn shutdown(&self) -> Result<()> {
        let _ = self.shutdown_tx.send(()).await;
        Ok(())
    }
}

impl MessageRouter {
    /// 运行消息路由（消耗 self）
    pub async fn run(mut self) {
        tracing::info!("消息路由器开始运行");

        loop {
            tokio::select! {
                // 监听消息
                msg = self.receiver.recv() => {
                    match msg {
                        Some(message) => {
                            if message.is_topic_message() {
                                let topic = message.topic.as_ref().unwrap();
                                tracing::debug!("收到主题消息: from={}, topic={}", message.from, topic);
                            } else {
                                tracing::debug!("收到点对点消息: from={}, to={}", message.from, message.to);
                            }

                            // 路由消息
                            let result = if message.is_topic_message() {
                                self.route_topic_message(message).await
                            } else {
                                self.route_direct_message(message).await
                            };

                            match result {
                                MessageResult::Success => {
                                    tracing::trace!("消息路由成功");
                                }
                                MessageResult::PluginNotFound(ref target) => {
                                    tracing::warn!("目标不存在: {}", target);
                                }
                                MessageResult::Failed(ref reason) => {
                                    tracing::error!("消息路由失败: {}", reason);
                                }
                            }
                        }
                        None => {
                            tracing::info!("消息通道已关闭");
                            break;
                        }
                    }
                }
                // 监听关闭信号
                _ = self.shutdown_rx.recv() => {
                    tracing::info!("收到关闭信号，停止消息路由");
                    break;
                }
            }
        }

        tracing::info!("消息路由器已停止");
    }

    /// 路由点对点消息
    async fn route_direct_message(&self, message: Message) -> MessageResult {
        // 在 await 之前获取发送器的克隆，避免跨 await 持有锁
        let tx_opt = {
            let channels = self.plugin_channels.read();
            channels.get(&message.to).cloned()
        };

        if let Some(tx) = tx_opt {
            match tx.send(message).await {
                Ok(_) => MessageResult::Success,
                Err(_) => MessageResult::Failed("通道已关闭".to_string()),
            }
        } else {
            MessageResult::PluginNotFound(message.to.clone())
        }
    }

    /// 路由主题消息
    async fn route_topic_message(&self, message: Message) -> MessageResult {
        let topic = message.topic.as_ref().expect("主题消息必须有topic字段");

        // 获取订阅者列表
        let subscribers = {
            let subscriptions = self.topic_subscriptions.read();
            subscriptions
                .get(topic)
                .map(|subs| subs.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        };

        if subscribers.is_empty() {
            return MessageResult::PluginNotFound(format!("主题 '{topic}' 没有订阅者"));
        }

        // 在 await 之前收集所有需要的发送器
        let senders: Vec<_> = {
            let channels = self.plugin_channels.read();
            subscribers
                .iter()
                .filter_map(|subscriber| {
                    channels
                        .get(subscriber)
                        .map(|tx| (subscriber.clone(), tx.clone()))
                })
                .collect()
        };

        let mut successful_sends = 0;
        let mut failed_sends = 0;

        // 发送消息给所有订阅者
        for (_subscriber, tx) in senders {
            match tx.send(message.clone()).await {
                Ok(_) => successful_sends += 1,
                Err(_) => failed_sends += 1,
            }
        }

        // 统计没有找到通道的订阅者
        let total_attempted = successful_sends + failed_sends;
        let missing_channels = subscribers.len().saturating_sub(total_attempted);
        failed_sends += missing_channels;

        if successful_sends > 0 {
            MessageResult::Success
        } else {
            MessageResult::Failed(format!("所有订阅者都发送失败 ({failed_sends})"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_message_bus() {
        let (handle, _router) = create_message_bus(100);

        // 测试克隆
        let handle2 = handle.clone();
        assert_eq!(
            handle.plugin_channels.read().len(),
            handle2.plugin_channels.read().len()
        );
    }

    #[tokio::test]
    async fn test_plugin_registration() {
        let (handle, _router) = create_message_bus(100);

        // 注册插件
        let _rx = handle.register_plugin("test_plugin".to_string());
        assert_eq!(handle.plugin_channels.read().len(), 1);

        // 注销插件
        handle.unregister_plugin("test_plugin");
        assert_eq!(handle.plugin_channels.read().len(), 0);
    }

    #[tokio::test]
    async fn test_topic_subscription() {
        let (handle, _router) = create_message_bus(100);

        // 订阅主题
        assert!(handle.subscribe_topic("plugin1", "topic1"));
        assert!(handle.subscribe_topic("plugin2", "topic1"));

        // 获取订阅者
        let subscribers = handle.get_topic_subscribers("topic1");
        assert_eq!(subscribers.len(), 2);

        // 取消订阅
        assert!(handle.unsubscribe_topic("plugin1", "topic1"));
        let subscribers = handle.get_topic_subscribers("topic1");
        assert_eq!(subscribers.len(), 1);
    }
}
