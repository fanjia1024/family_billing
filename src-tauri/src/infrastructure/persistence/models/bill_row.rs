/// One row from the `bill` table. Used only inside SqliteBillRepository.
#[derive(Debug, Clone)]
pub struct BillRow {
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
