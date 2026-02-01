use crate::domain::entities::bill::Bill;
use std::collections::HashMap;

/// Domain result for monthly aggregation (no serde; used by Application).
#[derive(Debug, Clone)]
pub struct MonthlyStat {
    pub month: String,
    pub income: f64,
    pub expense: f64,
}

/// Domain result for category aggregation (no serde; used by Application).
#[derive(Debug, Clone)]
pub struct CategoryStat {
    pub category_id: i64,
    pub amount: f64,
}

/// Port: pure domain logic for statistics. Repository only fetches bills; this does sum/group.
pub trait StatisticsCalculator: Send + Sync {
    fn total_income_expense(&self, bills: &[Bill]) -> (f64, f64);
    fn monthly(&self, bills: &[Bill]) -> Vec<MonthlyStat>;
    fn by_category(&self, bills: &[Bill]) -> Vec<CategoryStat>;
}

/// In-memory implementation: iterate bills and aggregate.
pub struct DefaultStatisticsCalculator;

impl DefaultStatisticsCalculator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultStatisticsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsCalculator for DefaultStatisticsCalculator {
    fn total_income_expense(&self, bills: &[Bill]) -> (f64, f64) {
        let mut income = 0.0;
        let mut expense = 0.0;
        for b in bills {
            if b.r#type == "income" {
                income += b.amount;
            } else {
                expense += b.amount;
            }
        }
        (income, expense)
    }

    fn monthly(&self, bills: &[Bill]) -> Vec<MonthlyStat> {
        let mut by_month: HashMap<String, (f64, f64)> = HashMap::new();
        for b in bills {
            let month = b
                .bill_month
                .clone()
                .unwrap_or_else(|| b.bill_date.chars().take(7).collect::<String>());
            let entry = by_month.entry(month).or_insert((0.0, 0.0));
            if b.r#type == "income" {
                entry.0 += b.amount;
            } else {
                entry.1 += b.amount;
            }
        }
        let mut result: Vec<MonthlyStat> = by_month
            .into_iter()
            .map(|(month, (income, expense))| MonthlyStat {
                month,
                income,
                expense,
            })
            .collect();
        result.sort_by(|a, b| a.month.cmp(&b.month));
        result
    }

    fn by_category(&self, bills: &[Bill]) -> Vec<CategoryStat> {
        let mut by_cat: HashMap<i64, f64> = HashMap::new();
        for b in bills {
            if b.r#type == "expense" {
                *by_cat.entry(b.category_id).or_insert(0.0) += b.amount;
            }
        }
        let mut result: Vec<CategoryStat> = by_cat
            .into_iter()
            .map(|(category_id, amount)| CategoryStat {
                category_id,
                amount,
            })
            .collect();
        result.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal));
        result
    }
}
