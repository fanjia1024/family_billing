use crate::domain::entities::category::{Category, CreateCategory, UpdateCategory};
use crate::domain::error::DomainError;

/// Port: category persistence. Implemented by Infrastructure.
pub trait CategoryRepository: Send + Sync {
    fn list_all(&self) -> Result<Vec<Category>, DomainError>;
    fn create(&self, c: &CreateCategory) -> Result<i64, DomainError>;
    fn update(&self, id: i64, c: &UpdateCategory) -> Result<(), DomainError>;
    fn delete(&self, id: i64) -> Result<(), DomainError>;
}
