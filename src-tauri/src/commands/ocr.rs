use crate::models::bill::CreateBill;
use crate::services::{database, ocr_service};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use log::{info, debug, error, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrResult {
    pub r#type: String,
    pub amount: f64,
    pub description: String,
    pub bill_date: String,
    pub raw_text: String,
}

/// 单个账单项
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrBillItem {
    pub category: String,
    pub amount: f64,
    pub percentage: Option<f64>,
    pub bill_type: String,  // income 或 expense
}

/// 批量 OCR 识别结果
#[derive(Debug, Serialize, Deserialize)]
pub struct OcrBatchResult {
    pub items: Vec<OcrBillItem>,
    pub total_amount: f64,
    pub bill_date: String,
    pub raw_text: String,
}

#[tauri::command]
pub fn ocr_recognize(_app: AppHandle, image_path: String) -> Result<OcrResult, String> {
    info!("[ocr_recognize] 开始OCR识别: image_path={}", image_path);
    
    if image_path.is_empty() {
        warn!("[ocr_recognize] 图片路径为空");
        return Err("图片路径不能为空".to_string());
    }
    
    // 检查文件是否存在
    if !std::path::Path::new(&image_path).exists() {
        error!("[ocr_recognize] 图片文件不存在: {}", image_path);
        return Err("图片文件不存在".to_string());
    }
    
    debug!("[ocr_recognize] 调用OCR服务");
    let result = ocr_service::recognize_image(&image_path).map_err(|e| {
        error!("[ocr_recognize] OCR识别失败: {}", e);
        format!("OCR识别失败: {}", e)
    })?;

    info!("[ocr_recognize] OCR识别成功: type={}, amount={}, date={}", 
        result.r#type, result.amount, result.bill_date);
    debug!("[ocr_recognize] 原始文本长度: {} 字符", result.raw_text.len());

    Ok(OcrResult {
        r#type: result.r#type,
        amount: result.amount,
        description: result.description,
        bill_date: result.bill_date,
        raw_text: result.raw_text,
    })
}

/// 批量OCR识别，返回多条账单
#[tauri::command]
pub fn ocr_recognize_batch(_app: AppHandle, image_path: String) -> Result<OcrBatchResult, String> {
    info!("[ocr_recognize_batch] 开始批量OCR识别: image_path={}", image_path);
    
    if image_path.is_empty() {
        warn!("[ocr_recognize_batch] 图片路径为空");
        return Err("图片路径不能为空".to_string());
    }
    
    // 检查文件是否存在
    if !std::path::Path::new(&image_path).exists() {
        error!("[ocr_recognize_batch] 图片文件不存在: {}", image_path);
        return Err("图片文件不存在".to_string());
    }
    
    debug!("[ocr_recognize_batch] 调用OCR服务识别多条账单");
    let result = ocr_service::recognize_bill_details(&image_path).map_err(|e| {
        error!("[ocr_recognize_batch] OCR批量识别失败: {}", e);
        format!("OCR识别失败: {}", e)
    })?;

    let items: Vec<OcrBillItem> = result.items.iter().map(|item| OcrBillItem {
        category: item.category.clone(),
        amount: item.amount,
        percentage: item.percentage,
        bill_type: item.bill_type.clone(),
    }).collect();

    info!("[ocr_recognize_batch] OCR批量识别成功: {} 条账单, 总金额={}, 日期={}", 
        items.len(), result.total_amount, result.bill_date);
    debug!("[ocr_recognize_batch] 原始文本长度: {} 字符", result.raw_text.len());

    Ok(OcrBatchResult {
        items,
        total_amount: result.total_amount,
        bill_date: result.bill_date,
        raw_text: result.raw_text,
    })
}

#[tauri::command]
pub fn save_bill_with_image(
    app: AppHandle,
    bill: CreateBill,
    image_path: String,
) -> Result<i64, String> {
    info!("[save_bill_with_image] 开始保存账单和图片");
    debug!("[save_bill_with_image] 账单数据: member_id={}, category_id={}, type={}, amount={}, date={}",
        bill.member_id, bill.category_id, bill.r#type, bill.amount, bill.bill_date);
    debug!("[save_bill_with_image] 图片路径: {}", image_path);
    
    // 数据验证
    if bill.member_id <= 0 {
        warn!("[save_bill_with_image] 无效的成员ID: {}", bill.member_id);
        return Err("无效的成员ID".to_string());
    }
    if bill.category_id <= 0 {
        warn!("[save_bill_with_image] 无效的分类ID: {}", bill.category_id);
        return Err("无效的分类ID".to_string());
    }
    if bill.amount <= 0.0 {
        warn!("[save_bill_with_image] 无效的金额: {}", bill.amount);
        return Err("金额必须大于0".to_string());
    }
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[save_bill_with_image] 数据库连接失败: {}", e);
        e.to_string()
    })?;

    // Save bill
    debug!("[save_bill_with_image] 保存账单记录");
    conn.execute(
        "INSERT INTO bill (member_id, category_id, type, amount, description, source, bill_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
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
        error!("[save_bill_with_image] 账单INSERT失败: {}", e);
        format!("保存账单失败: {}", e)
    })?;

    let bill_id = conn.last_insert_rowid();
    info!("[save_bill_with_image] 账单保存成功, bill_id={}", bill_id);

    // Save image
    debug!("[save_bill_with_image] 尝试OCR识别图片内容");
    let ocr_result = ocr_service::recognize_image(&image_path).ok();

    let ocr_text = ocr_result
        .as_ref()
        .map(|r| r.raw_text.clone())
        .unwrap_or_default();

    debug!("[save_bill_with_image] 保存图片记录");
    conn.execute(
        "INSERT INTO bill_image (bill_id, image_path, ocr_raw_text) VALUES (?1, ?2, ?3)",
        rusqlite::params![bill_id, image_path, ocr_text],
    )
    .map_err(|e| {
        error!("[save_bill_with_image] 图片记录INSERT失败: {}", e);
        format!("保存图片记录失败: {}", e)
    })?;

    info!("[save_bill_with_image] 账单和图片保存完成, bill_id={}", bill_id);
    Ok(bill_id)
}
