use crate::domain::entities::bill::{Bill, BillFilters, CreateBill, UpdateBill};
use crate::domain::entities::ocr::{OcrDetailResult, OcrResult};
use crate::domain::ports::bill_repository::BillRepository;
use crate::domain::ports::ocr_engine::OcrEngine;
use log::{debug, info};

/// Application service for bill use cases.
/// Depends only on domain ports (BillRepository, OcrEngine).
pub struct BillAppService {
    bill_repo: Box<dyn BillRepository>,
    ocr_engine: Box<dyn OcrEngine>,
}

impl BillAppService {
    pub fn new(bill_repo: Box<dyn BillRepository>, ocr_engine: Box<dyn OcrEngine>) -> Self {
        Self {
            bill_repo,
            ocr_engine,
        }
    }

    pub fn get_bills(&self, filters: Option<BillFilters>) -> Result<Vec<Bill>, String> {
        info!("[BillAppService] get_bills");
        self.bill_repo.list_with_filters(filters)
    }

    pub fn add_bill(&self, bill: CreateBill) -> Result<i64, String> {
        info!("[BillAppService] add_bill");
        self.bill_repo.create(&bill)
    }

    pub fn update_bill(&self, id: i64, bill: UpdateBill) -> Result<(), String> {
        info!("[BillAppService] update_bill id={}", id);
        self.bill_repo.update(id, &bill)
    }

    pub fn delete_bill(&self, id: i64) -> Result<(), String> {
        info!("[BillAppService] delete_bill id={}", id);
        self.bill_repo.delete(id)
    }

    /// Save a bill and optionally attach an image with OCR raw text.
    pub fn save_bill_with_ocr(&self, bill: CreateBill, image_path: &str) -> Result<i64, String> {
        info!("[BillAppService] save_bill_with_ocr");
        let bill_id = self.bill_repo.create(&bill)?;
        debug!("[BillAppService] bill created id={}", bill_id);

        let ocr_text = self
            .ocr_engine
            .recognize_image(image_path)
            .ok()
            .map(|r| r.raw_text)
            .unwrap_or_default();

        self.bill_repo
            .create_bill_image(bill_id, image_path, &ocr_text)?;
        Ok(bill_id)
    }

    pub fn recognize_image(&self, image_path: &str) -> Result<OcrResult, String> {
        info!("[BillAppService] recognize_image");
        self.ocr_engine.recognize_image(image_path)
    }

    pub fn recognize_bill_details(&self, image_path: &str) -> Result<OcrDetailResult, String> {
        info!("[BillAppService] recognize_bill_details");
        self.ocr_engine.recognize_bill_details(image_path)
    }
}
