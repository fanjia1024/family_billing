use crate::domain::entities::member::{CreateMember, Member, UpdateMember};
use crate::domain::error::DomainError;

pub trait MemberRepository: Send + Sync {
    fn list(&self) -> Result<Vec<Member>, DomainError>;
    fn create(&self, family_id: i64, c: &CreateMember) -> Result<i64, DomainError>;
    fn update(&self, id: i64, c: &UpdateMember) -> Result<(), DomainError>;
    fn delete(&self, id: i64) -> Result<(), DomainError>;
}
