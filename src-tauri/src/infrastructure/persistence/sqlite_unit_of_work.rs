use crate::domain::entities::bill::CreateBill;
use crate::domain::error::DomainError;
use crate::domain::ports::import_context::ImportTransactionContext;
use crate::domain::ports::unit_of_work::{BillTransactionContext, UnitOfWork};
use crate::infrastructure::persistence::database;
use log::error;
use rusqlite::params;
use tauri::AppHandle;

fn to_persistence(e: impl std::fmt::Display) -> DomainError {
    DomainError::PersistenceError(e.to_string())
}

pub struct SqliteUnitOfWork {
    app: AppHandle,
}

impl SqliteUnitOfWork {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl UnitOfWork for SqliteUnitOfWork {
    fn run_bill_with_image<F, T>(&self, f: F) -> Result<T, DomainError>
    where
        F: FnOnce(&mut dyn BillTransactionContext) -> Result<T, DomainError>,
    {
        let mut conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteUnitOfWork] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        let tx = conn.transaction().map_err(|e| {
            error!("[SqliteUnitOfWork] 开启事务失败: {}", e);
            to_persistence(e)
        })?;

        let mut ctx = SqliteBillTransactionContext { tx: &tx };
        let result = f(&mut ctx);

        match result {
            Ok(t) => {
                tx.commit().map_err(|e| {
                    error!("[SqliteUnitOfWork] 提交事务失败: {}", e);
                    to_persistence(e)
                })?;
                Ok(t)
            }
            Err(e) => {
                let _ = tx.rollback();
                Err(e)
            }
        }
    }

    fn run_import<F, T>(&self, f: F) -> Result<T, DomainError>
    where
        F: FnOnce(&mut dyn ImportTransactionContext) -> Result<T, DomainError>,
    {
        let mut conn = database::get_connection(&self.app).map_err(|e| {
            error!("[SqliteUnitOfWork] 数据库连接失败: {}", e);
            to_persistence(e)
        })?;

        let tx = conn.transaction().map_err(|e| {
            error!("[SqliteUnitOfWork] 开启导入事务失败: {}", e);
            to_persistence(e)
        })?;

        let mut ctx = SqliteImportTransactionContext { tx: &tx };
        let result = f(&mut ctx);

        match result {
            Ok(t) => {
                tx.commit().map_err(|e| {
                    error!("[SqliteUnitOfWork] 提交导入事务失败: {}", e);
                    to_persistence(e)
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
    fn create_bill(&mut self, bill: &CreateBill) -> Result<i64, DomainError> {
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
                to_persistence(e)
            })?;
        Ok(self.tx.last_insert_rowid())
    }

    fn create_bill_image(
        &mut self,
        bill_id: i64,
        image_path: &str,
        ocr_raw_text: &str,
    ) -> Result<(), DomainError> {
        self.tx
            .execute(
                "INSERT INTO bill_image (bill_id, image_path, ocr_raw_text) VALUES (?1, ?2, ?3)",
                params![bill_id, image_path, ocr_raw_text],
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] INSERT bill_image 失败: {}", e);
                to_persistence(e)
            })?;
        Ok(())
    }
}

struct SqliteImportTransactionContext<'a> {
    tx: &'a rusqlite::Transaction<'a>,
}

impl ImportTransactionContext for SqliteImportTransactionContext<'_> {
    fn insert_family_if_not_exists(
        &mut self,
        id: i64,
        name: &str,
        created_at: &str,
    ) -> Result<(), DomainError> {
        let exists: bool = self
            .tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM family WHERE id = ?1)",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] 检查家庭存在失败: {}", e);
                to_persistence(e)
            })?;
        if exists {
            return Ok(());
        }
        self.tx
            .execute(
                "INSERT INTO family (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, name, created_at],
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] INSERT family 失败: {}", e);
                to_persistence(e)
            })?;
        Ok(())
    }

    fn insert_member_if_not_exists(
        &mut self,
        id: i64,
        family_id: i64,
        name: &str,
        avatar: Option<&str>,
        role: &str,
        created_at: &str,
    ) -> Result<(), DomainError> {
        let exists: bool = self
            .tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM member WHERE id = ?1)",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] 检查成员存在失败: {}", e);
                to_persistence(e)
            })?;
        if exists {
            return Ok(());
        }
        self.tx
            .execute(
                "INSERT INTO member (id, family_id, name, avatar, role, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, family_id, name, avatar, role, created_at],
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] INSERT member 失败: {}", e);
                to_persistence(e)
            })?;
        Ok(())
    }

    fn insert_category_if_not_exists(
        &mut self,
        id: i64,
        name: &str,
        type_: &str,
        icon: Option<&str>,
        created_at: &str,
    ) -> Result<(), DomainError> {
        let exists: bool = self
            .tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM category WHERE id = ?1)",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] 检查分类存在失败: {}", e);
                to_persistence(e)
            })?;
        if exists {
            return Ok(());
        }
        self.tx
            .execute(
                "INSERT INTO category (id, name, type, icon, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, name, type_, icon, created_at],
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] INSERT category 失败: {}", e);
                to_persistence(e)
            })?;
        Ok(())
    }

    fn insert_bill_if_not_exists(
        &mut self,
        id: i64,
        member_id: i64,
        category_id: i64,
        type_: &str,
        amount: f64,
        description: Option<&str>,
        source: &str,
        bill_date: &str,
        bill_month: &str,
        created_at: &str,
    ) -> Result<(), DomainError> {
        let exists: bool = self
            .tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM bill WHERE id = ?1)",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] 检查账单存在失败: {}", e);
                to_persistence(e)
            })?;
        if exists {
            return Ok(());
        }
        self.tx
            .execute(
                "INSERT INTO bill (id, member_id, category_id, type, amount, description, source, bill_date, bill_month, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
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
                ],
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] INSERT bill 失败: {}", e);
                to_persistence(e)
            })?;
        Ok(())
    }

    fn insert_bill_image_if_not_exists(
        &mut self,
        bill_id: i64,
        image_path: &str,
        ocr_raw_text: &str,
        created_at: &str,
    ) -> Result<(), DomainError> {
        let exists: bool = self
            .tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM bill_image WHERE bill_id = ?1 AND image_path = ?2)",
                params![bill_id, image_path],
                |row| row.get(0),
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] 检查 bill_image 存在失败: {}", e);
                to_persistence(e)
            })?;
        if exists {
            return Ok(());
        }
        self.tx
            .execute(
                "INSERT INTO bill_image (bill_id, image_path, ocr_raw_text, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![bill_id, image_path, ocr_raw_text, created_at],
            )
            .map_err(|e| {
                error!("[SqliteUnitOfWork] INSERT bill_image 失败: {}", e);
                to_persistence(e)
            })?;
        Ok(())
    }
}
