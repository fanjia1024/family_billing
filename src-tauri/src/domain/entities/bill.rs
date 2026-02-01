// Domain entities for Bill (no serde; used by Application and Repository).
// Commands convert to/from models::bill (DTO) for frontend.

#[derive(Debug, Clone)]
pub struct Bill {
    pub id: i64,
    pub member_id: i64,
    pub category_id: i64,
    pub r#type: String,
    pub amount: f64,
    pub description: Option<String>,
    pub source: String,
    pub bill_date: String,
    pub bill_month: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateBill {
    pub member_id: i64,
    pub category_id: i64,
    pub r#type: String,
    pub amount: f64,
    pub description: Option<String>,
    pub source: String,
    pub bill_date: String,
    pub bill_month: String,
}

#[derive(Debug, Clone)]
pub struct UpdateBill {
    pub member_id: Option<i64>,
    pub category_id: Option<i64>,
    pub r#type: Option<String>,
    pub amount: Option<f64>,
    pub description: Option<String>,
    pub bill_date: Option<String>,
    pub bill_month: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BillFilters {
    pub member_id: Option<i64>,
    pub category_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
