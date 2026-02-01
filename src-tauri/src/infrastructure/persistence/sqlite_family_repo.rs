use crate::domain::entities::family::{CreateFamily, Family, UpdateFamily};
use crate::domain::ports::family_repository::FamilyRepository;
use crate::infrastructure::persistence::database;
use log::error;
use rusqlite::params;
use tauri::AppHandle;

pub struct SqliteFamilyRepository {
    app: AppHandle,
}

impl SqliteFamilyRepository {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl FamilyRepository for SqliteFamilyRepository {
    fn get_default(&self) -> Result<Option<Family>, String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteFamilyRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let mut stmt = conn
            .prepare("SELECT id, name, created_at FROM family LIMIT 1")
            .map_err(|e| {
                error!("[SqliteFamilyRepository] SQL准备失败: {}", e);
                e.to_string()
            })?;

        match stmt.query_row([], |row| {
            Ok(Family {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        }) {
            Ok(family) => Ok(Some(family)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => {
                error!("[SqliteFamilyRepository] 查询失败: {}", e);
                Err(e.to_string())
            }
        }
    }

    fn create(&self, c: &CreateFamily) -> Result<i64, String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteFamilyRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        conn.execute("INSERT INTO family (name) VALUES (?1)", params![c.name])
            .map_err(|e| {
                error!("[SqliteFamilyRepository] INSERT失败: {}", e);
                format!("创建家庭失败: {}", e)
            })?;

        Ok(conn.last_insert_rowid())
    }

    fn update(&self, id: i64, c: &UpdateFamily) -> Result<(), String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteFamilyRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let affected = conn
            .execute("UPDATE family SET name = ?1 WHERE id = ?2", params![c.name, id])
            .map_err(|e| {
                error!("[SqliteFamilyRepository] UPDATE失败: {}", e);
                format!("更新家庭失败: {}", e)
            })?;

        if affected == 0 {
            return Err("家庭不存在".to_string());
        }
        Ok(())
    }
}
