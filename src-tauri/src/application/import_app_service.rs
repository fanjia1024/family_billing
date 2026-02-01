//! Application service for JSON import. Uses UnitOfWork::run_import to persist
//! families → members → categories → bills → bill_images in a single transaction.

use crate::domain::error::DomainError;
use crate::domain::ports::import_context::ImportTransactionContext;
use crate::domain::ports::unit_of_work::UnitOfWork;
use log::{debug, info};
use std::fs;

pub struct ImportAppService {
    uow: Box<dyn UnitOfWork>,
}

impl ImportAppService {
    pub fn new(uow: Box<dyn UnitOfWork>) -> Self {
        Self { uow }
    }

    /// Import from JSON file. Validates basic format, then runs all inserts in one transaction.
    /// Skips entities that already exist (by id). Errors map to DomainError for Command to show via to_user_message.
    pub fn import_from_json(&self, file_path: &str) -> Result<(), DomainError> {
        info!("[ImportAppService] import_from_json: {}", file_path);

        if file_path.is_empty() {
            return Err(DomainError::PersistenceError("文件路径不能为空".to_string()));
        }

        if !std::path::Path::new(file_path).exists() {
            return Err(DomainError::NotFound("文件不存在".to_string()));
        }

        let content = fs::read_to_string(file_path).map_err(|e| {
            DomainError::PersistenceError(format!("文件读取失败: {}", e))
        })?;

        let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            DomainError::PersistenceError(format!("JSON解析失败: {}", e))
        })?;

        let obj = data.as_object().ok_or_else(|| {
            DomainError::PersistenceError("JSON格式无效：应为对象".to_string())
        })?;

        self.uow.run_import(|ctx| {
            if let Some(families) = obj.get("families").and_then(|v| v.as_array()) {
                info!("[ImportAppService] 导入 {} 个家庭", families.len());
                for family in families {
                    let id: i64 = family.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let name = family.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let created_at = family.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                    ctx.insert_family_if_not_exists(id, name, created_at)?;
                }
            }

            if let Some(members) = obj.get("members").and_then(|v| v.as_array()) {
                info!("[ImportAppService] 导入 {} 个成员", members.len());
                for member in members {
                    let id: i64 = member.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let family_id = member.get("family_id").and_then(|v| v.as_i64()).unwrap_or(1);
                    let name = member.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let avatar = member.get("avatar").and_then(|v| v.as_str());
                    let role = member.get("role").and_then(|v| v.as_str()).unwrap_or("member");
                    let created_at = member.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                    ctx.insert_member_if_not_exists(id, family_id, name, avatar, role, created_at)?;
                }
            }

            if let Some(categories) = obj.get("categories").and_then(|v| v.as_array()) {
                info!("[ImportAppService] 导入 {} 个分类", categories.len());
                for category in categories {
                    let id: i64 = category.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let name = category.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let type_ = category.get("type").and_then(|v| v.as_str()).unwrap_or("expense");
                    let icon = category.get("icon").and_then(|v| v.as_str());
                    let created_at = category.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                    ctx.insert_category_if_not_exists(id, name, type_, icon, created_at)?;
                }
            }

            if let Some(bills) = obj.get("bills").and_then(|v| v.as_array()) {
                info!("[ImportAppService] 导入 {} 条账单", bills.len());
                for bill in bills {
                    let id: i64 = bill.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let member_id = bill.get("member_id").and_then(|v| v.as_i64()).unwrap_or(1);
                    let category_id = bill.get("category_id").and_then(|v| v.as_i64()).unwrap_or(1);
                    let type_ = bill.get("type").and_then(|v| v.as_str()).unwrap_or("expense");
                    let amount = bill.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let description = bill.get("description").and_then(|v| v.as_str());
                    let source = bill.get("source").and_then(|v| v.as_str()).unwrap_or("manual");
                    let bill_date = bill.get("bill_date").and_then(|v| v.as_str()).unwrap_or("");
                    let bill_month = bill.get("bill_month").and_then(|v| v.as_str()).unwrap_or("");
                    let created_at = bill.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                    ctx.insert_bill_if_not_exists(
                        id,
                        member_id,
                        category_id,
                        type_,
                        amount,
                        description,
                        source,
                        bill_date,
                        bill_month,
                        created_at,
                    )?;
                }
            }

            if let Some(images) = obj.get("bill_images").and_then(|v| v.as_array()) {
                debug!("[ImportAppService] 导入 {} 条账单图片", images.len());
                for img in images {
                    let bill_id = img.get("bill_id").and_then(|v| v.as_i64()).unwrap_or(0);
                    let image_path = img.get("image_path").and_then(|v| v.as_str()).unwrap_or("");
                    let ocr_raw_text = img.get("ocr_raw_text").and_then(|v| v.as_str()).unwrap_or("");
                    let created_at = img.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                    ctx.insert_bill_image_if_not_exists(
                        bill_id,
                        image_path,
                        ocr_raw_text,
                        created_at,
                    )?;
                }
            }

            Ok(())
        })?;

        info!("[ImportAppService] 导入成功");
        Ok(())
    }
}
