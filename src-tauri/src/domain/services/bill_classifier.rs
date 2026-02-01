use crate::domain::entities::bill::Bill;
use crate::domain::entities::category::Category;

/// Domain service: classify a bill into a category (e.g. by description). Rule or LLM impl later.
pub trait BillClassifier: Send + Sync {
    fn classify(&self, bill: &Bill) -> Option<Category>;
}

/// Placeholder: no classification until rules or ExpenseAnalyzer are wired.
pub struct DefaultBillClassifier;

impl DefaultBillClassifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultBillClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl BillClassifier for DefaultBillClassifier {
    fn classify(&self, _bill: &Bill) -> Option<Category> {
        None
    }
}
