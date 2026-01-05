use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bill {
    pub id: i64,
    pub member_id: i64,
    pub category_id: i64,
    pub r#type: String,
    pub amount: f64,
    pub description: Option<String>,
    pub source: String,
    pub bill_date: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBill {
    pub member_id: i64,
    pub category_id: i64,
    pub r#type: String,
    pub amount: f64,
    pub description: Option<String>,
    pub source: String,
    pub bill_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBill {
    pub member_id: Option<i64>,
    pub category_id: Option<i64>,
    pub r#type: Option<String>,
    pub amount: Option<f64>,
    pub description: Option<String>,
    pub bill_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillFilters {
    pub member_id: Option<i64>,
    pub category_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

