use crate::domain::entities::member::{CreateMember, Member, UpdateMember};

pub trait MemberRepository: Send + Sync {
    fn list(&self) -> Result<Vec<Member>, String>;
    fn create(&self, family_id: i64, c: &CreateMember) -> Result<i64, String>;
    fn update(&self, id: i64, c: &UpdateMember) -> Result<(), String>;
    fn delete(&self, id: i64) -> Result<(), String>;
}
