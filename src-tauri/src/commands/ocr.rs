use crate::application::bill_app_service::BillAppService;
use crate::models::bill::CreateBill;
use anyhow::Result;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrResult {
    pub r#type: String,
    pub amount: f64,
    pub description: String,
    pub bill_date: String,
    pub raw_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrBillItem {
    pub category: String,
    pub amount: f64,
    pub percentage: Option<f64>,
    pub bill_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrBatchResult {
    pub items: Vec<OcrBillItem>,
    pub total_amount: f64,
    pub bill_date: String,
    pub bill_month: String,
    pub raw_text: String,
}

#[tauri::command]
pub fn ocr_recognize(app: AppHandle, image_path: String) -> Result<OcrResult, String> {
    info!("[ocr_recognize] 开始OCR识别: image_path={}", image_path);

    if image_path.is_empty() {
        warn!("[ocr_recognize] 图片路径为空");
        return Err("图片路径不能为空".to_string());
    }
    if !std::path::Path::new(&image_path).exists() {
        error!("[ocr_recognize] 图片文件不存在: {}", image_path);
        return Err("图片文件不存在".to_string());
    }

    let service = app.state::<BillAppService>();
    let result = service.recognize_image(&image_path).map_err(|e| {
        error!("[ocr_recognize] OCR识别失败: {}", e);
        format!("OCR识别失败: {}", e)
    })?;

    info!(
        "[ocr_recognize] OCR识别成功: type={}, amount={}, date={}",
        result.r#type, result.amount, result.bill_date
    );

    Ok(OcrResult {
        r#type: result.r#type,
        amount: result.amount,
        description: result.description,
        bill_date: result.bill_date,
        raw_text: result.raw_text,
    })
}

#[tauri::command]
pub fn ocr_recognize_batch(app: AppHandle, image_path: String) -> Result<OcrBatchResult, String> {
    info!("[ocr_recognize_batch] 开始批量OCR识别: image_path={}", image_path);

    if image_path.is_empty() {
        warn!("[ocr_recognize_batch] 图片路径为空");
        return Err("图片路径不能为空".to_string());
    }
    if !std::path::Path::new(&image_path).exists() {
        error!("[ocr_recognize_batch] 图片文件不存在: {}", image_path);
        return Err("图片文件不存在".to_string());
    }

    let service = app.state::<BillAppService>();
    let result = service.recognize_bill_details(&image_path).map_err(|e| {
        error!("[ocr_recognize_batch] OCR批量识别失败: {}", e);
        format!("OCR识别失败: {}", e)
    })?;

    let items: Vec<OcrBillItem> = result
        .items
        .iter()
        .map(|item| OcrBillItem {
            category: item.category.clone(),
            amount: item.amount,
            percentage: item.percentage,
            bill_type: item.bill_type.clone(),
        })
        .collect();

    let bill_month = if result.bill_date.len() >= 7 {
        result.bill_date.chars().take(7).collect::<String>()
    } else {
        chrono::Local::now().format("%Y-%m").to_string()
    };

    Ok(OcrBatchResult {
        items,
        total_amount: result.total_amount,
        bill_date: result.bill_date.clone(),
        bill_month,
        raw_text: result.raw_text.clone(),
    })
}

#[tauri::command]
pub fn save_bill_with_image(
    app: AppHandle,
    bill: CreateBill,
    image_path: String,
) -> Result<i64, String> {
    info!("[save_bill_with_image] 开始保存账单和图片");
    debug!("[save_bill_with_image] 账单数据: member_id={}, category_id={}, amount={}, date={}",
        bill.member_id, bill.category_id, bill.amount, bill.bill_date);

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

    let bill_month = if bill.bill_month.trim().is_empty() {
        bill.bill_date.chars().take(7).collect::<String>()
    } else {
        bill.bill_month.clone()
    };

    let domain_bill = crate::domain::entities::bill::CreateBill {
        member_id: bill.member_id,
        category_id: bill.category_id,
        r#type: bill.r#type.clone(),
        amount: bill.amount,
        description: bill.description.clone(),
        source: bill.source.clone(),
        bill_date: bill.bill_date.clone(),
        bill_month,
    };

    let service = app.state::<BillAppService>();
    let bill_id = service.save_bill_with_ocr(domain_bill, &image_path)?;

    info!("[save_bill_with_image] 账单和图片保存完成, bill_id={}", bill_id);
    Ok(bill_id)
}
