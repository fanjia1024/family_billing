use crate::domain::entities::bill::{
    Bill, BillFilters, BillImageForExport, CreateBill, UpdateBill,
};

/// Port: persistence for bills. Implemented by Infrastructure (e.g. SqliteBillRepository).
/// Only get/put data; no aggregation (sum/group by) here.
pub trait BillRepository: Send + Sync {
    fn list_with_filters(&self, filters: Option<BillFilters>) -> Result<Vec<Bill>, String>;
    fn create(&self, bill: &CreateBill) -> Result<i64, String>;
    fn update(&self, id: i64, bill: &UpdateBill) -> Result<(), String>;
    fn delete(&self, id: i64) -> Result<(), String>;
    /// Persist a bill_image row linked to a bill (for OCR save flow).
    fn create_bill_image(&self, bill_id: i64, image_path: &str, ocr_raw_text: &str)
        -> Result<(), String>;
    /// Create bill and bill_image in a single transaction (for save_bill_with_ocr).
    fn create_bill_with_image(
        &self,
        bill: &CreateBill,
        image_path: &str,
        ocr_raw_text: &str,
    ) -> Result<i64, String>;
    /// List all bill_images for export.
    fn list_bill_images(&self) -> Result<Vec<BillImageForExport>, String>;
}
