use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DashboardLayout {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub grid_columns: i64,
    pub grid_rows: i64,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LayoutWidget {
    pub id: i64,
    pub layout_id: i64,
    pub widget_type: String,
    pub plugin_id: Option<String>,
    pub position_col: i64,
    pub position_row: i64,
    pub size_col_span: i64,
    pub size_row_span: i64,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLayoutRequest {
    pub name: String,
    pub description: Option<String>,
    pub grid_columns: Option<i64>,
    pub grid_rows: Option<i64>,
    pub widgets: Vec<CreateWidgetRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWidgetRequest {
    pub widget_type: String,
    pub plugin_id: Option<String>,
    pub position_col: i64,
    pub position_row: i64,
    pub size_col_span: i64,
    pub size_row_span: i64,
    pub config: Option<serde_json::Value>,
}

pub struct LayoutManager {
    pool: SqlitePool,
}

impl LayoutManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 辅助函数：将 SQLite 的 NaiveDateTime 转换为 DateTime<Utc>
    fn naive_to_utc(naive: Option<NaiveDateTime>) -> DateTime<Utc> {
        naive.map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or_else(Utc::now)
    }

    /// 创建新的布局
    pub async fn create_layout(&self, request: CreateLayoutRequest) -> Result<DashboardLayout> {
        let grid_columns = request.grid_columns.unwrap_or(4);
        let grid_rows = request.grid_rows.unwrap_or(3);

        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 插入布局
        let layout_id = sqlx::query!(
            r#"
            INSERT INTO dashboard_layouts (name, description, grid_columns, grid_rows)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            request.name,
            request.description,
            grid_columns,
            grid_rows
        )
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

        // 插入组件
        for widget in request.widgets {
            sqlx::query!(
                r#"
                INSERT INTO layout_widgets 
                (layout_id, widget_type, plugin_id, position_col, position_row, 
                 size_col_span, size_row_span, config)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                layout_id,
                widget.widget_type,
                widget.plugin_id,
                widget.position_col,
                widget.position_row,
                widget.size_col_span,
                widget.size_row_span,
                widget.config
            )
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;

        // 返回创建的布局
        self.get_layout(layout_id).await
    }

    /// 获取布局详情
    pub async fn get_layout(&self, layout_id: i64) -> Result<DashboardLayout> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, grid_columns, grid_rows,
                   is_default, created_at, updated_at
            FROM dashboard_layouts
            WHERE id = ?1
            "#,
        )
        .bind(layout_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardLayout {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            grid_columns: row.get("grid_columns"),
            grid_rows: row.get("grid_rows"),
            is_default: row.get("is_default"),
            created_at: Self::naive_to_utc(row.get("created_at")),
            updated_at: Self::naive_to_utc(row.get("updated_at")),
        })
    }

    /// 获取布局的所有组件
    pub async fn get_layout_widgets(&self, layout_id: i64) -> Result<Vec<LayoutWidget>> {
        let rows = sqlx::query(
            r#"
            SELECT id, layout_id, widget_type, plugin_id, 
                   position_col, position_row, size_col_span, size_row_span,
                   config, created_at
            FROM layout_widgets
            WHERE layout_id = ?1
            ORDER BY position_row, position_col
            "#,
        )
        .bind(layout_id)
        .fetch_all(&self.pool)
        .await?;

        let widgets = rows
            .into_iter()
            .map(|row| {
                let config_str: Option<String> = row.get("config");
                let config = config_str
                    .and_then(|s| serde_json::from_str(&s).ok());

                LayoutWidget {
                    id: row.get("id"),
                    layout_id: row.get("layout_id"),
                    widget_type: row.get("widget_type"),
                    plugin_id: row.get("plugin_id"),
                    position_col: row.get("position_col"),
                    position_row: row.get("position_row"),
                    size_col_span: row.get("size_col_span"),
                    size_row_span: row.get("size_row_span"),
                    config,
                    created_at: Self::naive_to_utc(row.get("created_at")),
                }
            })
            .collect();

        Ok(widgets)
    }

    /// 列出所有布局
    pub async fn list_layouts(&self) -> Result<Vec<DashboardLayout>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, grid_columns, grid_rows,
                   is_default, created_at, updated_at
            FROM dashboard_layouts
            ORDER BY is_default DESC, created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let layouts = rows
            .into_iter()
            .map(|row| DashboardLayout {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                grid_columns: row.get("grid_columns"),
                grid_rows: row.get("grid_rows"),
                is_default: row.get("is_default"),
                created_at: Self::naive_to_utc(row.get("created_at")),
                updated_at: Self::naive_to_utc(row.get("updated_at")),
            })
            .collect();

        Ok(layouts)
    }

    /// 设置默认布局
    pub async fn set_default_layout(&self, layout_id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 先清除所有默认标记
        sqlx::query!("UPDATE dashboard_layouts SET is_default = FALSE")
            .execute(&mut *tx)
            .await?;

        // 设置新的默认布局
        sqlx::query!(
            "UPDATE dashboard_layouts SET is_default = TRUE WHERE id = ?1",
            layout_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 删除布局
    pub async fn delete_layout(&self, layout_id: i64) -> Result<()> {
        let result = sqlx::query!("DELETE FROM dashboard_layouts WHERE id = ?1", layout_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Layout not found"));
        }

        Ok(())
    }

    /// 保存当前布局快照
    pub async fn save_current_layout(
        &self,
        name: String,
        widgets: Vec<CreateWidgetRequest>,
    ) -> Result<DashboardLayout> {
        let request = CreateLayoutRequest {
            name,
            description: Some("自动保存的布局".to_string()),
            grid_columns: Some(4),
            grid_rows: Some(3),
            widgets,
        };

        self.create_layout(request).await
    }

    /// 获取默认布局
    pub async fn get_default_layout(&self) -> Result<Option<DashboardLayout>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, grid_columns, grid_rows,
                   is_default, created_at, updated_at
            FROM dashboard_layouts
            WHERE is_default = TRUE
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| DashboardLayout {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            grid_columns: row.get("grid_columns"),
            grid_rows: row.get("grid_rows"),
            is_default: row.get("is_default"),
            created_at: Self::naive_to_utc(row.get("created_at")),
            updated_at: Self::naive_to_utc(row.get("updated_at")),
        }))
    }
}
