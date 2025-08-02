//! 存储模块
//!
//! 基于 SQLite 的本地存储

pub mod layout;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::path::Path;

/// 插件数据模型
#[derive(Debug, sqlx::FromRow)]
pub struct PluginData {
    pub id: i64,
    pub plugin_id: String,
    pub key: String,
    pub value: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 插件元数据模型
#[derive(Debug, sqlx::FromRow)]
pub struct PluginMetadata {
    pub id: i64,
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub enabled: bool,
    pub loaded_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
    pub config: Option<JsonValue>,
}

/// 消息日志模型
#[derive(Debug, sqlx::FromRow)]
pub struct MessageLogEntry {
    pub id: i64,
    pub message_id: String,
    pub from_plugin: String,
    pub to_plugin: String,
    pub payload: Option<Vec<u8>>,
    pub message_type: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

/// 存储管理器
pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    /// 创建新的存储实例
    pub async fn new(database_url: &str) -> Result<Self> {
        tracing::info!("正在初始化存储层...");

        // 确保数据库目录存在
        if let Some(parent) = Path::new(database_url.trim_start_matches("sqlite:")).parent() {
            if !parent.exists() && !parent.as_os_str().is_empty() {
                tracing::debug!("创建数据库目录: {:?}", parent);
                std::fs::create_dir_all(parent)?;
            }
        }

        tracing::info!("正在连接数据库: {}", database_url);

        // 创建连接池，添加超时和优化配置
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(60))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(database_url)
            .await
            .map_err(|e| anyhow::anyhow!("无法连接到数据库: {}", e))?;

        // 设置 SQLite 优化参数
        tracing::debug!("设置 SQLite 优化参数");
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA busy_timeout = 5000")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA temp_store = MEMORY")
            .execute(&pool)
            .await?;

        tracing::info!("正在运行数据库迁移...");

