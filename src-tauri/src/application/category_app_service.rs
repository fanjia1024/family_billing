use crate::domain::entities::bill::BillFilters;
use crate::domain::entities::category::{Category, CreateCategory, UpdateCategory};
use crate::domain::error::DomainError;
use crate::domain::ports::bill_repository::BillRepository;
use crate::domain::ports::category_repository::CategoryRepository;
use crate::domain::services::category_validator::CategoryValidator;
use crate::domain::services::deletion_policy::DeletionPolicy;
use log::info;

pub struct CategoryAppService {
    category_repo: Box<dyn CategoryRepository>,
    bill_repo: Box<dyn BillRepository>,
    category_validator: Box<dyn CategoryValidator>,
    deletion_policy: Box<dyn DeletionPolicy>,
}

impl CategoryAppService {
    pub fn new(
        category_repo: Box<dyn CategoryRepository>,
        bill_repo: Box<dyn BillRepository>,
        category_validator: Box<dyn CategoryValidator>,
        deletion_policy: Box<dyn DeletionPolicy>,
    ) -> Self {
        Self {
            category_repo,
            bill_repo,
            category_validator,
            deletion_policy,
        }
    }

    pub fn get_categories(&self) -> Result<Vec<Category>, DomainError> {
        info!("[CategoryAppService] get_categories");
        self.category_repo.list_all()
    }

    pub fn create_category(&self, name: String, r#type: String, icon: Option<String>) -> Result<i64, DomainError> {
        info!("[CategoryAppService] create_category");
        let existing = self.category_repo.list_all()?;
        self.category_validator.allow_create(&name, &r#type, &existing)?;
        self.category_repo.create(&CreateCategory { name, r#type, icon })
    }

    pub fn update_category(&self, id: i64, name: String, icon: Option<String>) -> Result<(), DomainError> {
        info!("[CategoryAppService] update_category id={}", id);
        self.category_repo.update(id, &UpdateCategory { id, name, icon })
    }

    pub fn delete_category(&self, id: i64) -> Result<(), DomainError> {
        info!("[CategoryAppService] delete_category id={}", id);
        let bills = self.bill_repo.list_with_filters(Some(BillFilters {
            member_id: None,
            category_id: Some(id),
            start_date: None,
            end_date: None,
        }))?;
        self.deletion_policy.allow_delete(bills.len())?;
        self.category_repo.delete(id)
    }
}
