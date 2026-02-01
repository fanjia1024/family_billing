use crate::domain::entities::bill::{Bill, BillFilters, BillImageForExport, CreateBill, UpdateBill};
use crate::domain::error::DomainError;
use crate::domain::ports::bill_repository::BillRepository;
use crate::infrastructure::persistence::database;
use crate::infrastructure::persistence::models::bill_row::BillRow;
use log::{debug, error};
use rusqlite::params;
use tauri::AppHandle;

fn to_persistence(e: impl std::fmt::Display) -> DomainError {
    DomainError::PersistenceError(e.to_string())
}

fn row_to_bill(row: &BillRow) -> Bill {
    let bill_month = row.bill_month.clone().or_else(|| {
        if row.bill_date.len() >= 7 {
            Some(row.bill_date[..7].to_string())
        } else {
            None
        }
    });
    Bill {
        id: row.id,
        member_id: row.member_id,
        category_id: row.category_id,
        r#type: row.r#type.clone(),
        amount: row.amount,
        description: row.description.clone(),
        source: row.source.clone(),
        bill_date: row.bill_date.clone(),
        bill_month,
        created_at: row.created_at.clone(),
    }
}

pub struct SqliteBillRepository {
    app: AppHandle,
}

impl SqliteBillRepository {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl BillRepository for SqliteBillRepository {
    fn list_with_filters(&self, filters: Option<BillFilters>) -> Result<Vec<Bill>, DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            to_persistence(e)
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
            to_persistence(e)
        })?;

        let params: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();

        let rows: Vec<BillRow> = stmt
            .query_map(&params[..], |row| {
                Ok(BillRow {
                    id: row.get(0)?,
                    member_id: row.get(1)?,
                    category_id: row.get(2)?,
                    r#type: row.get(3)?,
                    amount: row.get(4)?,
                    description: row.get(5)?,
                    source: row.get(6)?,
                    bill_date: row.get(7)?,
                    bill_month: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| {
                error!("[SqliteBillRepository] 查询执行失败: {}", e);
                to_persistence(e)
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!("[SqliteBillRepository] 结果解析失败: {}", e);
                to_persistence(e)
            })?;

        Ok(rows.iter().map(row_to_bill).collect())
    }

    fn create(&self, bill: &CreateBill) -> Result<i64, DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            to_persistence(e)
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
            to_persistence(e)
        })?;

        Ok(conn.last_insert_rowid())
    }

    fn update(&self, id: i64, bill: &UpdateBill) -> Result<(), DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            to_persistence(e)
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
            to_persistence(e)
        })?;

        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        let affected = conn
            .execute("DELETE FROM bill WHERE id = ?1", params![id])
            .map_err(|e| {
                error!("[SqliteBillRepository] DELETE执行失败: {}", e);
                to_persistence(e)
            })?;

        if affected == 0 {
            return Err(DomainError::NotFound("账单不存在".to_string()));
        }

        Ok(())
    }

    fn create_bill_image(
        &self,
        bill_id: i64,
        image_path: &str,
        ocr_raw_text: &str,
    ) -> Result<(), DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        conn.execute(
            "INSERT INTO bill_image (bill_id, image_path, ocr_raw_text) VALUES (?1, ?2, ?3)",
            params![bill_id, image_path, ocr_raw_text],
        )
        .map_err(|e| {
            error!("[SqliteBillRepository] 图片记录INSERT失败: {}", e);
            to_persistence(e)
        })?;

        Ok(())
    }

    fn create_bill_with_image(
        &self,
        bill: &CreateBill,
        image_path: &str,
        ocr_raw_text: &str,
    ) -> Result<i64, DomainError> {
        let mut conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        let tx = conn.transaction().map_err(|e| {
            error!("[SqliteBillRepository] 开启事务失败: {}", e);
            to_persistence(e)
        })?;

        tx.execute(
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
            error!("[SqliteBillRepository] INSERT bill 失败: {}", e);
            to_persistence(e)
        })?;

        let bill_id = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO bill_image (bill_id, image_path, ocr_raw_text) VALUES (?1, ?2, ?3)",
            params![bill_id, image_path, ocr_raw_text],
        )
        .map_err(|e| {
            error!("[SqliteBillRepository] INSERT bill_image 失败: {}", e);
            to_persistence(e)
        })?;

        tx.commit().map_err(|e| {
            error!("[SqliteBillRepository] 提交事务失败: {}", e);
            to_persistence(e)
        })?;

        Ok(bill_id)
    }

    fn list_bill_images(&self) -> Result<Vec<BillImageForExport>, DomainError> {
        let conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteBillRepository] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        let rows = conn
            .prepare("SELECT bill_id, image_path, ocr_raw_text, created_at FROM bill_image")
            .map_err(|e| {
                error!("[SqliteBillRepository] SQL准备失败 (bill_image): {}", e);
                to_persistence(e)
            })?
            .query_map([], |row| {
                Ok(BillImageForExport {
                    bill_id: row.get(0)?,
                    image_path: row.get(1)?,
                    ocr_raw_text: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| {
                error!("[SqliteBillRepository] 查询bill_image失败: {}", e);
                to_persistence(e)
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!("[SqliteBillRepository] 解析bill_image失败: {}", e);
                to_persistence(e)
            })?;

        Ok(rows)
    }
}
