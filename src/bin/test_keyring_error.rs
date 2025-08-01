use anyhow::Result;
use minimal_kernel::identity::IdentityManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("测试 keyring 错误处理");
    println!("=====================");

    // 首先尝试删除现有密钥
    println!("\n1. 尝试删除现有密钥:");
    if IdentityManager::has_saved_key() {
        match IdentityManager::load_from_keyring() {
            Ok(manager) => match manager.delete_from_keyring() {
                Ok(_) => println!("   ✓ 成功删除密钥"),
                Err(e) => println!("   ✗ 删除失败: {}", e),
            },
            Err(e) => println!("   ✗ 无法加载以删除: {}", e),
        }
    } else {
        println!("   没有保存的密钥需要删除");
    }

    // 现在尝试加载不存在的密钥
    println!("\n2. 尝试加载不存在的密钥:");
    match IdentityManager::load_from_keyring() {
        Ok(_) => println!("   ✗ 意外成功！不应该能加载"),
        Err(e) => println!("   ✓ 预期的错误: {}", e),
    }

    // 测试在模拟的 keyring 不可用情况下的行为
    println!("\n3. 测试无效的 keyring 服务名:");
    // 注意：这需要修改 IdentityManager 的构造函数来接受自定义服务名
    // 但由于我们不想修改太多代码，这里只是说明概念
    println!("   当 keyring 服务不可用时，应该返回明确的错误信息");

    Ok(())
}
