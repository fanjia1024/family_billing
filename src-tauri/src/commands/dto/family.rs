use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FamilyDto {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFamilyDto {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFamilyDto {
    pub id: i64,
    pub name: String,
}
