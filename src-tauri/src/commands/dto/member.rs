use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberDto {
    pub id: i64,
    pub family_id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMemberDto {
    pub name: String,
    pub role: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemberDto {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub avatar: Option<String>,
}
