use crate::domain::entities::bill::Bill;
use crate::domain::entities::category::Category;

/// Port: expense/bill analysis (e.g. classify, summarize).
/// Infrastructure may provide rule-based or LLM-based implementations.
/// Declared only; not implemented yet.
pub trait ExpenseAnalyzer: Send + Sync {
    /// Classify a bill into a category (e.g. by description or OCR text).
    fn classify(&self, _bill: &Bill) -> Option<Category> {
        None
    }

    /// Optional: summarize a set of bills (e.g. for reports).
    #[allow(dead_code)]
    fn summarize(&self, _bills: &[Bill]) -> Option<String> {
        None
    }
}
