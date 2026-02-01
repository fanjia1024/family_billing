use crate::domain::entities::member::{CreateMember, Member, UpdateMember};
use crate::domain::ports::member_repository::MemberRepository;
use crate::infrastructure::persistence::database;
use log::error;
use rusqlite::params;
use tauri::AppHandle;

pub struct SqliteMemberRepository {
    app: AppHandle,
}

impl SqliteMemberRepository {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl MemberRepository for SqliteMemberRepository {
    fn list(&self) -> Result<Vec<Member>, String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteMemberRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let rows = conn
            .prepare("SELECT id, family_id, name, avatar, role, created_at FROM member ORDER BY created_at")
            .map_err(|e| {
                error!("[SqliteMemberRepository] SQL准备失败: {}", e);
                e.to_string()
            })?
            .query_map([], |row| {
                Ok(Member {
                    id: row.get(0)?,
                    family_id: row.get(1)?,
                    name: row.get(2)?,
                    avatar: row.get(3)?,
                    role: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| {
                error!("[SqliteMemberRepository] 查询失败: {}", e);
                e.to_string()
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!("[SqliteMemberRepository] 解析失败: {}", e);
                e.to_string()
            })?;

        Ok(rows)
    }

    fn create(&self, family_id: i64, c: &CreateMember) -> Result<i64, String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteMemberRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        conn.execute(
            "INSERT INTO member (family_id, name, role, avatar) VALUES (?1, ?2, ?3, ?4)",
            params![family_id, c.name, c.role, c.avatar],
        )
        .map_err(|e| {
            error!("[SqliteMemberRepository] INSERT失败: {}", e);
            format!("创建成员失败: {}", e)
        })?;

        Ok(conn.last_insert_rowid())
    }

    fn update(&self, id: i64, c: &UpdateMember) -> Result<(), String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteMemberRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let affected = conn
            .execute(
                "UPDATE member SET name = ?1, role = ?2, avatar = ?3 WHERE id = ?4",
                params![c.name, c.role, c.avatar, id],
            )
            .map_err(|e| {
                error!("[SqliteMemberRepository] UPDATE失败: {}", e);
                format!("更新成员失败: {}", e)
            })?;

        if affected == 0 {
            return Err("成员不存在".to_string());
        }
        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteMemberRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let affected = conn
            .execute("DELETE FROM member WHERE id = ?1", params![id])
            .map_err(|e| {
                error!("[SqliteMemberRepository] DELETE失败: {}", e);
                format!("删除成员失败: {}", e)
            })?;

        if affected == 0 {
            return Err("成员不存在".to_string());
        }
        Ok(())
    }
}
