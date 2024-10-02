pub mod jwt;
pub mod user_db;

use axum::body::Bytes;
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Deserialize, Type, Serialize, PartialEq)]
pub enum AuthLevel {
    User,
    Admin,
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

#[derive(Deserialize)]
pub struct User {
    id: i64,
    pub username: String,
    pub password: String,
    pub auth_level: AuthLevel,
    pub salt: String,
}

impl User {
    pub fn get_id(&self) -> i64 {
        self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReturnUser {
    pub id: i64,
    pub username: String,
}

impl From<User> for ReturnUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
        }
    }
}

impl ReturnUser {
    #[allow(dead_code)] // used in test
    pub fn from_bytes(input: &Bytes) -> Self {
        serde_json::from_slice(input).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub username: String,
    pub password: String,
}
