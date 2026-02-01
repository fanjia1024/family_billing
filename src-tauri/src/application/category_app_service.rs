use crate::domain::entities::bill::BillFilters;
use crate::domain::entities::category::{Category, CreateCategory, UpdateCategory};
use crate::domain::ports::bill_repository::BillRepository;
use crate::domain::ports::category_repository::CategoryRepository;
use log::info;

pub struct CategoryAppService {
    category_repo: Box<dyn CategoryRepository>,
    bill_repo: Box<dyn BillRepository>,
}

impl CategoryAppService {
    pub fn new(
        category_repo: Box<dyn CategoryRepository>,
        bill_repo: Box<dyn BillRepository>,
    ) -> Self {
        Self {
            category_repo,
            bill_repo,
        }
    }

    pub fn get_categories(&self) -> Result<Vec<Category>, String> {
        info!("[CategoryAppService] get_categories");
        self.category_repo.list_all()
    }

    pub fn create_category(&self, name: String, r#type: String, icon: Option<String>) -> Result<i64, String> {
        info!("[CategoryAppService] create_category");
        let existing = self.category_repo.list_all()?;
        if existing.iter().any(|c| c.name == name && c.r#type == r#type) {
            return Err("该分类已存在".to_string());
        }
        self.category_repo.create(&CreateCategory { name, r#type, icon })
    }

    pub fn update_category(&self, id: i64, name: String, icon: Option<String>) -> Result<(), String> {
        info!("[CategoryAppService] update_category id={}", id);
        self.category_repo.update(id, &UpdateCategory { id, name, icon })
    }

    pub fn delete_category(&self, id: i64) -> Result<(), String> {
        info!("[CategoryAppService] delete_category id={}", id);
        let bills = self.bill_repo.list_with_filters(Some(BillFilters {
            member_id: None,
            category_id: Some(id),
            start_date: None,
            end_date: None,
        }))?;
        if !bills.is_empty() {
            return Err(format!(
                "该分类有 {} 条关联账单，请先删除账单或更改账单分类",
                bills.len()
            ));
        }
        self.category_repo.delete(id)
    }
}
