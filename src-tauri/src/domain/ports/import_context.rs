//! Port for import transaction: insert families, members, categories, bills, bill_images
//! in order within a single transaction. Implemented by Infrastructure (e.g. SqliteUnitOfWork).

use crate::domain::error::DomainError;

/// Context for import operations within a single transaction.
/// Used by ImportAppService to insert families → members → categories → bills → bill_images
/// with "skip if exists" semantics.
pub trait ImportTransactionContext {
    fn insert_family_if_not_exists(
        &mut self,
        id: i64,
        name: &str,
        created_at: &str,
    ) -> Result<(), DomainError>;

    fn insert_member_if_not_exists(
        &mut self,
        id: i64,
        family_id: i64,
        name: &str,
        avatar: Option<&str>,
        role: &str,
        created_at: &str,
    ) -> Result<(), DomainError>;

    fn insert_category_if_not_exists(
        &mut self,
        id: i64,
        name: &str,
        type_: &str,
        icon: Option<&str>,
        created_at: &str,
    ) -> Result<(), DomainError>;

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
    ) -> Result<(), DomainError>;

    fn insert_bill_image_if_not_exists(
        &mut self,
        bill_id: i64,
        image_path: &str,
        ocr_raw_text: &str,
        created_at: &str,
    ) -> Result<(), DomainError>;
}
