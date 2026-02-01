use crate::domain::error::DomainError;

/// Domain rule: entity with dependent bills cannot be deleted. Application passes bill count.
pub trait DeletionPolicy: Send + Sync {
    fn allow_delete(&self, dependent_bill_count: usize) -> Result<(), DomainError>;
}

pub struct DefaultDeletionPolicy;

impl DefaultDeletionPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultDeletionPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl DeletionPolicy for DefaultDeletionPolicy {
    fn allow_delete(&self, dependent_bill_count: usize) -> Result<(), DomainError> {
        if dependent_bill_count > 0 {
            return Err(DomainError::HasDependentBills(dependent_bill_count as u32));
        }
        Ok(())
    }
}
