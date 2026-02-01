use crate::domain::error::DomainError;
use crate::domain::ports::bill_repository::BillRepository;
use crate::domain::ports::category_repository::CategoryRepository;
use crate::domain::ports::family_repository::FamilyRepository;
use crate::domain::ports::member_repository::MemberRepository;
use log::info;
use std::fs;

pub struct ExportAppService {
    family_repo: Box<dyn FamilyRepository>,
    member_repo: Box<dyn MemberRepository>,
    category_repo: Box<dyn CategoryRepository>,
    bill_repo: Box<dyn BillRepository>,
}

impl ExportAppService {
    pub fn new(
        family_repo: Box<dyn FamilyRepository>,
        member_repo: Box<dyn MemberRepository>,
        category_repo: Box<dyn CategoryRepository>,
        bill_repo: Box<dyn BillRepository>,
    ) -> Self {
        Self {
            family_repo,
            member_repo,
            category_repo,
            bill_repo,
        }
    }

    pub fn export_to_json(&self, file_path: &str) -> Result<(), DomainError> {
        info!("[ExportAppService] export_to_json");

        let mut export_data = serde_json::Map::new();

        let families: Vec<serde_json::Value> = self
            .family_repo
            .get_default()?
            .into_iter()
            .map(|f| {
                serde_json::json!({
                    "id": f.id,
                    "name": f.name,
                    "created_at": f.created_at,
                })
            })
            .collect();
        export_data.insert("families".to_string(), serde_json::Value::Array(families));

        let members: Vec<serde_json::Value> = self
            .member_repo
            .list()?
            .into_iter()
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "family_id": m.family_id,
                    "name": m.name,
                    "avatar": m.avatar,
                    "role": m.role,
                    "created_at": m.created_at,
                })
            })
            .collect();
        export_data.insert("members".to_string(), serde_json::Value::Array(members));

        let categories: Vec<serde_json::Value> = self
            .category_repo
            .list_all()?
            .into_iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "name": c.name,
                    "type": c.r#type,
                    "icon": c.icon,
                    "created_at": c.created_at,
                })
            })
            .collect();
        export_data.insert("categories".to_string(), serde_json::Value::Array(categories));

        let bills: Vec<serde_json::Value> = self
            .bill_repo
            .list_with_filters(None)?
            .into_iter()
            .map(|b| {
                serde_json::json!({
                    "id": b.id,
                    "member_id": b.member_id,
                    "category_id": b.category_id,
                    "type": b.r#type,
                    "amount": b.amount,
                    "description": b.description,
                    "source": b.source,
                    "bill_date": b.bill_date,
                    "bill_month": b.bill_month,
                    "created_at": b.created_at,
                })
            })
            .collect();
        export_data.insert("bills".to_string(), serde_json::Value::Array(bills));

        let bill_images: Vec<serde_json::Value> = self
            .bill_repo
            .list_bill_images()?
            .into_iter()
            .map(|img| {
                serde_json::json!({
                    "bill_id": img.bill_id,
                    "image_path": img.image_path,
                    "ocr_raw_text": img.ocr_raw_text,
                    "created_at": img.created_at,
                })
            })
            .collect();
        export_data.insert("bill_images".to_string(), serde_json::Value::Array(bill_images));

        let json = serde_json::to_string_pretty(&export_data)
            .map_err(|e| DomainError::PersistenceError(format!("JSON序列化失败: {}", e)))?;
        fs::write(file_path, json)
            .map_err(|e| DomainError::PersistenceError(format!("文件写入失败: {}", e)))?;

        info!("[ExportAppService] 导出成功: {}", file_path);
        Ok(())
    }
}
