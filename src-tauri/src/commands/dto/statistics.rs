use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsDto {
    pub total_income: f64,
    pub total_expense: f64,
    pub balance: f64,
    pub monthly_data: Vec<MonthlyDataDto>,
    pub category_data: Vec<CategoryDataDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyDataDto {
    pub month: String,
    pub income: f64,
    pub expense: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryDataDto {
    pub category_id: i64,
    pub category_name: String,
    pub amount: f64,
    pub percentage: f64,
}
