pub mod jwt;
pub mod user_db;

use mongodb::bson::Bson;
use serde::{Deserialize, Deserializer, Serialize};

use super::deserialize_id;

#[derive(Serialize, PartialEq, Debug)]
pub enum AuthLevel {
    User,
    Admin,
}

fn deserialize<'de, D>(deserializer: D) -> Result<AuthLevel, D::Error>
where
    D: Deserializer<'de>,
{
    let bson = Bson::deserialize(deserializer)?;
    if let Bson::Int64(value) = bson {
        Ok(AuthLevel::from(value))
    } else {
        Err(serde::de::Error::custom("Expected an Int64 while deserializing AuthLevel"))
    }
}

impl From<AuthLevel> for Bson {
    fn from(value: AuthLevel) -> Self {
        match value {
            AuthLevel::User => Bson::Int64(0),
            AuthLevel::Admin => Bson::Int64(1),
        }
    }
}

impl From<Bson> for AuthLevel {
    fn from(value: Bson) -> Self {
        match value {
            Bson::Int64(int) => AuthLevel::from(int),
            _ => panic!("Tried to deserialize an AuthLevel that was stored as a non-int"),
        }
    }
}

impl From<i64> for AuthLevel {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::User,
            1 => Self::Admin,
            _ => panic!("Unsupported value when converting an i64 to an AuthLevel"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id", deserialize_with = "deserialize_id")]
    id: String,
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize")]
    pub auth_level: AuthLevel,
    pub salt: String,
}

impl User {
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReturnUser {
    pub id: String,
    pub username: String,
}

impl From<User> for ReturnUser {
    fn from(value: User) -> Self {
        Self {
            id: value.get_id(),
            username: value.username,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub username: String,
    pub password: String,
}
