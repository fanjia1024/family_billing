#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub icon: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateCategory {
    pub name: String,
    pub r#type: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateCategory {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
}
