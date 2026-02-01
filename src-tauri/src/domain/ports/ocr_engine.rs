use crate::domain::entities::ocr::{OcrDetailResult, OcrResult};
use crate::domain::error::DomainError;

/// Port: OCR recognition. Implemented by Infrastructure (e.g. TesseractOcrEngine).
pub trait OcrEngine: Send + Sync {
    fn recognize_image(&self, image_path: &str) -> Result<OcrResult, DomainError>;
    fn recognize_bill_details(&self, image_path: &str) -> Result<OcrDetailResult, DomainError>;
}
