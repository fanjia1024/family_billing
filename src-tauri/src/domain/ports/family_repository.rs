use crate::domain::entities::family::{CreateFamily, Family, UpdateFamily};
use crate::domain::error::DomainError;

pub trait FamilyRepository: Send + Sync {
    fn get_default(&self) -> Result<Option<Family>, DomainError>;
    fn create(&self, c: &CreateFamily) -> Result<i64, DomainError>;
    fn update(&self, id: i64, c: &UpdateFamily) -> Result<(), DomainError>;
}
