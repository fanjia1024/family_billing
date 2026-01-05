use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Family {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFamily {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFamily {
    pub id: i64,
    pub name: String,
}

