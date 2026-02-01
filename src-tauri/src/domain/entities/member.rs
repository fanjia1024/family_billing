#[derive(Debug, Clone)]
pub struct Member {
    pub id: i64,
    pub family_id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateMember {
    pub name: String,
    pub role: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateMember {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub avatar: Option<String>,
}
