pub mod reminder_db;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    VeryHigh = 3,
}

impl From<i64> for Priority {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::Low,
            1 => Self::Medium,
            2 => Self::High,
            3 => Self::VeryHigh,
            _ => panic!("Got an invalid i64 when converting an i64 to a Priority"),
        }
    }
}

impl From<Priority> for i64 {
    fn from(value: Priority) -> Self {
        value as i64
    }
}

#[derive(Deserialize)]
pub struct CategorySchema {
    slug: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub user_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ReminderSchema {
    name: String,
    description: String,
    categories: Vec<i64>,
    priority: Priority,
}

#[derive(Serialize, Deserialize)]
pub struct ReminderUpdateSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<i64>>,
    pub priority: Option<Priority>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Reminder {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub categories: Vec<i64>,
    pub priority: Priority,
    pub user_id: i64,
    pub date_time: i64,
}
