use crate::domain::entities::category::{Category, CreateCategory, UpdateCategory};
use crate::domain::ports::category_repository::CategoryRepository;
use crate::infrastructure::persistence::database;
use log::error;
use rusqlite::params;
use tauri::AppHandle;

pub struct SqliteCategoryRepository {
    app: AppHandle,
}

impl SqliteCategoryRepository {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl CategoryRepository for SqliteCategoryRepository {
    fn list_all(&self) -> Result<Vec<Category>, String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteCategoryRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let rows = conn
            .prepare("SELECT id, name, type, icon, created_at FROM category ORDER BY type, name")
            .map_err(|e| {
                error!("[SqliteCategoryRepository] SQL准备失败: {}", e);
                e.to_string()
            })?
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    r#type: row.get(2)?,
                    icon: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| {
                error!("[SqliteCategoryRepository] 查询失败: {}", e);
                e.to_string()
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!("[SqliteCategoryRepository] 解析失败: {}", e);
                e.to_string()
            })?;

        Ok(rows)
    }

    fn create(&self, c: &CreateCategory) -> Result<i64, String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteCategoryRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        conn.execute(
            "INSERT INTO category (name, type, icon) VALUES (?1, ?2, ?3)",
            params![c.name, c.r#type, c.icon],
        )
        .map_err(|e| {
            error!("[SqliteCategoryRepository] INSERT失败: {}", e);
            format!("创建分类失败: {}", e)
        })?;

        Ok(conn.last_insert_rowid())
    }

    fn update(&self, id: i64, c: &UpdateCategory) -> Result<(), String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteCategoryRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let affected = conn
            .execute(
                "UPDATE category SET name = ?1, icon = ?2 WHERE id = ?3",
                params![c.name, c.icon, id],
            )
            .map_err(|e| {
                error!("[SqliteCategoryRepository] UPDATE失败: {}", e);
                format!("更新分类失败: {}", e)
            })?;

        if affected == 0 {
            return Err("分类不存在".to_string());
        }
        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteCategoryRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let affected = conn
            .execute("DELETE FROM category WHERE id = ?1", params![id])
            .map_err(|e| {
                error!("[SqliteCategoryRepository] DELETE失败: {}", e);
                format!("删除分类失败: {}", e)
            })?;

        if affected == 0 {
            return Err("分类不存在".to_string());
        }
        Ok(())
    }
}
