use crate::models::bill::{Bill, CreateBill, UpdateBill, BillFilters};
use crate::services::database;
use tauri::AppHandle;
use rusqlite::params;
use anyhow::Result;
use log::{info, debug, error, warn};

#[tauri::command]
pub fn get_bills(app: AppHandle, filters: Option<BillFilters>) -> Result<Vec<Bill>, String> {
    info!("[get_bills] 开始获取账单列表");
    debug!("[get_bills] 筛选条件: {:?}", filters);
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[get_bills] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let mut query = "SELECT id, member_id, category_id, type, amount, description, source, bill_date, created_at FROM bill WHERE 1=1".to_string();
    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    
    if let Some(f) = filters {
        if let Some(member_id) = f.member_id {
            query.push_str(" AND member_id = ?");
            query_params.push(Box::new(member_id));
            debug!("[get_bills] 添加成员筛选: member_id={}", member_id);
        }
        if let Some(category_id) = f.category_id {
            query.push_str(" AND category_id = ?");
            query_params.push(Box::new(category_id));
            debug!("[get_bills] 添加分类筛选: category_id={}", category_id);
        }
        if let Some(start_date) = f.start_date {
            query.push_str(" AND bill_date >= ?");
            query_params.push(Box::new(start_date.clone()));
            debug!("[get_bills] 添加开始日期筛选: {}", start_date);
        }
        if let Some(end_date) = f.end_date {
            query.push_str(" AND bill_date <= ?");
            query_params.push(Box::new(end_date.clone()));
            debug!("[get_bills] 添加结束日期筛选: {}", end_date);
        }
    }
    
    query.push_str(" ORDER BY bill_date DESC, created_at DESC");
    debug!("[get_bills] 执行SQL: {}", query);
    
    let mut stmt = conn.prepare(&query).map_err(|e| {
        error!("[get_bills] SQL准备失败: {}", e);
        e.to_string()
    })?;
    
    // Build params array
    let params: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();
    
    let bills = stmt
        .query_map(&params[..], |row| {
            Ok(Bill {
                id: row.get(0)?,
                member_id: row.get(1)?,
                category_id: row.get(2)?,
                r#type: row.get(3)?,
                amount: row.get(4)?,
                description: row.get(5)?,
                source: row.get(6)?,
                bill_date: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| {
            error!("[get_bills] 查询执行失败: {}", e);
            e.to_string()
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("[get_bills] 结果解析失败: {}", e);
            e.to_string()
        })?;
    
    info!("[get_bills] 成功获取 {} 条账单记录", bills.len());
    Ok(bills)
}

#[tauri::command]
pub fn create_bill(app: AppHandle, bill: CreateBill) -> Result<i64, String> {
    info!("[create_bill] 开始创建账单");
    debug!("[create_bill] 账单数据: member_id={}, category_id={}, type={}, amount={}, description={:?}, source={}, bill_date={}",
        bill.member_id, bill.category_id, bill.r#type, bill.amount, bill.description, bill.source, bill.bill_date);
    
    // 数据验证
    if bill.member_id <= 0 {
        warn!("[create_bill] 无效的成员ID: {}", bill.member_id);
        return Err("无效的成员ID".to_string());
    }
    if bill.category_id <= 0 {
        warn!("[create_bill] 无效的分类ID: {}", bill.category_id);
        return Err("无效的分类ID".to_string());
    }
    if bill.amount <= 0.0 {
        warn!("[create_bill] 无效的金额: {}", bill.amount);
        return Err("金额必须大于0".to_string());
    }
    if bill.bill_date.is_empty() {
        warn!("[create_bill] 账单日期为空");
        return Err("账单日期不能为空".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[create_bill] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    debug!("[create_bill] 执行INSERT语句");
    conn.execute(
        "INSERT INTO bill (member_id, category_id, type, amount, description, source, bill_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            bill.member_id,
            bill.category_id,
            bill.r#type,
            bill.amount,
            bill.description,
            bill.source,
            bill.bill_date
        ],
    )
    .map_err(|e| {
        error!("[create_bill] INSERT执行失败: {}", e);
        format!("创建账单失败: {}", e)
    })?;
    
    let id = conn.last_insert_rowid();
    info!("[create_bill] 账单创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_bill(app: AppHandle, id: i64, bill: UpdateBill) -> Result<(), String> {
    info!("[update_bill] 开始更新账单, id={}", id);
    debug!("[update_bill] 更新数据: {:?}", bill);
    
    if id <= 0 {
        warn!("[update_bill] 无效的账单ID: {}", id);
        return Err("无效的账单ID".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[update_bill] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let mut updates = Vec::new();
    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    
    if let Some(member_id) = bill.member_id {
        updates.push("member_id = ?");
        query_params.push(Box::new(member_id));
        debug!("[update_bill] 更新member_id={}", member_id);
    }
    if let Some(category_id) = bill.category_id {
        updates.push("category_id = ?");
        query_params.push(Box::new(category_id));
        debug!("[update_bill] 更新category_id={}", category_id);
    }
    if let Some(ref r#type) = bill.r#type {
        updates.push("type = ?");
        query_params.push(Box::new(r#type.clone()));
        debug!("[update_bill] 更新type={}", r#type);
    }
    if let Some(amount) = bill.amount {
        updates.push("amount = ?");
        query_params.push(Box::new(amount));
        debug!("[update_bill] 更新amount={}", amount);
    }
    if bill.description.is_some() {
        updates.push("description = ?");
        query_params.push(Box::new(bill.description.clone().unwrap_or_default()));
        debug!("[update_bill] 更新description");
    }
    if let Some(ref bill_date) = bill.bill_date {
        updates.push("bill_date = ?");
        query_params.push(Box::new(bill_date.clone()));
        debug!("[update_bill] 更新bill_date={}", bill_date);
    }
    
    if updates.is_empty() {
        warn!("[update_bill] 没有需要更新的字段");
        return Ok(());
    }
    
    query_params.push(Box::new(id));
    
    let query = format!(
        "UPDATE bill SET {} WHERE id = ?",
        updates.join(", ")
    );
    debug!("[update_bill] 执行SQL: {}", query);
    
    let params: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();
    let affected = conn.execute(&query, &params[..])
        .map_err(|e| {
            error!("[update_bill] UPDATE执行失败: {}", e);
            format!("更新账单失败: {}", e)
        })?;
    
    info!("[update_bill] 账单更新成功, 影响行数={}", affected);
    Ok(())
}

#[tauri::command]
pub fn delete_bill(app: AppHandle, id: i64) -> Result<(), String> {
    info!("[delete_bill] 开始删除账单, id={}", id);
    
    if id <= 0 {
        warn!("[delete_bill] 无效的账单ID: {}", id);
        return Err("无效的账单ID".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[delete_bill] 数据库连接失败: {}", e);
        e.to_string()
    })?;
    
    let affected = conn.execute("DELETE FROM bill WHERE id = ?1", params![id])
        .map_err(|e| {
            error!("[delete_bill] DELETE执行失败: {}", e);
            format!("删除账单失败: {}", e)
        })?;
    
    if affected == 0 {
        warn!("[delete_bill] 未找到要删除的账单, id={}", id);
        return Err("账单不存在".to_string());
    }
    
    info!("[delete_bill] 账单删除成功, id={}", id);
    Ok(())
}
