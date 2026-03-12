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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
}
