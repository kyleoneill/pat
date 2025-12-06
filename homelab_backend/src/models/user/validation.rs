use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserSchema {
    pub username: Option<String>,
    pub password: Option<String>,
}
