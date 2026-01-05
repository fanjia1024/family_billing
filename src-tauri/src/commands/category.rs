use crate::models::category::Category;
use crate::services::database;
use tauri::AppHandle;
use rusqlite::params;
use anyhow::Result;
use log::{info, debug, error, warn};

#[tauri::command]
pub fn get_categories(app: AppHandle) -> Result<Vec<Category>, String> {
    info!("[get_categories] 开始获取分类列表");
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[get_categories] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let mut stmt = conn
        .prepare("SELECT id, name, type, icon, created_at FROM category ORDER BY type, name")
        .map_err(|e| {
            error!("[get_categories] SQL准备失败: {}", e);
            e.to_string()
        })?;
    
    let categories = stmt
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
            error!("[get_categories] 查询执行失败: {}", e);
            e.to_string()
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("[get_categories] 结果解析失败: {}", e);
            e.to_string()
        })?;
    
    info!("[get_categories] 成功获取 {} 个分类", categories.len());
    for c in &categories {
        debug!("[get_categories] 分类: id={}, name={}, type={}", c.id, c.name, c.r#type);
    }
    
    Ok(categories)
}

#[tauri::command]
pub fn create_category(
    app: AppHandle,
    name: String,
    r#type: String,
    icon: Option<String>,
) -> Result<i64, String> {
    info!("[create_category] 开始创建分类: name={}, type={}", name, r#type);
    
    if name.trim().is_empty() {
        warn!("[create_category] 分类名称为空");
        return Err("分类名称不能为空".to_string());
    }
    
    if r#type != "income" && r#type != "expense" {
        warn!("[create_category] 无效的分类类型: {}", r#type);
        return Err("类型必须是 income 或 expense".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[create_category] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    // 检查同名分类是否存在
    let existing: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM category WHERE name = ?1 AND type = ?2",
            params![name, r#type],
            |row| row.get(0),
        )
        .map_err(|e| {
            error!("[create_category] 检查现有分类失败: {}", e);
            e.to_string()
        })?;
    
    if existing > 0 {
        warn!("[create_category] 同名分类已存在: name={}, type={}", name, r#type);
        return Err("该分类已存在".to_string());
    }
    
    debug!("[create_category] 执行INSERT语句");
    conn.execute(
        "INSERT INTO category (name, type, icon) VALUES (?1, ?2, ?3)",
        params![name, r#type, icon],
    )
    .map_err(|e| {
        error!("[create_category] INSERT执行失败: {}", e);
        format!("创建分类失败: {}", e)
    })?;
    
    let id = conn.last_insert_rowid();
    info!("[create_category] 分类创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_category(
    app: AppHandle,
    id: i64,
    name: String,
    icon: Option<String>,
) -> Result<(), String> {
    info!("[update_category] 开始更新分类: id={}, name={}", id, name);
    
    if id <= 0 {
        warn!("[update_category] 无效的分类ID: {}", id);
        return Err("无效的分类ID".to_string());
    }
    
    if name.trim().is_empty() {
        warn!("[update_category] 分类名称为空");
        return Err("分类名称不能为空".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[update_category] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let affected = conn.execute(
        "UPDATE category SET name = ?1, icon = ?2 WHERE id = ?3",
        params![name, icon, id],
    )
    .map_err(|e| {
        error!("[update_category] UPDATE执行失败: {}", e);
        format!("更新分类失败: {}", e)
    })?;
    
    if affected == 0 {
        warn!("[update_category] 未找到要更新的分类, id={}", id);
        return Err("分类不存在".to_string());
    }
    
    info!("[update_category] 分类更新成功, id={}", id);
    Ok(())
}

#[tauri::command]
pub fn delete_category(app: AppHandle, id: i64) -> Result<(), String> {
    info!("[delete_category] 开始删除分类, id={}", id);
    
    if id <= 0 {
        warn!("[delete_category] 无效的分类ID: {}", id);
        return Err("无效的分类ID".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[delete_category] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    // 检查是否有关联的账单
    let bill_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM bill WHERE category_id = ?1", params![id], |row| row.get(0))
        .map_err(|e| {
            error!("[delete_category] 检查关联账单失败: {}", e);
            e.to_string()
        })?;
    
    if bill_count > 0 {
        warn!("[delete_category] 分类有 {} 条关联账单，无法删除", bill_count);
        return Err(format!("该分类有 {} 条关联账单，请先删除账单或更改账单分类", bill_count));
    }
    
    let affected = conn.execute("DELETE FROM category WHERE id = ?1", params![id])
        .map_err(|e| {
            error!("[delete_category] DELETE执行失败: {}", e);
            format!("删除分类失败: {}", e)
        })?;
    
    if affected == 0 {
        warn!("[delete_category] 未找到要删除的分类, id={}", id);
        return Err("分类不存在".to_string());
    }
    
    info!("[delete_category] 分类删除成功, id={}", id);
    Ok(())
}
