use crate::domain::entities::category::{Category, CreateCategory, UpdateCategory};
use crate::domain::error::DomainError;
use crate::domain::ports::category_repository::CategoryRepository;
use crate::infrastructure::persistence::database;
use log::error;
use rusqlite::params;
use tauri::AppHandle;

fn to_persistence(e: impl std::fmt::Display) -> DomainError {
    DomainError::PersistenceError(e.to_string())
}

pub struct SqliteCategoryRepository {
    app: AppHandle,
}

impl SqliteCategoryRepository {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl CategoryRepository for SqliteCategoryRepository {
    fn list_all(&self) -> Result<Vec<Category>, DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteCategoryRepository] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        let rows = conn
            .prepare("SELECT id, name, type, icon, created_at FROM category ORDER BY type, name")
            .map_err(|e| {
                error!("[SqliteCategoryRepository] SQL准备失败: {}", e);
                to_persistence(e)
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
                to_persistence(e)
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!("[SqliteCategoryRepository] 解析失败: {}", e);
                to_persistence(e)
            })?;

        Ok(rows)
    }

    fn create(&self, c: &CreateCategory) -> Result<i64, DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteCategoryRepository] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        conn.execute(
            "INSERT INTO category (name, type, icon) VALUES (?1, ?2, ?3)",
            params![c.name, c.r#type, c.icon],
        )
        .map_err(|e| {
            error!("[SqliteCategoryRepository] INSERT失败: {}", e);
            to_persistence(e)
        })?;

        Ok(conn.last_insert_rowid())
    }

    fn update(&self, id: i64, c: &UpdateCategory) -> Result<(), DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteCategoryRepository] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        let affected = conn
            .execute(
                "UPDATE category SET name = ?1, icon = ?2 WHERE id = ?3",
                params![c.name, c.icon, id],
            )
            .map_err(|e| {
                error!("[SqliteCategoryRepository] UPDATE失败: {}", e);
                to_persistence(e)
            })?;

        if affected == 0 {
            return Err(DomainError::NotFound("分类不存在".to_string()));
        }
        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteCategoryRepository] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        let affected = conn
            .execute("DELETE FROM category WHERE id = ?1", params![id])
            .map_err(|e| {
                error!("[SqliteCategoryRepository] DELETE失败: {}", e);
                to_persistence(e)
            })?;

        if affected == 0 {
            return Err(DomainError::NotFound("分类不存在".to_string()));
        }
        Ok(())
    }
}
