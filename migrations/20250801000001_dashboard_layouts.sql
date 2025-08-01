-- 仪表盘布局表
CREATE TABLE IF NOT EXISTS dashboard_layouts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    grid_columns INTEGER NOT NULL DEFAULT 4,
    grid_rows INTEGER NOT NULL DEFAULT 3,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 布局中的组件配置
CREATE TABLE IF NOT EXISTS layout_widgets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    layout_id INTEGER NOT NULL,
    widget_type TEXT NOT NULL,
    plugin_id TEXT,
    position_col INTEGER NOT NULL,
    position_row INTEGER NOT NULL,
    size_col_span INTEGER NOT NULL DEFAULT 1,
    size_row_span INTEGER NOT NULL DEFAULT 1,
    config JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (layout_id) REFERENCES dashboard_layouts(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_layout_widgets_layout_id ON layout_widgets(layout_id);
CREATE INDEX idx_dashboard_layouts_is_default ON dashboard_layouts(is_default);

-- 添加更新时间触发器
CREATE TRIGGER update_dashboard_layouts_timestamp 
AFTER UPDATE ON dashboard_layouts
BEGIN
    UPDATE dashboard_layouts SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;