/// One row from the `bill_image` table. Used only inside SqliteBillRepository.
#[derive(Debug, Clone)]
pub struct BillImageRow {
    pub id: i64,
    pub bill_id: i64,
    pub image_path: String,
    pub ocr_raw_text: Option<String>,
    pub created_at: String,
}
