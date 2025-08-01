//! 身份管理模块
//!
//! 基于以太坊的身份和加密系统

use crate::config::IdentityConfig;
use alloy::primitives::{Address, B256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use anyhow::{anyhow, Result};
use keyring::Entry;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 身份管理专用错误类型
#[derive(thiserror::Error, Debug)]
pub enum IdentityError {
    #[error("签名器错误: {0}")]
    SignerError(#[from] alloy::signers::Error),

    #[error("存储错误: {0}")]
    StorageError(#[from] keyring::Error),

    #[error("身份不存在: {address}")]
    IdentityNotFound { address: String },

    #[error("无效的地址格式: {address}")]
    InvalidAddress { address: String },

    #[error("无效的私钥格式")]
    InvalidPrivateKey,

    #[error("签名验证失败")]
    SignatureVerificationFailed,

    #[error("密钥派生失败: {0}")]
    KeyDerivationError(String),

    #[error("十六进制解码错误: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
}

/// 身份管理器
pub struct IdentityManager {
    /// 主密钥签名器
    master_key: PrivateKeySigner,
    /// 插件密钥缓存 (plugin_id -> PrivateKeySigner)
    plugin_keys: Arc<RwLock<HashMap<String, PrivateKeySigner>>>,
    /// keyring 条目名称
    keyring_service: String,
    keyring_username: String,
}

impl IdentityManager {
    /// 创建新的身份管理器（使用默认配置）
    pub fn new() -> Result<Self> {
        let master_key = Self::generate_master_key()?;
        Ok(Self {
            master_key,
            plugin_keys: Arc::new(RwLock::new(HashMap::new())),
            keyring_service: "minimal-kernel".to_string(),
            keyring_username: "master-key".to_string(),
        })
    }

    /// 使用配置创建身份管理器
    pub async fn new_with_config(config: &IdentityConfig) -> Result<Self> {
        // 优先从环境变量加载
        if config.allow_env_key {
            if let Ok(private_key_hex) = std::env::var("MINIMAL_KERNEL_PRIVATE_KEY") {
                tracing::info!("从环境变量加载身份密钥");
                return Self::from_private_key(&private_key_hex);
            }
        }

        // 尝试从文件加载
        if !config.use_keyring {
            if let Some(key_file) = &config.private_key_file {
                if key_file.exists() {
                    tracing::info!("从文件加载身份密钥: {:?}", key_file);
                    let key_data = tokio::fs::read_to_string(key_file)
                        .await
                        .map_err(|e| anyhow!("无法读取密钥文件: {}", e))?;
                    let key_data = key_data.trim();
                    return Self::from_private_key(key_data);
                }
            }
        }

        // 使用 keyring
        if config.use_keyring {
            if Self::has_saved_key() {
                tracing::info!("从系统 keyring 加载身份密钥");
                return Self::load_from_keyring();
            } else {
                // 创建新密钥并保存
                tracing::info!("创建新的身份密钥并保存到 keyring");
                let manager = Self::new()?;
                manager.save_to_keyring()?;
                return Ok(manager);
            }
        }

        // 创建新密钥但不保存到 keyring
        tracing::info!("创建新的身份密钥（不保存到 keyring）");
        let manager = Self::new()?;

        // 如果指定了文件路径，保存到文件
        if let Some(key_file) = &config.private_key_file {
            let private_key_hex = hex::encode(manager.master_key.to_bytes().as_slice());
            if let Some(parent) = key_file.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(key_file, private_key_hex)
                .await
                .map_err(|e| anyhow!("无法保存密钥到文件: {}", e))?;
            tracing::info!("密钥已保存到文件: {:?}", key_file);
        }

        Ok(manager)
    }

    /// 从私钥创建身份管理器
    pub fn from_private_key(private_key_hex: &str) -> Result<Self> {
        let master_key: PrivateKeySigner = private_key_hex
            .parse()
            .map_err(|e| anyhow!("Invalid private key: {}", e))?;

        Ok(Self {
            master_key,
            plugin_keys: Arc::new(RwLock::new(HashMap::new())),
            keyring_service: "minimal-kernel".to_string(),
            keyring_username: "master-key".to_string(),
        })
    }

    /// 生成随机主密钥
    pub fn generate_master_key() -> Result<PrivateKeySigner> {
        Ok(PrivateKeySigner::random())
    }

    /// 获取主地址
    pub fn get_master_address(&self) -> Address {
        self.master_key.address()
    }

    /// 验证密钥是否有效
    pub fn verify_key(&self) -> Result<bool> {
        // 简单验证：尝试签名一条测试消息
        let test_message = b"test message for verification";
        let _signature = self.master_key.sign_message_sync(test_message)?;
        Ok(true)
    }

    /// 为插件派生密钥（确定性）
    pub async fn derive_plugin_key(&self, plugin_id: &str) -> Result<PrivateKeySigner> {
        // 检查缓存
        {
            let cache = self.plugin_keys.read().await;
            if let Some(existing_key) = cache.get(plugin_id) {
                return Ok(existing_key.clone());
            }
        }

        // 生成确定性的插件密钥
        let plugin_key = self.generate_deterministic_key(plugin_id)?;

        // 缓存密钥
        {
            let mut cache = self.plugin_keys.write().await;
            cache.insert(plugin_id.to_string(), plugin_key.clone());
        }

        Ok(plugin_key)
    }

    /// 生成确定性密钥（简化版BIP32）
    fn generate_deterministic_key(&self, plugin_id: &str) -> Result<PrivateKeySigner> {
        // 使用主密钥和插件ID生成确定性种子
        let master_key_bytes = self.master_key.to_bytes();

        // 创建确定性哈希
        let mut hasher = DefaultHasher::new();
        master_key_bytes.hash(&mut hasher);
        plugin_id.hash(&mut hasher);
        "minimal-kernel-plugin-derivation".hash(&mut hasher);
        let hash_result = hasher.finish();

        // 将哈希结果扩展为32字节的种子
        let mut seed = [0u8; 32];
        let hash_bytes = hash_result.to_be_bytes();

        // 重复哈希填充32字节
        for i in 0..32 {
            seed[i] = hash_bytes[i % 8];
        }

        // 混合原始主密钥字节以增加熵
        for (i, &byte) in master_key_bytes.iter().enumerate() {
            if i < 32 {
                seed[i] ^= byte;
            }
        }

        // 从种子创建新的私钥
        let plugin_key = PrivateKeySigner::from_bytes(&B256::from(seed))?;
        Ok(plugin_key)
    }

    /// 获取插件地址
    pub async fn get_plugin_address(&self, plugin_id: &str) -> Result<Address> {
        let plugin_key = self.derive_plugin_key(plugin_id).await?;
        Ok(plugin_key.address())
    }

    /// 为插件签名消息
    pub async fn sign_for_plugin(&self, plugin_id: &str, message: &[u8]) -> Result<Vec<u8>> {
        let plugin_key = self.derive_plugin_key(plugin_id).await?;
        let signature = plugin_key.sign_message_sync(message)?;
        Ok(signature.as_bytes().to_vec())
    }

    /// 验证插件签名
    pub async fn verify_plugin_signature(
        &self,
        plugin_id: &str,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        let plugin_address = self.get_plugin_address(plugin_id).await?;

        // 使用 Alloy 的签名恢复功能
        let signature = alloy::primitives::Signature::try_from(signature)
            .map_err(|e| anyhow!("Invalid signature format: {}", e))?;

        let recovered_address = signature
            .recover_address_from_msg(message)
            .map_err(|e| anyhow!("Failed to recover address: {}", e))?;

        Ok(recovered_address == plugin_address)
    }

    /// 保存主密钥到系统keyring
    pub fn save_to_keyring(&self) -> Result<()> {
        let private_key_hex = hex::encode(self.master_key.to_bytes().as_slice());

        // 创建 keyring 条目
        let entry = Entry::new(&self.keyring_service, &self.keyring_username)
            .map_err(|e| anyhow!("无法访问系统密钥服务: {}", e))?;

        // 保存密钥
        entry
            .set_password(&private_key_hex)
            .map_err(|e| anyhow!("无法保存密钥到系统密钥服务: {}", e))?;

        tracing::info!("密钥已保存到系统 keyring");
        Ok(())
    }

    /// 从系统keyring加载主密钥
    pub fn load_from_keyring() -> Result<Self> {
        let keyring_service = "minimal-kernel".to_string();
        let keyring_username = "master-key".to_string();

        // 创建 keyring 条目
        let entry = Entry::new(&keyring_service, &keyring_username)
            .map_err(|e| anyhow!("无法访问系统密钥服务: {}", e))?;

        // 获取密钥
        let private_key_hex = entry
            .get_password()
            .map_err(|e| anyhow!("无法从系统密钥服务加载密钥: {}", e))?;

        tracing::info!("从系统 keyring 加载密钥");

        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| anyhow!("Invalid hex in stored key: {}", e))?;
        if private_key_bytes.len() != 32 {
            return Err(anyhow!("Invalid private key length"));
        }
        let master_key = PrivateKeySigner::from_bytes(&B256::from_slice(&private_key_bytes))?;

        Ok(Self {
            master_key,
            plugin_keys: Arc::new(RwLock::new(HashMap::new())),
            keyring_service,
            keyring_username,
        })
    }

    /// 检查是否存在保存的密钥
    pub fn has_saved_key() -> bool {
        let keyring_service = "minimal-kernel";
        let keyring_username = "master-key";

        // 检查 keyring
        if let Ok(entry) = Entry::new(keyring_service, keyring_username) {
            entry.get_password().is_ok()
        } else {
            false
        }
    }

    /// 删除keyring中的密钥
    pub fn delete_from_keyring(&self) -> Result<()> {
        let entry = Entry::new(&self.keyring_service, &self.keyring_username)?;
        entry
            .delete_password()
            .map_err(|e| anyhow!("Failed to delete key from keyring: {}", e))?;
        Ok(())
    }
}
