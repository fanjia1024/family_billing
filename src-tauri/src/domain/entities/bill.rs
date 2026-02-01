// Domain entities for Bill (no serde; used by Application and Repository).
// Commands convert to/from commands::dto::bill for frontend.

use crate::domain::error::DomainError;
use crate::domain::value_objects::bill_date::{BillDate, BillMonth};
use crate::domain::value_objects::money::Money;

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

impl CreateBill {
    /// Validate and build CreateBill. Returns DomainError on invalid amount, date, or type.
    pub fn try_new(
        member_id: i64,
        category_id: i64,
        r#type: String,
        amount: f64,
        description: Option<String>,
        source: String,
        bill_date: String,
        bill_month: String,
    ) -> Result<Self, DomainError> {
        if member_id <= 0 {
            return Err(DomainError::InvalidMemberId);
        }
        if category_id <= 0 {
            return Err(DomainError::InvalidCategoryId);
        }
        let money = Money::try_new(amount)?;
        if !money.is_positive() {
            return Err(DomainError::InvalidAmount);
        }
        let bill_date_vo = BillDate::parse(&bill_date)?;
        let bill_month_str = if bill_month.trim().is_empty() {
            bill_date_vo.as_str().chars().take(7).collect::<String>()
        } else {
            let m = BillMonth::parse(&bill_month)?;
            m.as_str().to_string()
        };
        let type_lower = r#type.to_lowercase();
        if type_lower != "income" && type_lower != "expense" {
            return Err(DomainError::InvalidBillType(
                "类型必须是 income 或 expense".to_string(),
            ));
        }
        Ok(CreateBill {
            member_id,
            category_id,
            r#type: type_lower,
            amount: money.as_f64(),
            description,
            source: if source.is_empty() { "manual".to_string() } else { source },
            bill_date: bill_date_vo.as_str().to_string(),
            bill_month: bill_month_str,
        })
    }
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

/// For export: one bill_image row (no id in domain).
#[derive(Debug, Clone)]
pub struct BillImageForExport {
    pub bill_id: i64,
    pub image_path: String,
    pub ocr_raw_text: Option<String>,
    pub created_at: String,
}
