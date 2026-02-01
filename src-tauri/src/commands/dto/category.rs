use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryDto {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub icon: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryDto {
    pub name: String,
    pub r#type: String,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCategoryDto {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
}
