use crate::domain::entities::family::{CreateFamily, Family, UpdateFamily};
use crate::domain::ports::family_repository::FamilyRepository;
use log::info;

pub struct FamilyAppService {
    family_repo: Box<dyn FamilyRepository>,
}

impl FamilyAppService {
    pub fn new(family_repo: Box<dyn FamilyRepository>) -> Self {
        Self { family_repo }
    }

    pub fn get_family(&self) -> Result<Option<Family>, String> {
        info!("[FamilyAppService] get_family");
        self.family_repo.get_default()
    }

    pub fn create_family(&self, name: String) -> Result<i64, String> {
        info!("[FamilyAppService] create_family");
        if self.family_repo.get_default()?.is_some() {
            return Err("家庭已存在".to_string());
        }
        self.family_repo.create(&CreateFamily { name })
    }

    pub fn update_family(&self, id: i64, name: String) -> Result<(), String> {
        info!("[FamilyAppService] update_family id={}", id);
        self.family_repo.update(id, &UpdateFamily { id, name })
    }

    /// Returns existing family id or creates default "我的家庭" and returns its id.
    pub fn get_or_create_default_family_id(&self) -> Result<i64, String> {
        if let Some(f) = self.family_repo.get_default()? {
            return Ok(f.id);
        }
        self.family_repo.create(&CreateFamily {
            name: "我的家庭".to_string(),
        })
    }
}
