use crate::domain::entities::bill::CreateBill;

/// Context for bill + bill_image operations within a single transaction.
/// Implemented by Infrastructure (e.g. SqliteBillTransactionContext).
pub trait BillTransactionContext {
    fn create_bill(&mut self, bill: &CreateBill) -> Result<i64, String>;
    fn create_bill_image(
        &mut self,
        bill_id: i64,
        image_path: &str,
        ocr_raw_text: &str,
    ) -> Result<(), String>;
}

/// Port: run a block of work (bill + bill_image) in a single transaction.
/// Implemented by Infrastructure (e.g. SqliteUnitOfWork).
pub trait UnitOfWork: Send + Sync {
    fn run_bill_with_image<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut dyn BillTransactionContext) -> Result<T, String>;
}
