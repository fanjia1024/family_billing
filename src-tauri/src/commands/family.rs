use crate::models::family::Family;
use crate::services::database;
use tauri::AppHandle;
use rusqlite::params;
use anyhow::Result;
use log::{info, debug, error, warn};

#[tauri::command]
pub fn get_family(app: AppHandle) -> Result<Option<Family>, String> {
    info!("[get_family] 开始获取家庭信息");
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[get_family] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let mut stmt = conn
        .prepare("SELECT id, name, created_at FROM family LIMIT 1")
        .map_err(|e| {
            error!("[get_family] SQL准备失败: {}", e);
            e.to_string()
        })?;
    
    match stmt.query_row([], |row| {
        Ok(Family {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    }) {
        Ok(family) => {
            info!("[get_family] 获取家庭成功: id={}, name={}", family.id, family.name);
            Ok(Some(family))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            info!("[get_family] 未找到家庭信息");
            Ok(None)
        }
        Err(e) => {
            error!("[get_family] 查询失败: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn create_family(app: AppHandle, name: String) -> Result<i64, String> {
    info!("[create_family] 开始创建家庭: name={}", name);
    
    if name.trim().is_empty() {
        warn!("[create_family] 家庭名称为空");
        return Err("家庭名称不能为空".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[create_family] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    // 检查是否已存在家庭
    let existing: i32 = conn
        .query_row("SELECT COUNT(*) FROM family", [], |row| row.get(0))
        .map_err(|e| {
            error!("[create_family] 检查现有家庭失败: {}", e);
            e.to_string()
        })?;
    
    if existing > 0 {
        warn!("[create_family] 家庭已存在，不允许重复创建");
        return Err("家庭已存在".to_string());
    }
    
    debug!("[create_family] 执行INSERT语句");
    conn.execute(
        "INSERT INTO family (name) VALUES (?1)",
        params![name],
    )
    .map_err(|e| {
        error!("[create_family] INSERT执行失败: {}", e);
        format!("创建家庭失败: {}", e)
    })?;
    
    let id = conn.last_insert_rowid();
    info!("[create_family] 家庭创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_family(app: AppHandle, id: i64, name: String) -> Result<(), String> {
    info!("[update_family] 开始更新家庭: id={}, name={}", id, name);
    
    if id <= 0 {
        warn!("[update_family] 无效的家庭ID: {}", id);
        return Err("无效的家庭ID".to_string());
    }
    
    if name.trim().is_empty() {
        warn!("[update_family] 家庭名称为空");
        return Err("家庭名称不能为空".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[update_family] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let affected = conn.execute(
        "UPDATE family SET name = ?1 WHERE id = ?2",
        params![name, id],
    )
    .map_err(|e| {
        error!("[update_family] UPDATE执行失败: {}", e);
        format!("更新家庭失败: {}", e)
    })?;
    
    if affected == 0 {
        warn!("[update_family] 未找到要更新的家庭, id={}", id);
        return Err("家庭不存在".to_string());
    }
    
    info!("[update_family] 家庭更新成功, id={}", id);
    Ok(())
}
