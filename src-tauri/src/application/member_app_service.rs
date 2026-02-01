use crate::domain::entities::bill::BillFilters;
use crate::domain::entities::family::CreateFamily;
use crate::domain::entities::member::{CreateMember, Member, UpdateMember};
use crate::domain::ports::bill_repository::BillRepository;
use crate::domain::ports::family_repository::FamilyRepository;
use crate::domain::ports::member_repository::MemberRepository;
use crate::domain::services::deletion_policy::DeletionPolicy;
use log::info;

pub struct MemberAppService {
    member_repo: Box<dyn MemberRepository>,
    family_repo: Box<dyn FamilyRepository>,
    bill_repo: Box<dyn BillRepository>,
    deletion_policy: Box<dyn DeletionPolicy>,
}

impl MemberAppService {
    pub fn new(
        member_repo: Box<dyn MemberRepository>,
        family_repo: Box<dyn FamilyRepository>,
        bill_repo: Box<dyn BillRepository>,
        deletion_policy: Box<dyn DeletionPolicy>,
    ) -> Self {
        Self {
            member_repo,
            family_repo,
            bill_repo,
            deletion_policy,
        }
    }

    pub fn get_members(&self) -> Result<Vec<Member>, String> {
        info!("[MemberAppService] get_members");
        self.member_repo.list()
    }

    pub fn create_member(
        &self,
        name: String,
        role: String,
        avatar: Option<String>,
    ) -> Result<i64, String> {
        info!("[MemberAppService] create_member");
        let family_id = match self.family_repo.get_default()? {
            Some(f) => f.id,
            None => self.family_repo.create(&CreateFamily {
                name: "我的家庭".to_string(),
            })?,
        };
        self.member_repo
            .create(family_id, &CreateMember { name, role, avatar })
    }

    pub fn update_member(
        &self,
        id: i64,
        name: String,
        role: String,
        avatar: Option<String>,
    ) -> Result<(), String> {
        info!("[MemberAppService] update_member id={}", id);
        self.member_repo
            .update(id, &UpdateMember { id, name, role, avatar })
    }

    pub fn delete_member(&self, id: i64) -> Result<(), String> {
        info!("[MemberAppService] delete_member id={}", id);
        let bills = self.bill_repo.list_with_filters(Some(BillFilters {
            member_id: Some(id),
            category_id: None,
            start_date: None,
            end_date: None,
        }))?;
        self.deletion_policy
            .allow_delete(bills.len())
            .map_err(|e| e.to_string())?;
        self.member_repo.delete(id)
    }
}
