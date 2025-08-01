//! 端到端消息传递测试
//!
//! 测试插件之间的消息传递功能

use minimal_kernel::kernel::{
    message_bus::{create_message_bus, MessageBusHandle, MessageRouter},
    plugin_loader::PluginLoader,
};
use minimal_kernel::storage::Storage;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;
use tokio::time::{sleep, timeout, Duration};

/// 测试辅助函数：编译插件
async fn build_test_plugins() -> anyhow::Result<()> {
    println!("构建测试插件...");

    // 构建 test_sender
    let output = tokio::process::Command::new("cargo")
        .current_dir("plugins/test_sender")
        .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to build test_sender: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // 构建 test_receiver
    let output = tokio::process::Command::new("cargo")
        .current_dir("plugins/test_receiver")
        .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to build test_receiver: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("测试插件构建完成");
    Ok(())
}

/// 创建测试环境
async fn setup_test_env() -> anyhow::Result<(
    PluginLoader,
    MessageBusHandle,
    MessageRouter,
    Arc<Storage>,
    TempDir,
)> {
    // 创建临时目录用于存储
    let temp_dir = TempDir::new()?;

    // 创建存储 - 使用内存数据库避免文件权限问题
    let storage = Arc::new(Storage::new("sqlite::memory:").await?);

    // 创建消息总线
    let (handle, router) = create_message_bus(100);
    let msg_sender = handle.get_sender();

    // 创建插件加载器
    let loader = PluginLoader::new(msg_sender.clone(), storage.clone(), None)?;

    Ok((loader, handle, router, storage, temp_dir))
}

#[tokio::test]
async fn test_basic_message_passing() -> anyhow::Result<()> {
    // 构建插件
    build_test_plugins().await?;

    // 设置测试环境
    let (mut loader, handle, router, _storage, _temp_dir) = setup_test_env().await?;

    // 注册插件到消息总线
    let mut sender_rx = handle.register_plugin("test_sender".to_string());
    let mut receiver_rx = handle.register_plugin("test_receiver".to_string());

    // 启动消息路由
    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 加载插件
    loader.load_plugin(
        "test_sender",
        "plugins/test_sender/target/wasm32-unknown-unknown/release/test_sender.wasm",
    )?;
    loader.load_plugin(
        "test_receiver",
        "plugins/test_receiver/target/wasm32-unknown-unknown/release/test_receiver.wasm",
    )?;

    // 初始化插件
    loader.call_plugin::<(), String>("test_sender", "init", ())?;
    loader.call_plugin::<(), String>("test_receiver", "init", ())?;

    // 测试发送单个消息
    let test_id = "basic_test_001";
    let send_request = json!({
        "to": "test_receiver",
        "message": "Hello from E2E test!",
        "test_id": test_id
    });

    let send_result: String =
        loader.call_plugin("test_sender", "send_test_message", send_request.to_string())?;

    let send_response: serde_json::Value = serde_json::from_str(&send_result)?;
    assert!(send_response["success"].as_bool().unwrap());
    let _message_id = send_response["message_id"].as_str().unwrap();

    // 等待消息被接收和处理
    sleep(Duration::from_millis(100)).await;

    // 让接收器处理消息
    if let Ok(Some(msg)) = timeout(Duration::from_secs(1), receiver_rx.recv()).await {
        let process_result: String = loader.call_plugin(
            "test_receiver",
            "process_message",
            serde_json::to_string(&msg)?,
        )?;

        let process_response: serde_json::Value = serde_json::from_str(&process_result)?;
        assert!(process_response["success"].as_bool().unwrap());
        assert_eq!(process_response["test_id"].as_str().unwrap(), test_id);
    } else {
        panic!("接收器未收到消息");
    }

    // 验证接收器的统计信息
    let stats_result: String = loader.call_plugin("test_receiver", "get_stats", ())?;
    let stats: serde_json::Value = serde_json::from_str(&stats_result)?;
    assert_eq!(stats["total_messages_received"].as_i64().unwrap(), 1);

    // 检查是否收到回复
    if let Ok(Some(reply)) = timeout(Duration::from_secs(1), sender_rx.recv()).await {
        println!("发送器收到回复: {:?}", reply);
        assert_eq!(reply.from, "test_receiver");
        assert_eq!(reply.to, "test_sender");
    }

    // 清理
    drop(router_handle);
    Ok(())
}

