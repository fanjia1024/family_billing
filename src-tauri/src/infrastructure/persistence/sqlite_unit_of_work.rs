use crate::domain::entities::bill::CreateBill;
use crate::domain::ports::unit_of_work::{BillTransactionContext, UnitOfWork};
use crate::infrastructure::persistence::database;
use log::error;
use rusqlite::params;
use tauri::AppHandle;

pub struct SqliteUnitOfWork {
    app: AppHandle,
}

impl SqliteUnitOfWork {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl UnitOfWork for SqliteUnitOfWork {
    fn run_bill_with_image<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut dyn BillTransactionContext) -> Result<T, String>,
    {
        let mut conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteUnitOfWork] 数据库连接失败: {}", e);
            e.to_string()
        })?;

        let tx = conn.transaction().map_err(|e| {
            error!("[SqliteUnitOfWork] 开启事务失败: {}", e);
            e.to_string()
        })?;

        let mut ctx = SqliteBillTransactionContext { tx: &tx };
        let result = f(&mut ctx);

        match result {
            Ok(t) => {
                tx.commit().map_err(|e| {
                    error!("[SqliteUnitOfWork] 提交事务失败: {}", e);
                    e.to_string()
                })?;
                Ok(t)
            }
            Err(e) => {
                let _ = tx.rollback();
                Err(e)
            }
        }
    }
}

struct SqliteBillTransactionContext<'a> {
    tx: &'a rusqlite::Transaction<'a>,
}

impl BillTransactionContext for SqliteBillTransactionContext<'_> {
    fn create_bill(&mut self, bill: &CreateBill) -> Result<i64, String> {
        self.tx
            .execute(
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
                error!("[SqliteUnitOfWork] INSERT bill 失败: {}", e);
                format!("创建账单失败: {}", e)
            })?;
        Ok(self.tx.last_insert_rowid())
    }

    fn create_bill_image(
        &mut self,
        bill_id: i64,
        image_path: &str,
        ocr_raw_text: &str,
    ) -> Result<(), String> {
        self.tx
            .execute(
                "INSERT INTO bill_image (bill_id, image_path, ocr_raw_text) VALUES (?1, ?2, ?3)",
                params![bill_id, image_path, ocr_raw_text],
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] INSERT bill_image 失败: {}", e);
                format!("保存图片记录失败: {}", e)
            })?;
        Ok(())
    }
}
