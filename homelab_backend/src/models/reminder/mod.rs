pub mod reminder_db;

use super::deserialize_id;
use mongodb::bson::Bson;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    VeryHigh = 3,
}

fn deserialize_from_mongo<'de, D>(deserializer: D) -> Result<Priority, D::Error>
where
    D: Deserializer<'de>,
{
    let bson = Bson::deserialize(deserializer)?;
    match bson {
        Bson::Int64(value) => Ok(Priority::from(value)),
        Bson::String(value) => match value.as_str() {
            "Low" => Ok(Priority::Low),
            "Medium" => Ok(Priority::Medium),
            "High" => Ok(Priority::High),
            "VeryHigh" => Ok(Priority::VeryHigh),
            _ => Err(serde::de::Error::custom(
                "Got an unexpected value when deserializing a Priority",
            )),
        },
        _ => Err(serde::de::Error::custom(
            "Got an unexpected value when deserializing a Priority",
        )),
    }
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
    #[serde(rename = "_id", deserialize_with = "deserialize_id")]
    pub id: String,
    pub slug: String,
    pub name: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReminderSchema {
    name: String,
    description: String,
    categories: Vec<String>,
    priority: Priority,
}

#[derive(Serialize, Deserialize)]
pub struct ReminderUpdateSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub priority: Option<Priority>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Reminder {
    #[serde(rename = "_id", deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub description: String,
    pub categories: Vec<String>,
    #[serde(deserialize_with = "deserialize_from_mongo")]
    pub priority: Priority,
    pub user_id: String,
    pub date_time: i64,
}
