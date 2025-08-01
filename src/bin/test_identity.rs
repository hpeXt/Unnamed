use anyhow::Result;
use minimal_kernel::identity::IdentityManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("测试身份管理器功能");
    println!("===================");

    // 测试1：检查是否有保存的密钥
    println!("\n1. 检查保存的密钥:");
    let has_key = IdentityManager::has_saved_key();
    println!("   存在保存的密钥: {}", has_key);

    // 测试2：尝试加载密钥
    println!("\n2. 尝试加载密钥:");
    match IdentityManager::load_from_keyring() {
        Ok(manager) => {
            println!("   ✓ 成功从 keyring 加载密钥");
            println!("   主地址: {}", manager.get_master_address());

            // 测试3：验证密钥
            println!("\n3. 验证密钥:");
            match manager.verify_key() {
                Ok(true) => println!("   ✓ 密钥验证成功"),
                Ok(false) => println!("   ✗ 密钥验证失败"),
                Err(e) => println!("   ✗ 验证错误: {}", e),
            }

            // 测试4：派生插件密钥
            println!("\n4. 派生插件密钥:");
            let plugin_id = "test-plugin";
            match manager.derive_plugin_key(plugin_id).await {
                Ok(plugin_key) => {
                    println!("   ✓ 成功派生插件密钥");
                    println!("   插件地址: {}", plugin_key.address());
                }
                Err(e) => println!("   ✗ 派生失败: {}", e),
            }
        }
        Err(e) => {
            println!("   ✗ 加载失败: {}", e);

            // 测试创建新密钥
            println!("\n尝试创建新的身份管理器:");
            match IdentityManager::new() {
                Ok(new_manager) => {
                    println!("   ✓ 成功创建新的身份管理器");
                    println!("   主地址: {}", new_manager.get_master_address());

                    // 尝试保存到 keyring
                    println!("\n5. 保存密钥到 keyring:");
                    match new_manager.save_to_keyring() {
                        Ok(_) => println!("   ✓ 成功保存到 keyring"),
                        Err(e) => println!("   ✗ 保存失败: {}", e),
                    }
                }
                Err(e) => println!("   ✗ 创建失败: {}", e),
            }
        }
    }

    // 测试6：测试错误处理（无 keyring 服务的情况）
    println!("\n6. 测试错误处理:");
    println!("   如果 keyring 服务不可用，应该返回明确的错误信息");

    Ok(())
}
