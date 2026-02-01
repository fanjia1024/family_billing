use crate::domain::entities::family::Family;
use crate::domain::error::DomainError;

/// Domain rule: at most one family (singleton). Application calls after get_default().
pub trait FamilyValidator: Send + Sync {
    fn allow_create(&self, existing: Option<&Family>) -> Result<(), DomainError>;
}

pub struct DefaultFamilyValidator;

impl DefaultFamilyValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultFamilyValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl FamilyValidator for DefaultFamilyValidator {
    fn allow_create(&self, existing: Option<&Family>) -> Result<(), DomainError> {
        if existing.is_some() {
            return Err(DomainError::FamilyAlreadyExists);
        }
        Ok(())
    }
}
