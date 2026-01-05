use crate::models::member::Member;
use crate::services::database;
use tauri::AppHandle;
use rusqlite::params;
use anyhow::Result;
use log::{info, debug, error, warn};

#[tauri::command]
pub fn get_members(app: AppHandle) -> Result<Vec<Member>, String> {
    info!("[get_members] 开始获取成员列表");
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[get_members] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let mut stmt = conn
        .prepare("SELECT id, family_id, name, avatar, role, created_at FROM member ORDER BY created_at")
        .map_err(|e| {
            error!("[get_members] SQL准备失败: {}", e);
            e.to_string()
        })?;
    
    let members = stmt
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
            error!("[get_members] 查询执行失败: {}", e);
            e.to_string()
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("[get_members] 结果解析失败: {}", e);
            e.to_string()
        })?;
    
    info!("[get_members] 成功获取 {} 个成员", members.len());
    for m in &members {
        debug!("[get_members] 成员: id={}, name={}, role={}", m.id, m.name, m.role);
    }
    
    Ok(members)
}

#[tauri::command]
pub fn create_member(
    app: AppHandle,
    name: String,
    role: String,
    avatar: Option<String>,
) -> Result<i64, String> {
    info!("[create_member] 开始创建成员: name={}, role={}", name, role);
    
    if name.trim().is_empty() {
        warn!("[create_member] 成员名称为空");
        return Err("成员名称不能为空".to_string());
    }
    
    if role != "admin" && role != "member" {
        warn!("[create_member] 无效的角色: {}", role);
        return Err("角色必须是 admin 或 member".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[create_member] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    // Get or create default family
    let family_id = get_or_create_default_family(&conn)?;
    debug!("[create_member] 使用家庭ID: {}", family_id);
    
    conn.execute(
        "INSERT INTO member (family_id, name, role, avatar) VALUES (?1, ?2, ?3, ?4)",
        params![family_id, name, role, avatar],
    )
    .map_err(|e| {
        error!("[create_member] INSERT执行失败: {}", e);
        format!("创建成员失败: {}", e)
    })?;
    
    let id = conn.last_insert_rowid();
    info!("[create_member] 成员创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_member(
    app: AppHandle,
    id: i64,
    name: String,
    role: String,
    avatar: Option<String>,
) -> Result<(), String> {
    info!("[update_member] 开始更新成员: id={}, name={}, role={}", id, name, role);
    
    if id <= 0 {
        warn!("[update_member] 无效的成员ID: {}", id);
        return Err("无效的成员ID".to_string());
    }
    
    if name.trim().is_empty() {
        warn!("[update_member] 成员名称为空");
        return Err("成员名称不能为空".to_string());
    }
    
    if role != "admin" && role != "member" {
        warn!("[update_member] 无效的角色: {}", role);
        return Err("角色必须是 admin 或 member".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[update_member] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let affected = conn.execute(
        "UPDATE member SET name = ?1, role = ?2, avatar = ?3 WHERE id = ?4",
        params![name, role, avatar, id],
    )
    .map_err(|e| {
        error!("[update_member] UPDATE执行失败: {}", e);
        format!("更新成员失败: {}", e)
    })?;
    
    if affected == 0 {
        warn!("[update_member] 未找到要更新的成员, id={}", id);
        return Err("成员不存在".to_string());
    }
    
    info!("[update_member] 成员更新成功, id={}", id);
    Ok(())
}

#[tauri::command]
pub fn delete_member(app: AppHandle, id: i64) -> Result<(), String> {
    info!("[delete_member] 开始删除成员, id={}", id);
    
    if id <= 0 {
        warn!("[delete_member] 无效的成员ID: {}", id);
        return Err("无效的成员ID".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[delete_member] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    // 检查是否有关联的账单
    let bill_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM bill WHERE member_id = ?1", params![id], |row| row.get(0))
        .map_err(|e| {
            error!("[delete_member] 检查关联账单失败: {}", e);
            e.to_string()
        })?;
    
    if bill_count > 0 {
        warn!("[delete_member] 成员有 {} 条关联账单，无法删除", bill_count);
        return Err(format!("该成员有 {} 条关联账单，请先删除账单", bill_count));
    }
    
    let affected = conn.execute("DELETE FROM member WHERE id = ?1", params![id])
        .map_err(|e| {
            error!("[delete_member] DELETE执行失败: {}", e);
            format!("删除成员失败: {}", e)
        })?;
    
    if affected == 0 {
        warn!("[delete_member] 未找到要删除的成员, id={}", id);
        return Err("成员不存在".to_string());
    }
    
    info!("[delete_member] 成员删除成功, id={}", id);
    Ok(())
}

fn get_or_create_default_family(conn: &rusqlite::Connection) -> Result<i64, String> {
    debug!("[get_or_create_default_family] 获取或创建默认家庭");
    
    let mut stmt = conn
        .prepare("SELECT id FROM family LIMIT 1")
        .map_err(|e| {
            error!("[get_or_create_default_family] SQL准备失败: {}", e);
            e.to_string()
        })?;
    
    match stmt.query_row([], |row| row.get::<_, i64>(0)) {
        Ok(id) => {
            debug!("[get_or_create_default_family] 找到现有家庭: id={}", id);
            Ok(id)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            info!("[get_or_create_default_family] 创建默认家庭");
            conn.execute(
                "INSERT INTO family (name) VALUES ('我的家庭')",
                [],
            )
            .map_err(|e| {
                error!("[get_or_create_default_family] INSERT执行失败: {}", e);
                e.to_string()
            })?;
            
            let id = conn.last_insert_rowid();
            info!("[get_or_create_default_family] 默认家庭创建成功, id={}", id);
            Ok(id)
        }
        Err(e) => {
            error!("[get_or_create_default_family] 查询失败: {}", e);
            Err(e.to_string())
        }
    }
}
