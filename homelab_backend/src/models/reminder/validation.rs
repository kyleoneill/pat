use crate::models::reminder::Priority;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateCategorySchema {
    pub slug: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateReminderSchema {
    pub name: String,
    pub description: String,
    pub categories: Vec<String>,
    pub priority: Priority,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateReminderSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub priority: Option<Priority>,
}
