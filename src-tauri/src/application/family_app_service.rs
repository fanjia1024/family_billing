use crate::domain::entities::family::{CreateFamily, Family, UpdateFamily};
use crate::domain::error::DomainError;
use crate::domain::ports::family_repository::FamilyRepository;
use crate::domain::services::family_validator::FamilyValidator;
use log::info;

pub struct FamilyAppService {
    family_repo: Box<dyn FamilyRepository>,
    family_validator: Box<dyn FamilyValidator>,
}

impl FamilyAppService {
    pub fn new(
        family_repo: Box<dyn FamilyRepository>,
        family_validator: Box<dyn FamilyValidator>,
    ) -> Self {
        Self {
            family_repo,
            family_validator,
        }
    }

    pub fn get_family(&self) -> Result<Option<Family>, DomainError> {
        info!("[FamilyAppService] get_family");
        self.family_repo.get_default()
    }

    pub fn create_family(&self, name: String) -> Result<i64, DomainError> {
        info!("[FamilyAppService] create_family");
        let existing = self.family_repo.get_default()?;
        self.family_validator.allow_create(existing.as_ref())?;
        self.family_repo.create(&CreateFamily { name })
    }

    pub fn update_family(&self, id: i64, name: String) -> Result<(), DomainError> {
        info!("[FamilyAppService] update_family id={}", id);
        self.family_repo.update(id, &UpdateFamily { id, name })
    }

    /// Returns existing family id or creates default "我的家庭" and returns its id.
    pub fn get_or_create_default_family_id(&self) -> Result<i64, DomainError> {
        if let Some(f) = self.family_repo.get_default()? {
            return Ok(f.id);
        }
        self.family_repo.create(&CreateFamily {
            name: "我的家庭".to_string(),
        })
    }
}
