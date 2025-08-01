//! 消息传递集成测试

use minimal_kernel::kernel::{message::Message, message_bus::create_message_bus};
use tokio::time::Duration;

#[tokio::test]
async fn test_plugin_message_passing() {
    // 创建消息总线
    let (handle, router) = create_message_bus(100);
    let sender = handle.get_sender();

    // 注册两个插件
    let mut hello_rx = handle.register_plugin("hello".to_string());
    let mut echo_rx = handle.register_plugin("echo".to_string());

    // 启动消息路由
    tokio::spawn(async move {
        router.run().await;
    });

    // 从 hello 发送消息到 echo
    let msg = Message::new(
        "hello".to_string(),
        "echo".to_string(),
        b"Hello Echo!".to_vec(),
    );

    sender.send(msg.clone()).await.unwrap();

    // 验证 echo 收到消息
    let received = tokio::time::timeout(Duration::from_secs(1), echo_rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(received.from, "hello");
    assert_eq!(received.to, "echo");
    assert_eq!(received.payload, b"Hello Echo!");

    // 从 echo 发送回复到 hello
    let reply = Message::new(
        "echo".to_string(),
        "hello".to_string(),
        b"Echo: Hello Echo!".to_vec(),
    );

    sender.send(reply).await.unwrap();

    // 验证 hello 收到回复
    let received_reply = tokio::time::timeout(Duration::from_secs(1), hello_rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(received_reply.from, "echo");
    assert_eq!(received_reply.to, "hello");
    assert_eq!(received_reply.payload, b"Echo: Hello Echo!");
}

#[tokio::test]
async fn test_message_to_nonexistent_plugin() {
    let (handle, router) = create_message_bus(100);

    // 启动消息路由
    tokio::spawn(async move {
        router.run().await;
    });

    // 等待路由器启动
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 发送消息到不存在的插件
    let result = handle
        .send_message(Message::new(
            "test".to_string(),
            "nonexistent".to_string(),
            b"test".to_vec(),
        ))
        .await;

    // 新架构中 send_message 返回 Result，不是 MessageResult
    // 消息发送到路由器后就认为成功，即使目标不存在
    assert!(result.is_ok());
}
