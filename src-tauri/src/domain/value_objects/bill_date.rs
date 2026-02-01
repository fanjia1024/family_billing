//! Value objects for bill date and month. Validate YYYY-MM-DD and YYYY-MM.

use crate::domain::error::DomainError;

/// Bill date string in YYYY-MM-DD format (validated on parse).
#[derive(Debug, Clone)]
pub struct BillDate(String);

impl BillDate {
    /// Parse and validate YYYY-MM-DD. Returns error if empty or invalid format.
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(DomainError::InvalidDate("日期不能为空".to_string()));
        }
        if s.len() != 10 {
            return Err(DomainError::InvalidDate(format!(
                "日期格式必须为 YYYY-MM-DD，当前: {}",
                s
            )));
        }
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(DomainError::InvalidDate("日期格式必须为 YYYY-MM-DD".to_string()));
        }
        let _y: i32 = parts[0].parse().map_err(|_| {
            DomainError::InvalidDate("年份必须是数字".to_string())
        })?;
        let m: u32 = parts[1].parse().map_err(|_| {
            DomainError::InvalidDate("月份必须是数字".to_string())
        })?;
        let _d: u32 = parts[2].parse().map_err(|_| {
            DomainError::InvalidDate("日必须是数字".to_string())
        })?;
        if m == 0 || m > 12 {
            return Err(DomainError::InvalidDate("月份必须在 1-12".to_string()));
        }
        Ok(BillDate(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Bill month string in YYYY-MM format (validated on parse).
#[derive(Debug, Clone)]
pub struct BillMonth(String);

impl BillMonth {
    /// Parse and validate YYYY-MM. If empty, can be derived from BillDate (first 7 chars).
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(DomainError::InvalidDate("月份不能为空".to_string()));
        }
        if s.len() != 7 {
            return Err(DomainError::InvalidDate(format!(
                "月份格式必须为 YYYY-MM，当前: {}",
                s
            )));
        }
        if s.chars().nth(4) != Some('-') {
            return Err(DomainError::InvalidDate("月份格式必须为 YYYY-MM".to_string()));
        }
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(DomainError::InvalidDate("月份格式必须为 YYYY-MM".to_string()));
        }
        let _y: i32 = parts[0].parse().map_err(|_| {
            DomainError::InvalidDate("年份必须是数字".to_string())
        })?;
        let m: u32 = parts[1].parse().map_err(|_| {
            DomainError::InvalidDate("月份必须是数字".to_string())
        })?;
        if m == 0 || m > 12 {
            return Err(DomainError::InvalidDate("月份必须在 1-12".to_string()));
        }
        Ok(BillMonth(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
