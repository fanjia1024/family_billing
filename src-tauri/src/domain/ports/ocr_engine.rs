use crate::domain::entities::ocr::{OcrDetailResult, OcrResult};

/// Port: OCR recognition. Implemented by Infrastructure (e.g. TesseractOcrEngine).
pub trait OcrEngine: Send + Sync {
    fn recognize_image(&self, image_path: &str) -> Result<OcrResult, String>;
    fn recognize_bill_details(&self, image_path: &str) -> Result<OcrDetailResult, String>;
}
