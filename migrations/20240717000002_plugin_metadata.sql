-- 创建插件元数据表
CREATE TABLE IF NOT EXISTS plugin_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    author TEXT,
    enabled BOOLEAN DEFAULT 1,
    loaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMP,
    config JSON,
    UNIQUE(name, version)
);

-- 创建插件消息日志表（可选，用于调试）
CREATE TABLE IF NOT EXISTS message_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id TEXT NOT NULL UNIQUE,
    from_plugin TEXT NOT NULL,
    to_plugin TEXT NOT NULL,
    payload BLOB,
    message_type TEXT,
    status TEXT DEFAULT 'sent',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    delivered_at TIMESTAMP
);

-- 创建插件订阅表
CREATE TABLE IF NOT EXISTS plugin_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    subscribed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(plugin_id, topic),
    FOREIGN KEY (plugin_id) REFERENCES plugin_metadata(plugin_id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_plugin_metadata_name ON plugin_metadata(name);
CREATE INDEX IF NOT EXISTS idx_message_log_from ON message_log(from_plugin);
CREATE INDEX IF NOT EXISTS idx_message_log_to ON message_log(to_plugin);
CREATE INDEX IF NOT EXISTS idx_message_log_created_at ON message_log(created_at);
CREATE INDEX IF NOT EXISTS idx_plugin_subscriptions_topic ON plugin_subscriptions(topic);