#[derive(Debug, Clone)]
pub struct Family {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateFamily {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct UpdateFamily {
    pub id: i64,
    pub name: String,
}