#[tokio::test]
async fn test_batch_message_sending() -> anyhow::Result<()> {
    build_test_plugins().await?;

    let (mut loader, handle, router, _storage, _temp_dir) = setup_test_env().await?;

    let _sender_rx = handle.register_plugin("test_sender".to_string());
    let mut receiver_rx = handle.register_plugin("test_receiver".to_string());

    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 加载插件
    loader.load_plugin(
        "test_sender",
        "plugins/test_sender/target/wasm32-unknown-unknown/release/test_sender.wasm",
    )?;
    loader.load_plugin(
        "test_receiver",
        "plugins/test_receiver/target/wasm32-unknown-unknown/release/test_receiver.wasm",
    )?;

    // 初始化插件
    loader.call_plugin::<(), String>("test_sender", "init", ())?;
    loader.call_plugin::<(), String>("test_receiver", "init", ())?;

    // 发送批量消息
    let test_id = "batch_test_001";
    let batch_request = json!({
        "to": "test_receiver",
        "messages": ["Message 1", "Message 2", "Message 3", "Message 4", "Message 5"],
        "test_id": test_id
    });

    let batch_result: String = loader.call_plugin(
        "test_sender",
        "send_batch_messages",
        batch_request.to_string(),
    )?;

    let batch_response: serde_json::Value = serde_json::from_str(&batch_result)?;
    assert!(batch_response["success"].as_bool().unwrap());
    assert_eq!(batch_response["sent_count"].as_i64().unwrap(), 5);

    // 处理所有接收到的消息
    let mut received_count = 0;
    while received_count < 5 {
        if let Ok(Some(msg)) = timeout(Duration::from_secs(1), receiver_rx.recv()).await {
            let _: String = loader.call_plugin(
                "test_receiver",
                "process_message",
                serde_json::to_string(&msg)?,
            )?;
            received_count += 1;
        } else {
            break;
        }
    }

    assert_eq!(received_count, 5, "应该接收到5条消息");

    // 验证统计信息
    let stats_result: String = loader.call_plugin("test_receiver", "get_stats", ())?;
    let stats: serde_json::Value = serde_json::from_str(&stats_result)?;
    assert_eq!(stats["total_messages_received"].as_i64().unwrap(), 5);

    drop(router_handle);
    Ok(())
}

#[tokio::test]
async fn test_send_to_nonexistent_plugin() -> anyhow::Result<()> {
    build_test_plugins().await?;

    let (mut loader, _handle, router, _storage, _temp_dir) = setup_test_env().await?;

    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 只加载发送器
    loader.load_plugin(
        "test_sender",
        "plugins/test_sender/target/wasm32-unknown-unknown/release/test_sender.wasm",
    )?;

    loader.call_plugin::<(), String>("test_sender", "init", ())?;

    // 尝试发送到不存在的插件
    let test_id = "error_test_001";
    let result: String =
        loader.call_plugin("test_sender", "send_to_nonexistent", test_id.to_string())?;

    let response: serde_json::Value = serde_json::from_str(&result)?;
    assert_eq!(response["test_id"].as_str().unwrap(), test_id);

    // 注意：由于当前实现，消息仍然会被发送，但不会有接收者
    // 这可以在未来改进为返回错误

    drop(router_handle);
    Ok(())
}

