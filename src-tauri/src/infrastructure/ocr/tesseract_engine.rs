// Infrastructure implementation of OcrEngine using Tesseract.
// Delegates to the existing OCR logic and maps results to domain types.

use crate::domain::entities::ocr::{BillItem, OcrDetailResult, OcrResult};
use crate::domain::error::DomainError;
use crate::domain::ports::ocr_engine::OcrEngine;
use crate::services::ocr_service;

pub struct TesseractOcrEngine;

impl TesseractOcrEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TesseractOcrEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OcrEngine for TesseractOcrEngine {
    fn recognize_image(&self, image_path: &str) -> Result<OcrResult, DomainError> {
        let r = ocr_service::recognize_image(image_path)
            .map_err(|e| DomainError::OcrFailed(e.to_string()))?;
        Ok(OcrResult {
            r#type: r.r#type,
            amount: r.amount,
            description: r.description,
            bill_date: r.bill_date,
            raw_text: r.raw_text,
        })
    }

    fn recognize_bill_details(&self, image_path: &str) -> Result<OcrDetailResult, DomainError> {
        let r = ocr_service::recognize_bill_details(image_path)
            .map_err(|e| DomainError::OcrFailed(e.to_string()))?;
        Ok(OcrDetailResult {
            items: r
                .items
                .into_iter()
                .map(|i| BillItem {
                    category: i.category,
                    amount: i.amount,
                    percentage: i.percentage,
                    bill_type: i.bill_type,
                })
                .collect(),
            total_amount: r.total_amount,
            raw_text: r.raw_text,
            bill_date: r.bill_date,
        })
    }
}
