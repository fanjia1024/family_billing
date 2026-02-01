// Domain types for OCR results (returned by OcrEngine port).

#[derive(Debug, Clone)]
pub struct OcrResult {
    pub r#type: String,
    pub amount: f64,
    pub description: String,
    pub bill_date: String,
    pub raw_text: String,
}

#[derive(Debug, Clone)]
pub struct BillItem {
    pub category: String,
    pub amount: f64,
    pub percentage: Option<f64>,
    pub bill_type: String,
}

#[derive(Debug, Clone)]
pub struct OcrDetailResult {
    pub items: Vec<BillItem>,
    pub total_amount: f64,
    pub raw_text: String,
    pub bill_date: String,
}