#[tokio::test]
async fn test_message_persistence() -> anyhow::Result<()> {
    build_test_plugins().await?;

    let (mut loader, handle, router, storage, _temp_dir) = setup_test_env().await?;

    let _sender_rx = handle.register_plugin("test_sender".to_string());
    let mut receiver_rx = handle.register_plugin("test_receiver".to_string());

    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 加载插件
    loader.load_plugin(
        "test_sender",
        "plugins/test_sender/target/wasm32-unknown-unknown/release/test_sender.wasm",
    )?;
    loader.load_plugin(
        "test_receiver",
        "plugins/test_receiver/target/wasm32-unknown-unknown/release/test_receiver.wasm",
    )?;

    // 初始化插件
    loader.call_plugin::<(), String>("test_sender", "init", ())?;
    loader.call_plugin::<(), String>("test_receiver", "init", ())?;

    // 发送消息
    let test_id = "persistence_test_001";
    let send_request = json!({
        "to": "test_receiver",
        "message": "Test persistence",
        "test_id": test_id
    });

    let send_result: String =
        loader.call_plugin("test_sender", "send_test_message", send_request.to_string())?;

    let send_response: serde_json::Value = serde_json::from_str(&send_result)?;
    let message_id = send_response["message_id"].as_str().unwrap();

    // 处理消息
    if let Ok(Some(msg)) = timeout(Duration::from_secs(1), receiver_rx.recv()).await {
        loader.call_plugin::<String, String>(
            "test_receiver",
            "process_message",
            serde_json::to_string(&msg)?,
        )?;
    }

    // 验证消息被持久化
    sleep(Duration::from_millis(100)).await;

    // 检查发送记录
    let send_key = format!("sent_{}_{}", test_id, message_id);
    let send_data = storage.get_data("test_sender", &send_key).await?;
    assert!(send_data.is_some(), "发送记录应该被保存");

    // 检查接收计数
    let count_data = storage.get_data("test_receiver", "message_count").await?;
    assert!(count_data.is_some(), "接收计数应该被保存");
    assert_eq!(count_data.unwrap().as_i64().unwrap(), 1);

    drop(router_handle);
    Ok(())
}

#[tokio::test]
async fn test_concurrent_messaging() -> anyhow::Result<()> {
    build_test_plugins().await?;

    let (mut loader, handle, router, _storage, _temp_dir) = setup_test_env().await?;

    let _sender_rx = handle.register_plugin("test_sender".to_string());
    let mut receiver_rx = handle.register_plugin("test_receiver".to_string());

    let router_handle = tokio::spawn(async move {
        router.run().await;
    });

    // 加载插件
    loader.load_plugin(
        "test_sender",
        "plugins/test_sender/target/wasm32-unknown-unknown/release/test_sender.wasm",
    )?;
    loader.load_plugin(
        "test_receiver",
        "plugins/test_receiver/target/wasm32-unknown-unknown/release/test_receiver.wasm",
    )?;

    // 初始化插件
    loader.call_plugin::<(), String>("test_sender", "init", ())?;
    loader.call_plugin::<(), String>("test_receiver", "init", ())?;

    // 重置接收器统计
    loader.call_plugin::<(), String>("test_receiver", "reset_stats", ())?;

    // 并发发送多个消息
    let mut handles = vec![];
    let loader = Arc::new(Mutex::new(loader));

    for i in 0..10 {
        let loader_clone = loader.clone();
        let handle = tokio::spawn(async move {
            let send_request = json!({
                "to": "test_receiver",
                "message": format!("Concurrent message {}", i),
                "test_id": format!("concurrent_test_{}", i)
            });

            let mut loader = loader_clone.lock().unwrap();
            loader.call_plugin::<String, String>(
                "test_sender",
                "send_test_message",
                send_request.to_string(),
            )
        });
        handles.push(handle);
    }

    // 等待所有发送完成
    for handle in handles {
        handle.await??;
    }

    // 处理所有接收到的消息
    let mut received_count = 0;
    let mut loader = Arc::try_unwrap(loader).unwrap().into_inner().unwrap();

    while received_count < 10 {
        if let Ok(Some(msg)) = timeout(Duration::from_secs(2), receiver_rx.recv()).await {
            loader.call_plugin::<String, String>(
                "test_receiver",
                "process_message",
                serde_json::to_string(&msg)?,
            )?;
            received_count += 1;
        } else {
            break;
        }
    }

    assert_eq!(received_count, 10, "应该接收到所有10条消息");

    // 验证统计信息
    let stats_result: String = loader.call_plugin("test_receiver", "get_stats", ())?;
    let stats: serde_json::Value = serde_json::from_str(&stats_result)?;
    assert_eq!(stats["total_messages_received"].as_i64().unwrap(), 10);

    drop(router_handle);
    Ok(())
}
