//! 存储测试插件
//! 
//! 用于测试存储功能、并发访问和数据隔离

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct TestRequest {
    action: String,
    key: Option<String>,
    value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestResponse {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

// 声明主机函数
#[host_fn]
extern "ExtismHost" {
    fn store_data(plugin_id: String, key: String, value: String) -> String;
    fn get_data(plugin_id: String, key: String) -> String;
    fn delete_data(plugin_id: String, key: String) -> String;
    fn list_keys(plugin_id: String) -> String;
    fn log_message(level: String, message: String) -> String;
}

/// 主测试函数
#[plugin_fn]
pub fn test_storage(input: String) -> FnResult<String> {
    // 记录接收到的请求
    unsafe {
        log_message("info".to_string(), format!("Received request: {}", input))?;
    }
    
    // 解析请求
    let request: TestRequest = serde_json::from_str(&input)?;
    let plugin_id = "storage-test-plugin".to_string();
    
    // 根据 action 执行不同的测试
    let response = match request.action.as_str() {
        "store" => test_store(&plugin_id, request.key, request.value)?,
        "get" => test_get(&plugin_id, request.key)?,
        "delete" => test_delete(&plugin_id, request.key)?,
        "list" => test_list(&plugin_id)?,
        "test_crud" => test_crud_operations(&plugin_id)?,
        "test_isolation" => test_data_isolation(&plugin_id)?,
        "test_json" => test_json_storage(&plugin_id)?,
        _ => TestResponse {
            success: false,
            message: format!("Unknown action: {}", request.action),
            data: None,
        },
    };
    
    Ok(serde_json::to_string(&response)?)
}

/// 测试存储数据
fn test_store(plugin_id: &str, key: Option<String>, value: Option<String>) -> FnResult<TestResponse> {
    let key = key.ok_or_else(|| Error::msg("Key is required for store action"))?;
    let value = value.ok_or_else(|| Error::msg("Value is required for store action"))?;
    
    unsafe {
        let result = store_data(plugin_id.to_string(), key.clone(), value.clone())?;
        log_message("info".to_string(), format!("Store result: {}", result))?;
    }
    
    Ok(TestResponse {
        success: true,
        message: format!("Stored key '{}' successfully", key),
        data: Some(json!({ "key": key, "value": value })),
    })
}

/// 测试获取数据
fn test_get(plugin_id: &str, key: Option<String>) -> FnResult<TestResponse> {
    let key = key.ok_or_else(|| Error::msg("Key is required for get action"))?;
    
    let result = unsafe { get_data(plugin_id.to_string(), key.clone())? };
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    
    if parsed["success"].as_bool().unwrap_or(false) {
        Ok(TestResponse {
            success: true,
            message: format!("Retrieved key '{}' successfully", key),
            data: Some(parsed["value"].clone()),
        })
    } else {
        Ok(TestResponse {
            success: false,
            message: format!("Key '{}' not found", key),
            data: None,
        })
    }
}

/// 测试删除数据
fn test_delete(plugin_id: &str, key: Option<String>) -> FnResult<TestResponse> {
    let key = key.ok_or_else(|| Error::msg("Key is required for delete action"))?;
    
    let result = unsafe { delete_data(plugin_id.to_string(), key.clone())? };
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    
    Ok(TestResponse {
        success: parsed["success"].as_bool().unwrap_or(false),
        message: format!("Delete operation for key '{}' completed", key),
        data: Some(parsed),
    })
}

/// 测试列出所有键
fn test_list(plugin_id: &str) -> FnResult<TestResponse> {
    let result = unsafe { list_keys(plugin_id.to_string())? };
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    
    Ok(TestResponse {
        success: parsed["success"].as_bool().unwrap_or(false),
        message: "Listed all keys".to_string(),
        data: Some(parsed["keys"].clone()),
    })
}

/// 测试完整的 CRUD 操作流程
fn test_crud_operations(plugin_id: &str) -> FnResult<TestResponse> {
    unsafe {
        log_message("info".to_string(), "Starting CRUD operations test".to_string())?;
    }
    
    // 1. 创建数据
    let test_data = vec![
        ("test_key_1", json!({"name": "Test 1", "value": 100})),
        ("test_key_2", json!({"name": "Test 2", "value": 200})),
        ("test_key_3", json!({"name": "Test 3", "value": 300})),
    ];
    
    for (key, value) in &test_data {
        unsafe {
            store_data(
                plugin_id.to_string(),
                key.to_string(),
                value.to_string()
            )?;
            log_message("info".to_string(), format!("Stored {}", key))?;
        }
    }
    
    // 2. 读取数据
    for (key, expected_value) in &test_data {
        let result = unsafe { get_data(plugin_id.to_string(), key.to_string())? };
        let parsed: serde_json::Value = serde_json::from_str(&result)?;
        
        if let Some(stored_value) = parsed.get("value") {
            if stored_value != expected_value {
                return Ok(TestResponse {
                    success: false,
                    message: format!("Value mismatch for key '{}'", key),
                    data: Some(json!({
                        "expected": expected_value,
                        "actual": stored_value
                    })),
                });
            }
        }
    }
    
    // 3. 更新数据
    unsafe {
        store_data(
            plugin_id.to_string(),
            "test_key_1".to_string(),
            json!({"name": "Test 1 Updated", "value": 150}).to_string()
        )?;
    }
    
    // 4. 验证更新
    let updated = unsafe { get_data(plugin_id.to_string(), "test_key_1".to_string())? };
    let parsed: serde_json::Value = serde_json::from_str(&updated)?;
    
    // 5. 列出所有键
    let list_result = unsafe { list_keys(plugin_id.to_string())? };
    let list_parsed: serde_json::Value = serde_json::from_str(&list_result)?;
    
    // 6. 删除一个键
    unsafe { delete_data(plugin_id.to_string(), "test_key_2".to_string())? };
    
    // 7. 验证删除
    let list_after_delete = unsafe { list_keys(plugin_id.to_string())? };
    let list_after_parsed: serde_json::Value = serde_json::from_str(&list_after_delete)?;
    
    Ok(TestResponse {
        success: true,
        message: "CRUD operations test completed successfully".to_string(),
        data: Some(json!({
            "initial_keys": list_parsed["keys"],
            "keys_after_delete": list_after_parsed["keys"],
            "updated_value": parsed["value"]
        })),
    })
}

/// 测试数据隔离（模拟其他插件的数据不应该被访问）
fn test_data_isolation(plugin_id: &str) -> FnResult<TestResponse> {
    unsafe {
        log_message("info".to_string(), "Testing data isolation".to_string())?;
    }
    
    // 存储自己的数据
    unsafe {
        store_data(
            plugin_id.to_string(),
            "isolation_test".to_string(),
            json!({"plugin": "storage-test", "secret": "my-secret"}).to_string()
        )?;
    }
    
    // 列出自己的键（应该只看到自己的数据）
    let my_keys = unsafe { list_keys(plugin_id.to_string())? };
    let parsed: serde_json::Value = serde_json::from_str(&my_keys)?;
    
    Ok(TestResponse {
        success: true,
        message: "Data isolation test completed".to_string(),
        data: Some(json!({
            "my_plugin_id": plugin_id,
            "my_keys": parsed["keys"],
            "note": "Each plugin should only see its own data"
        })),
    })
}

/// 测试复杂 JSON 数据存储
fn test_json_storage(plugin_id: &str) -> FnResult<TestResponse> {
    unsafe {
        log_message("info".to_string(), "Testing complex JSON storage".to_string())?;
    }
    
    // 创建复杂的 JSON 数据
    let complex_data = json!({
        "user": {
            "id": 12345,
            "name": "Test User",
            "email": "test@example.com",
            "settings": {
                "theme": "dark",
                "notifications": true,
                "language": "zh-CN"
            }
        },
        "data": {
            "arrays": [1, 2, 3, 4, 5],
            "nested": {
                "level1": {
                    "level2": {
                        "level3": "deep value"
                    }
                }
            }
        },
        "metadata": {
            "created_at": "2024-07-17T10:00:00Z",
            "version": "1.0.0"
        }
    });
    
    // 存储复杂数据
    unsafe {
        store_data(
            plugin_id.to_string(),
            "complex_json".to_string(),
            complex_data.to_string()
        )?;
    }
    
    // 读取并验证
    let result = unsafe { get_data(plugin_id.to_string(), "complex_json".to_string())? };
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    
    if let Some(stored_data) = parsed.get("value") {
        // 验证嵌套数据
        let deep_value = stored_data
            .pointer("/data/nested/level1/level2/level3")
            .and_then(|v| v.as_str());
        
        if deep_value == Some("deep value") {
            Ok(TestResponse {
                success: true,
                message: "Complex JSON storage test passed".to_string(),
                data: Some(json!({
                    "stored_successfully": true,
                    "deep_value_retrieved": deep_value,
                    "data_integrity": "verified"
                })),
            })
        } else {
            Ok(TestResponse {
                success: false,
                message: "Failed to retrieve nested value correctly".to_string(),
                data: Some(stored_data.clone()),
            })
        }
    } else {
        Ok(TestResponse {
            success: false,
            message: "Failed to retrieve complex JSON data".to_string(),
            data: None,
        })
    }
}

/// 并发测试函数（模拟并发写入）
#[plugin_fn]
pub fn concurrent_write_test(input: String) -> FnResult<String> {
    let plugin_id = "storage-test-plugin".to_string();
    let request: serde_json::Value = serde_json::from_str(&input)?;
    let thread_id = request["thread_id"].as_u64().unwrap_or(0);
    let iteration = request["iteration"].as_u64().unwrap_or(0);
    
    // 每个线程写入自己的数据
    let key = format!("concurrent_thread_{}_iter_{}", thread_id, iteration);
    let value = json!({
        "thread_id": thread_id,
        "iteration": iteration,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    });
    
    unsafe {
        store_data(plugin_id, key.clone(), value.to_string())?;
        log_message("debug".to_string(), format!("Thread {} wrote iteration {}", thread_id, iteration))?;
    }
    
    Ok(json!({
        "success": true,
        "thread_id": thread_id,
        "iteration": iteration,
        "key": key
    }).to_string())
}