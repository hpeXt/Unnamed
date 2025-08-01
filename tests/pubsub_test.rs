//! 发布-订阅功能测试
//!
//! 测试主题订阅、发布和取消订阅功能

use minimal_kernel::kernel::{message::Message, message_bus::create_message_bus};
use tokio::time::{sleep, timeout, Duration};

#[tokio::test]
async fn test_topic_subscription() {
    // 创建消息总线
    let (handle, router) = create_message_bus(100);

    // 注册两个插件
    let mut plugin1_rx = handle.register_plugin("plugin1".to_string());
    let mut plugin2_rx = handle.register_plugin("plugin2".to_string());

    // 两个插件都订阅同一个主题
    let topic = "test_topic";
    assert!(handle.subscribe_topic("plugin1", topic));
    assert!(handle.subscribe_topic("plugin2", topic));

    // 获取发送器
    let sender = handle.get_sender();

    // 启动消息路由
    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 创建主题消息
    let message = Message::new_topic(
        "publisher".to_string(),
        topic.to_string(),
        b"Hello topic subscribers!".to_vec(),
    );

    // 通过消息总线发送
    sender.send(message).await.unwrap();

    // 等待消息传播
    sleep(Duration::from_millis(100)).await;

    // 验证两个插件都收到了消息
    let msg1 = timeout(Duration::from_secs(1), plugin1_rx.recv())
        .await
        .expect("插件1应该收到消息")
        .expect("消息不应该为空");

    let msg2 = timeout(Duration::from_secs(1), plugin2_rx.recv())
        .await
        .expect("插件2应该收到消息")
        .expect("消息不应该为空");

    // 验证消息内容
    assert_eq!(msg1.from, "publisher");
    assert_eq!(msg1.topic, Some(topic.to_string()));
    assert_eq!(msg1.payload, b"Hello topic subscribers!");

    assert_eq!(msg2.from, "publisher");
    assert_eq!(msg2.topic, Some(topic.to_string()));
    assert_eq!(msg2.payload, b"Hello topic subscribers!");

    // 清理
    router_handle.abort();
}

#[tokio::test]
async fn test_topic_unsubscription() {
    let (handle, router) = create_message_bus(100);
    let mut plugin1_rx = handle.register_plugin("plugin1".to_string());
    let mut plugin2_rx = handle.register_plugin("plugin2".to_string());

    let topic = "test_topic";

    // 两个插件都订阅主题
    assert!(handle.subscribe_topic("plugin1", topic));
    assert!(handle.subscribe_topic("plugin2", topic));

    // plugin1 取消订阅
    assert!(handle.unsubscribe_topic("plugin1", topic));

    // 获取发送器
    let sender = handle.get_sender();

    // 启动消息路由
    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 发送主题消息
    let message = Message::new_topic(
        "publisher".to_string(),
        topic.to_string(),
        b"Only plugin2 should receive this".to_vec(),
    );

    sender.send(message).await.unwrap();

    // 等待消息传播
    sleep(Duration::from_millis(100)).await;

    // 验证只有 plugin2 收到消息
    let msg2 = timeout(Duration::from_secs(1), plugin2_rx.recv())
        .await
        .expect("插件2应该收到消息")
        .expect("消息不应该为空");

    assert_eq!(msg2.from, "publisher");
    assert_eq!(msg2.topic, Some(topic.to_string()));

    // 验证 plugin1 没有收到消息
    let result = timeout(Duration::from_millis(500), plugin1_rx.recv()).await;
    assert!(result.is_err(), "插件1不应该收到消息");

    router_handle.abort();
}

#[tokio::test]
async fn test_empty_topic_subscribers() {
    let (handle, router) = create_message_bus(100);

    // 获取发送器
    let sender = handle.get_sender();

    // 启动消息路由
    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 发送到没有订阅者的主题
    let message = Message::new_topic(
        "publisher".to_string(),
        "empty_topic".to_string(),
        b"No one should receive this".to_vec(),
    );

    // 这不应该导致错误，只是没有接收者
    let result = sender.send(message).await;
    assert!(result.is_ok());

    router_handle.abort();
}

#[tokio::test]
async fn test_multiple_topics() {
    let (handle, router) = create_message_bus(100);
    let mut plugin1_rx = handle.register_plugin("plugin1".to_string());
    let mut plugin2_rx = handle.register_plugin("plugin2".to_string());

    // plugin1 订阅 topic1，plugin2 订阅 topic2
    assert!(handle.subscribe_topic("plugin1", "topic1"));
    assert!(handle.subscribe_topic("plugin2", "topic2"));

    // 获取发送器
    let sender = handle.get_sender();

    // 启动消息路由
    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 发送到 topic1
    let msg1 = Message::new_topic(
        "publisher".to_string(),
        "topic1".to_string(),
        b"Message for topic1".to_vec(),
    );
    sender.send(msg1).await.unwrap();

    // 发送到 topic2
    let msg2 = Message::new_topic(
        "publisher".to_string(),
        "topic2".to_string(),
        b"Message for topic2".to_vec(),
    );
    sender.send(msg2).await.unwrap();

    // 等待消息传播
    sleep(Duration::from_millis(100)).await;

    // 验证 plugin1 只收到 topic1 的消息
    let received1 = timeout(Duration::from_secs(1), plugin1_rx.recv())
        .await
        .expect("插件1应该收到消息")
        .expect("消息不应该为空");

    assert_eq!(received1.topic, Some("topic1".to_string()));
    assert_eq!(received1.payload, b"Message for topic1");

    // 验证 plugin2 只收到 topic2 的消息
    let received2 = timeout(Duration::from_secs(1), plugin2_rx.recv())
        .await
        .expect("插件2应该收到消息")
        .expect("消息不应该为空");

    assert_eq!(received2.topic, Some("topic2".to_string()));
    assert_eq!(received2.payload, b"Message for topic2");

    router_handle.abort();
}

#[tokio::test]
async fn test_get_topic_subscribers() {
    let (handle, _router) = create_message_bus(100);
    let topic = "test_topic";

    // 初始时没有订阅者
    assert_eq!(handle.get_topic_subscribers(topic).len(), 0);

    // 添加订阅者
    assert!(handle.subscribe_topic("plugin1", topic));
    assert!(handle.subscribe_topic("plugin2", topic));
    assert!(handle.subscribe_topic("plugin3", topic));

    let subscribers = handle.get_topic_subscribers(topic);
    assert_eq!(subscribers.len(), 3);
    assert!(subscribers.contains(&"plugin1".to_string()));
    assert!(subscribers.contains(&"plugin2".to_string()));
    assert!(subscribers.contains(&"plugin3".to_string()));

    // 移除一个订阅者
    assert!(handle.unsubscribe_topic("plugin2", topic));

    let subscribers_after = handle.get_topic_subscribers(topic);
    assert_eq!(subscribers_after.len(), 2);
    assert!(subscribers_after.contains(&"plugin1".to_string()));
    assert!(subscribers_after.contains(&"plugin3".to_string()));
    assert!(!subscribers_after.contains(&"plugin2".to_string()));
}
