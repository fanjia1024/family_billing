use crate::domain::entities::family::{CreateFamily, Family, UpdateFamily};

pub trait FamilyRepository: Send + Sync {
    fn get_default(&self) -> Result<Option<Family>, String>;
    fn create(&self, c: &CreateFamily) -> Result<i64, String>;
    fn update(&self, id: i64, c: &UpdateFamily) -> Result<(), String>;
}
