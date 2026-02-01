use crate::domain::entities::bill::BillFilters;
use crate::domain::ports::bill_repository::BillRepository;
use crate::domain::ports::category_repository::CategoryRepository;
use crate::domain::services::statistics_calculator::{CategoryStat, StatisticsCalculator};
use log::info;
use std::collections::HashMap;

/// Statistics result DTO (no serde; converted to command DTO in command layer).
pub struct StatisticsResult {
    pub total_income: f64,
    pub total_expense: f64,
    pub balance: f64,
    pub monthly_data: Vec<MonthlyDataItem>,
    pub category_data: Vec<CategoryDataItem>,
}

pub struct MonthlyDataItem {
    pub month: String,
    pub income: f64,
    pub expense: f64,
}

pub struct CategoryDataItem {
    pub category_id: i64,
    pub category_name: String,
    pub amount: f64,
    pub percentage: f64,
}

pub struct StatisticsAppService {
    bill_repo: Box<dyn BillRepository>,
    category_repo: Box<dyn CategoryRepository>,
    calculator: Box<dyn StatisticsCalculator>,
}

impl StatisticsAppService {
    pub fn new(
        bill_repo: Box<dyn BillRepository>,
        category_repo: Box<dyn CategoryRepository>,
        calculator: Box<dyn StatisticsCalculator>,
    ) -> Self {
        Self {
            bill_repo,
            category_repo,
            calculator,
        }
    }

    pub fn get_statistics(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<StatisticsResult, String> {
        info!("[StatisticsAppService] get_statistics");

        let filters = Some(BillFilters {
            member_id: None,
            category_id: None,
            start_date: start_date.clone(),
            end_date: end_date.clone(),
        });

        let bills = self.bill_repo.list_with_filters(filters)?;
        let (total_income, total_expense) = self.calculator.total_income_expense(&bills);
        let balance = total_income - total_expense;

        let monthly_data: Vec<MonthlyDataItem> = self
            .calculator
            .monthly(&bills)
            .into_iter()
            .map(|m| MonthlyDataItem {
                month: m.month,
                income: m.income,
                expense: m.expense,
            })
            .collect();

        let by_category: Vec<CategoryStat> = self.calculator.by_category(&bills);
        let category_names: HashMap<i64, String> = self
            .category_repo
            .list_all()?
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect();
        let total_expense_for_pct = if total_expense > 0.0 { total_expense } else { 1.0 };
        let category_data: Vec<CategoryDataItem> = by_category
            .into_iter()
            .map(|c| {
                let percentage = (c.amount / total_expense_for_pct) * 100.0;
                let category_name = category_names
                    .get(&c.category_id)
                    .cloned()
                    .unwrap_or_else(|| format!("分类#{}", c.category_id));
                CategoryDataItem {
                    category_id: c.category_id,
                    category_name,
                    amount: c.amount,
                    percentage,
                }
            })
            .collect();

        Ok(StatisticsResult {
            total_income,
            total_expense,
            balance,
            monthly_data,
            category_data,
        })
    }
}
