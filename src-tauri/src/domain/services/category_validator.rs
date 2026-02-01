use crate::domain::entities::category::Category;
use crate::domain::error::DomainError;

/// Domain rule: no duplicate (name + type). Application passes existing list from repo.
pub trait CategoryValidator: Send + Sync {
    fn allow_create(&self, name: &str, r#type: &str, existing: &[Category]) -> Result<(), DomainError>;
}

pub struct DefaultCategoryValidator;

impl DefaultCategoryValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultCategoryValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CategoryValidator for DefaultCategoryValidator {
    fn allow_create(&self, name: &str, r#type: &str, existing: &[Category]) -> Result<(), DomainError> {
        if existing.iter().any(|c| c.name == name && c.r#type == r#type) {
            return Err(DomainError::CategoryAlreadyExists);
        }
        Ok(())
    }
}
