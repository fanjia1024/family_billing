use crate::domain::entities::category::{Category, CreateCategory, UpdateCategory};

/// Port: category persistence. Implemented by Infrastructure.
pub trait CategoryRepository: Send + Sync {
    fn list_all(&self) -> Result<Vec<Category>, String>;
    fn create(&self, c: &CreateCategory) -> Result<i64, String>;
    fn update(&self, id: i64, c: &UpdateCategory) -> Result<(), String>;
    fn delete(&self, id: i64) -> Result<(), String>;
}