        // 运行迁移，添加超时保护
        let migrate_result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            sqlx::migrate!("./migrations").run(&pool),
        )
        .await;

        match migrate_result {
            Ok(Ok(_)) => {
                tracing::info!("数据库迁移完成");
            }
            Ok(Err(e)) => {
                return Err(anyhow::anyhow!("数据库迁移失败: {}", e));
            }
            Err(_) => {
                return Err(anyhow::anyhow!("数据库迁移超时（10秒）"));
            }
        }

        tracing::info!("存储层初始化完成");
        Ok(Self { pool })
    }

    /// 存储插件数据
    pub async fn store_data(&self, plugin_id: &str, key: &str, value: &JsonValue) -> Result<()> {
        let query = r#"
            INSERT INTO plugin_data (plugin_id, key, value)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(plugin_id, key) DO UPDATE SET
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP
        "#;

        sqlx::query(query)
            .bind(plugin_id)
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 获取插件数据
    pub async fn get_data(&self, plugin_id: &str, key: &str) -> Result<Option<JsonValue>> {
        let query = "SELECT value FROM plugin_data WHERE plugin_id = ?1 AND key = ?2";

        let result = sqlx::query(query)
            .bind(plugin_id)
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.map(|row| row.get("value")))
    }

    /// 删除插件数据
    pub async fn delete_data(&self, plugin_id: &str, key: &str) -> Result<bool> {
        let query = "DELETE FROM plugin_data WHERE plugin_id = ?1 AND key = ?2";

        let result = sqlx::query(query)
            .bind(plugin_id)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取插件的所有键
    pub async fn list_keys(&self, plugin_id: &str) -> Result<Vec<String>> {
        let query = "SELECT key FROM plugin_data WHERE plugin_id = ?1 ORDER BY key";

        let rows = sqlx::query(query)
            .bind(plugin_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| row.get("key")).collect())
    }

    /// 清空插件的所有数据
    pub async fn clear_plugin_data(&self, plugin_id: &str) -> Result<u64> {
        let query = "DELETE FROM plugin_data WHERE plugin_id = ?1";

        let result = sqlx::query(query)
            .bind(plugin_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    // 插件元数据管理

    /// 注册插件
    pub async fn register_plugin(&self, metadata: &PluginMetadata) -> Result<()> {
        let query = r#"
            INSERT INTO plugin_metadata (
                plugin_id, name, version, description, author, enabled, config
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(plugin_id) DO UPDATE SET
                name = excluded.name,
                version = excluded.version,
                description = excluded.description,
                author = excluded.author,
                config = excluded.config,
                last_active = CURRENT_TIMESTAMP
        "#;

        sqlx::query(query)
            .bind(&metadata.plugin_id)
            .bind(&metadata.name)
            .bind(&metadata.version)
            .bind(&metadata.description)
            .bind(&metadata.author)
            .bind(metadata.enabled)
            .bind(&metadata.config)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 更新插件活跃时间
    pub async fn update_plugin_activity(&self, plugin_id: &str) -> Result<()> {
        let query =
            "UPDATE plugin_metadata SET last_active = CURRENT_TIMESTAMP WHERE plugin_id = ?1";

        sqlx::query(query)
            .bind(plugin_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 获取插件元数据
    pub async fn get_plugin_metadata(&self, plugin_id: &str) -> Result<Option<PluginMetadata>> {
        let query = "SELECT * FROM plugin_metadata WHERE plugin_id = ?1";

        let result = sqlx::query_as::<_, PluginMetadata>(query)
            .bind(plugin_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    /// 列出所有插件
    pub async fn list_plugins(&self) -> Result<Vec<PluginMetadata>> {
        let query = "SELECT * FROM plugin_metadata ORDER BY name";

        let result = sqlx::query_as::<_, PluginMetadata>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(result)
    }

    /// 启用/禁用插件
    pub async fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> Result<()> {
        let query = "UPDATE plugin_metadata SET enabled = ?1 WHERE plugin_id = ?2";

        sqlx::query(query)
            .bind(enabled)
            .bind(plugin_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // 消息日志功能

    /// 记录消息
    pub async fn log_message(
        &self,
        message_id: &str,
        from: &str,
        to: &str,
        payload: Option<&[u8]>,
        message_type: Option<&str>,
    ) -> Result<()> {
        let query = r#"
            INSERT INTO message_log (message_id, from_plugin, to_plugin, payload, message_type)
            VALUES (?1, ?2, ?3, ?4, ?5)
        "#;

        sqlx::query(query)
            .bind(message_id)
            .bind(from)
            .bind(to)
            .bind(payload)
            .bind(message_type)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 更新消息状态
    pub async fn update_message_status(&self, message_id: &str, status: &str) -> Result<()> {
        let query = r#"
            UPDATE message_log 
            SET status = ?1, delivered_at = CASE WHEN ?1 = 'delivered' THEN CURRENT_TIMESTAMP ELSE delivered_at END
            WHERE message_id = ?2
        "#;

        sqlx::query(query)
            .bind(status)
            .bind(message_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 获取消息历史
    pub async fn get_message_history(
        &self,
        plugin_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MessageLogEntry>> {
        let query = match plugin_id {
            Some(_) => {
                r#"
                SELECT * FROM message_log 
                WHERE from_plugin = ?1 OR to_plugin = ?1
                ORDER BY created_at DESC
                LIMIT ?2 OFFSET ?3
            "#
            }
            None => {
                r#"
                SELECT * FROM message_log 
                ORDER BY created_at DESC
                LIMIT ?1 OFFSET ?2
            "#
            }
        };

        let result = match plugin_id {
            Some(id) => {
                sqlx::query_as::<_, MessageLogEntry>(query)
                    .bind(id)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_as::<_, MessageLogEntry>(query)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
        };

        Ok(result)
    }

    // 订阅管理

    /// 添加订阅
    pub async fn add_subscription(&self, plugin_id: &str, topic: &str) -> Result<()> {
        let query = r#"
            INSERT INTO plugin_subscriptions (plugin_id, topic)
            VALUES (?1, ?2)
            ON CONFLICT(plugin_id, topic) DO NOTHING
        "#;

        sqlx::query(query)
            .bind(plugin_id)
            .bind(topic)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 移除订阅
    pub async fn remove_subscription(&self, plugin_id: &str, topic: &str) -> Result<bool> {
        let query = "DELETE FROM plugin_subscriptions WHERE plugin_id = ?1 AND topic = ?2";

        let result = sqlx::query(query)
            .bind(plugin_id)
            .bind(topic)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取主题的所有订阅者
    pub async fn get_topic_subscribers(&self, topic: &str) -> Result<Vec<String>> {
        let query = "SELECT plugin_id FROM plugin_subscriptions WHERE topic = ?1";

        let rows = sqlx::query(query).bind(topic).fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(|row| row.get("plugin_id")).collect())
    }

    /// 获取插件的所有订阅
    pub async fn get_plugin_subscriptions(&self, plugin_id: &str) -> Result<Vec<String>> {
        let query = "SELECT topic FROM plugin_subscriptions WHERE plugin_id = ?1";

        let rows = sqlx::query(query)
            .bind(plugin_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| row.get("topic")).collect())
    }

    /// 获取数据库连接池（用于高级操作）
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_db() -> (Storage, TempDir) {
        // 在CI环境中，使用更明确的临时目录路径
        let temp_dir = if std::env::var("CI").is_ok() {
            // CI环境：在当前目录下创建临时目录
            TempDir::new_in(".").unwrap_or_else(|_| TempDir::new().unwrap())
        } else {
            // 本地环境：使用系统默认临时目录
            TempDir::new().unwrap()
        };
        
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let storage = Storage::new(&db_url).await.unwrap();
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_store_and_get_data() {
        let (storage, _temp_dir) = setup_test_db().await;

        let plugin_id = "test_plugin";
        let key = "test_key";
        let value = serde_json::json!({"data": "test_value"});

        // 存储数据
        storage.store_data(plugin_id, key, &value).await.unwrap();

        // 读取数据
        let retrieved = storage.get_data(plugin_id, key).await.unwrap();
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_update_data() {
        let (storage, _temp_dir) = setup_test_db().await;

        let plugin_id = "test_plugin";
        let key = "test_key";
        let value1 = serde_json::json!({"data": "value1"});
        let value2 = serde_json::json!({"data": "value2"});

        // 第一次存储
        storage.store_data(plugin_id, key, &value1).await.unwrap();

        // 更新数据
        storage.store_data(plugin_id, key, &value2).await.unwrap();

        // 验证更新
        let retrieved = storage.get_data(plugin_id, key).await.unwrap();
        assert_eq!(retrieved, Some(value2));
    }

    #[tokio::test]
    async fn test_delete_data() {
        let (storage, _temp_dir) = setup_test_db().await;

        let plugin_id = "test_plugin";
        let key = "test_key";
        let value = serde_json::json!({"data": "test_value"});

        // 存储数据
        storage.store_data(plugin_id, key, &value).await.unwrap();

        // 删除数据
        let deleted = storage.delete_data(plugin_id, key).await.unwrap();
        assert!(deleted);

        // 验证删除
        let retrieved = storage.get_data(plugin_id, key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_list_keys() {
        let (storage, _temp_dir) = setup_test_db().await;

        let plugin_id = "test_plugin";
        let keys = vec!["key1", "key2", "key3"];
        let value = serde_json::json!({"data": "test"});

        // 存储多个键
        for key in &keys {
            storage.store_data(plugin_id, key, &value).await.unwrap();
        }

        // 列出键
        let listed_keys = storage.list_keys(plugin_id).await.unwrap();
        assert_eq!(listed_keys.len(), keys.len());
        for key in keys {
            assert!(listed_keys.contains(&key.to_string()));
        }
    }
}
