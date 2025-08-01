-- 创建插件数据表
CREATE TABLE IF NOT EXISTS plugin_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value JSON NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(plugin_id, key)
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_plugin_data_plugin_id ON plugin_data(plugin_id);
CREATE INDEX IF NOT EXISTS idx_plugin_data_key ON plugin_data(key);
CREATE INDEX IF NOT EXISTS idx_plugin_data_created_at ON plugin_data(created_at);

-- 创建更新时间触发器
CREATE TRIGGER IF NOT EXISTS update_plugin_data_updated_at 
AFTER UPDATE ON plugin_data
BEGIN
    UPDATE plugin_data SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;