//! 身份管理测试

use minimal_kernel::identity::IdentityManager;
use anyhow::Result;

#[tokio::test]
async fn test_identity_manager_creation() -> Result<()> {
    let identity = IdentityManager::new()?;
    
    // 测试密钥验证
    assert!(identity.verify_key().is_ok());
    
    // 测试地址生成
    let address = identity.get_master_address();
    println!("Master address: {:?}", address);
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_key_derivation() -> Result<()> {
    let identity = IdentityManager::new()?;
    
    // 为插件派生地址
    let plugin_addr1 = identity.get_plugin_address("test-plugin-1").await?;
    let plugin_addr2 = identity.get_plugin_address("test-plugin-2").await?;
    let plugin_addr1_again = identity.get_plugin_address("test-plugin-1").await?;
    
    // 验证不同插件有不同地址
    assert_ne!(plugin_addr1, plugin_addr2);
    
    // 验证相同插件ID生成相同地址（缓存）
    assert_eq!(plugin_addr1, plugin_addr1_again);
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_signing() -> Result<()> {
    let identity = IdentityManager::new()?;
    let plugin_id = "test-signer";
    let message = b"Hello, Minimal Kernel!";
    
    // 为插件签名消息
    let signature = identity.sign_for_plugin(plugin_id, message).await?;
    assert!(!signature.is_empty());
    
    // 验证签名
    let is_valid = identity.verify_plugin_signature(plugin_id, message, &signature).await?;
    assert!(is_valid);
    
    // 验证错误的消息不会通过
    let wrong_message = b"Wrong message";
    let is_invalid = identity.verify_plugin_signature(plugin_id, wrong_message, &signature).await?;
    assert!(!is_invalid);
    
    Ok(())
}

#[tokio::test]
async fn test_from_private_key() -> Result<()> {
    // 使用已知私钥创建身份管理器
    let private_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let identity1 = IdentityManager::from_private_key(private_key)?;
    let identity2 = IdentityManager::from_private_key(private_key)?;
    
    // 验证相同私钥生成相同地址
    assert_eq!(identity1.get_master_address(), identity2.get_master_address());
    
    // 验证派生的插件密钥也相同
    let plugin_addr1 = identity1.get_plugin_address("test").await?;
    let plugin_addr2 = identity2.get_plugin_address("test").await?;
    assert_eq!(plugin_addr1, plugin_addr2);
    
    Ok(())
}