use crate::domain::entities::bill::{Bill, BillFilters, CreateBill, UpdateBill};
use crate::domain::ports::bill_repository::BillRepository;
use crate::infrastructure::persistence::database;
use log::{debug, error};
use rusqlite::params;
use tauri::AppHandle;

pub struct SqliteBillRepository {
    app: AppHandle,
}

impl SqliteBillRepository {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl BillRepository for SqliteBillRepository {
    fn list_with_filters(&self, filters: Option<BillFilters>) -> Result<Vec<Bill>, String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let mut query = "SELECT id, member_id, category_id, type, amount, description, source, bill_date, bill_month, created_at FROM bill WHERE 1=1".to_string();
        let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(f) = filters {
            if let Some(member_id) = f.member_id {
                query.push_str(" AND member_id = ?");
                query_params.push(Box::new(member_id));
            }
            if let Some(category_id) = f.category_id {
                query.push_str(" AND category_id = ?");
                query_params.push(Box::new(category_id));
            }
            if let Some(start_date) = f.start_date {
                query.push_str(" AND bill_date >= ?");
                query_params.push(Box::new(start_date.clone()));
            }
            if let Some(end_date) = f.end_date {
                query.push_str(" AND bill_date <= ?");
                query_params.push(Box::new(end_date.clone()));
            }
        }

        query.push_str(" ORDER BY bill_date DESC, created_at DESC");

        let mut stmt = conn.prepare(&query).map_err(|e| {
            error!("[SqliteBillRepository] SQL准备失败: {}", e);
            e.to_string()
        })?;

        let params: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();

        let bills = stmt
            .query_map(&params[..], |row| {
                let bill_date: String = row.get(7)?;
                let bill_month: Option<String> = row.get(8)?;
                let bill_month = bill_month.or_else(|| {
                    if bill_date.len() >= 7 {
                        Some(bill_date[..7].to_string())
                    } else {
                        None
                    }
                });

                Ok(Bill {
                    id: row.get(0)?,
                    member_id: row.get(1)?,
                    category_id: row.get(2)?,
                    r#type: row.get(3)?,
                    amount: row.get(4)?,
                    description: row.get(5)?,
                    source: row.get(6)?,
                    bill_date,
                    bill_month,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| {
                error!("[SqliteBillRepository] 查询执行失败: {}", e);
                e.to_string()
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!("[SqliteBillRepository] 结果解析失败: {}", e);
                e.to_string()
            })?;

        Ok(bills)
    }

    fn create(&self, bill: &CreateBill) -> Result<i64, String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        debug!("[SqliteBillRepository] 执行INSERT bill");
        conn.execute(
            "INSERT INTO bill (member_id, category_id, type, amount, description, source, bill_date, bill_month) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                bill.member_id,
                bill.category_id,
                bill.r#type,
                bill.amount,
                bill.description,
                bill.source,
                bill.bill_date,
                bill.bill_month
            ],
        )
        .map_err(|e| {
            error!("[SqliteBillRepository] INSERT执行失败: {}", e);
            format!("创建账单失败: {}", e)
        })?;

        Ok(conn.last_insert_rowid())
    }

    fn update(&self, id: i64, bill: &UpdateBill) -> Result<(), String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let mut updates = Vec::new();
        let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(member_id) = bill.member_id {
            updates.push("member_id = ?");
            query_params.push(Box::new(member_id));
        }
        if let Some(category_id) = bill.category_id {
            updates.push("category_id = ?");
            query_params.push(Box::new(category_id));
        }
        if let Some(ref r#type) = bill.r#type {
            updates.push("type = ?");
            query_params.push(Box::new(r#type.clone()));
        }
        if let Some(amount) = bill.amount {
            updates.push("amount = ?");
            query_params.push(Box::new(amount));
        }
        if bill.description.is_some() {
            updates.push("description = ?");
            query_params.push(Box::new(bill.description.clone().unwrap_or_default()));
        }
        if let Some(ref bill_date) = bill.bill_date {
            updates.push("bill_date = ?");
            query_params.push(Box::new(bill_date.clone()));
        }
        if let Some(ref bill_month) = bill.bill_month {
            updates.push("bill_month = ?");
            query_params.push(Box::new(bill_month.clone()));
        }

        if updates.is_empty() {
            return Ok(());
        }

        query_params.push(Box::new(id));

        let query = format!("UPDATE bill SET {} WHERE id = ?", updates.join(", "));
        let params: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();
        conn.execute(&query, &params[..]).map_err(|e| {
            error!("[SqliteBillRepository] UPDATE执行失败: {}", e);
            format!("更新账单失败: {}", e)
        })?;

        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let affected = conn
            .execute("DELETE FROM bill WHERE id = ?1", params![id])
            .map_err(|e| {
                error!("[SqliteBillRepository] DELETE执行失败: {}", e);
                format!("删除账单失败: {}", e)
            })?;

        if affected == 0 {
            return Err("账单不存在".to_string());
        }

        Ok(())
    }

    fn create_bill_image(
        &self,
        bill_id: i64,
        image_path: &str,
        ocr_raw_text: &str,
    ) -> Result<(), String> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        conn.execute(
            "INSERT INTO bill_image (bill_id, image_path, ocr_raw_text) VALUES (?1, ?2, ?3)",
            params![bill_id, image_path, ocr_raw_text],
        )
        .map_err(|e| {
            error!("[SqliteBillRepository] 图片记录INSERT失败: {}", e);
            format!("保存图片记录失败: {}", e)
        })?;

        Ok(())
    }
}
