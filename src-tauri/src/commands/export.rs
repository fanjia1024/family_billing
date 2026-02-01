use crate::application::export_app_service::ExportAppService;
use crate::domain::error::to_user_message;
use crate::services::database;
use log::{debug, error, info, warn};
use serde_json;
use std::fs;
use tauri::AppHandle;

#[tauri::command]
pub fn export_to_json(app: AppHandle, file_path: String) -> Result<(), String> {
    info!("[export_to_json] 开始导出数据到JSON: {}", file_path);

    if file_path.is_empty() {
        warn!("[export_to_json] 文件路径为空");
        return Err("文件路径不能为空".to_string());
    }

    let service = app.state::<ExportAppService>();
    service
        .export_to_json(&file_path)
        .map_err(|e| to_user_message(&e))?;

    info!("[export_to_json] 数据导出成功: {}", file_path);
    Ok(())
}

#[tauri::command]
pub fn export_to_excel(app: AppHandle, file_path: String) -> Result<(), String> {
    info!("[export_to_excel] 开始导出数据到Excel: {}", file_path);
    warn!("[export_to_excel] Excel导出暂不支持，将导出为JSON格式");
    let json_path = file_path.replace(".xlsx", ".json");
    export_to_json(app, json_path)
}

#[tauri::command]
pub fn import_from_json(app: AppHandle, file_path: String) -> Result<(), String> {
    info!("[import_from_json] 开始从JSON导入数据: {}", file_path);
    
    if file_path.is_empty() {
        warn!("[import_from_json] 文件路径为空");
        return Err("文件路径不能为空".to_string());
    }
    
    if !std::path::Path::new(&file_path).exists() {
        error!("[import_from_json] 文件不存在: {}", file_path);
        return Err("文件不存在".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[import_from_json] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    debug!("[import_from_json] 读取文件内容");
    let content = fs::read_to_string(&file_path)
        .map_err(|e| {
            error!("[import_from_json] 文件读取失败: {}", e);
            e.to_string()
        })?;
    
    debug!("[import_from_json] 解析JSON");
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| {
            error!("[import_from_json] JSON解析失败: {}", e);
            e.to_string()
        })?;
    
    // Import families
    if let Some(families) = data.get("families").and_then(|v| v.as_array()) {
        info!("[import_from_json] 导入 {} 个家庭", families.len());
        for family in families {
            let id: i64 = family.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let name = family.get("name").and_then(|v| v.as_str()).unwrap_or("");
            
            // Check if exists
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM family WHERE id = ?1)",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            
            if !exists {
                debug!("[import_from_json] 导入家庭: id={}, name={}", id, name);
                conn.execute(
                    "INSERT INTO family (id, name, created_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        id,
                        name,
                        family.get("created_at").and_then(|v| v.as_str()).unwrap_or("")
                    ],
                )
                .map_err(|e| {
                    error!("[import_from_json] 导入家庭失败: {}", e);
                    e.to_string()
                })?;
            } else {
                debug!("[import_from_json] 家庭已存在，跳过: id={}", id);
            }
        }
    }
    
    // Import members
    if let Some(members) = data.get("members").and_then(|v| v.as_array()) {
        info!("[import_from_json] 导入 {} 个成员", members.len());
        for member in members {
            let id: i64 = member.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM member WHERE id = ?1)",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            
            if !exists {
                debug!("[import_from_json] 导入成员: id={}", id);
                conn.execute(
                    "INSERT INTO member (id, family_id, name, avatar, role, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        id,
                        member.get("family_id").and_then(|v| v.as_i64()).unwrap_or(1),
                        member.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        member.get("avatar").and_then(|v| v.as_str()),
                        member.get("role").and_then(|v| v.as_str()).unwrap_or("member"),
                        member.get("created_at").and_then(|v| v.as_str()).unwrap_or("")
                    ],
                )
                .map_err(|e| {
                    error!("[import_from_json] 导入成员失败: {}", e);
                    e.to_string()
                })?;
            }
        }
    }
    
    // Import categories
    if let Some(categories) = data.get("categories").and_then(|v| v.as_array()) {
        info!("[import_from_json] 导入 {} 个分类", categories.len());
        for category in categories {
            let id: i64 = category.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM category WHERE id = ?1)",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            
            if !exists {
                debug!("[import_from_json] 导入分类: id={}", id);
                conn.execute(
                    "INSERT INTO category (id, name, type, icon, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        id,
                        category.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        category.get("type").and_then(|v| v.as_str()).unwrap_or("expense"),
                        category.get("icon").and_then(|v| v.as_str()),
                        category.get("created_at").and_then(|v| v.as_str()).unwrap_or("")
                    ],
                )
                .map_err(|e| {
                    error!("[import_from_json] 导入分类失败: {}", e);
                    e.to_string()
                })?;
            }
        }
    }
    
    // Import bills
    if let Some(bills) = data.get("bills").and_then(|v| v.as_array()) {
        info!("[import_from_json] 导入 {} 条账单", bills.len());
        for bill in bills {
            let id: i64 = bill.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM bill WHERE id = ?1)",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            
            if !exists {
                debug!("[import_from_json] 导入账单: id={}", id);
                conn.execute(
                    "INSERT INTO bill (id, member_id, category_id, type, amount, description, source, bill_date, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    rusqlite::params![
                        id,
                        bill.get("member_id").and_then(|v| v.as_i64()).unwrap_or(1),
                        bill.get("category_id").and_then(|v| v.as_i64()).unwrap_or(1),
                        bill.get("type").and_then(|v| v.as_str()).unwrap_or("expense"),
                        bill.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        bill.get("description").and_then(|v| v.as_str()),
                        bill.get("source").and_then(|v| v.as_str()).unwrap_or("manual"),
                        bill.get("bill_date").and_then(|v| v.as_str()).unwrap_or(""),
                        bill.get("created_at").and_then(|v| v.as_str()).unwrap_or("")
                    ],
                )
                .map_err(|e| {
                    error!("[import_from_json] 导入账单失败: {}", e);
                    e.to_string()
                })?;
            }
        }
    }
    
    info!("[import_from_json] 数据导入成功");
    Ok(())
}
