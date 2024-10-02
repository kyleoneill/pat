use super::{AuthLevel, User};
use crate::error_handler::DbError;
use sqlx::{sqlite::SqliteQueryResult, Error, SqlitePool};

pub async fn db_create_user(
    pool: &SqlitePool,
    username: String,
    hash: String,
    auth_level: AuthLevel,
    salt: String,
) -> Result<(), DbError> {
    match sqlx::query!(
        "INSERT INTO users (username, password, auth_level, salt) VALUES (?, ?, ?, ?)",
        username,
        hash,
        auth_level,
        salt
    )
    .execute(pool)
    .await
    {
        Ok(_res) => Ok(()),
        // TODO: Handle this
        Err(_e) => Err(DbError::UnhandledException),
    }
}

pub async fn db_get_user_by_username(pool: &SqlitePool, username: &str) -> Result<User, DbError> {
    match sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", username)
        .fetch_one(pool)
        .await
    {
        Ok(user) => Ok(user),
        Err(e) => match e {
            Error::RowNotFound => Err(DbError::NotFound("user".to_owned(), username.to_owned())),
            _ => Err(DbError::UnhandledException),
        },
    }
}

pub async fn db_delete_user(pool: &SqlitePool, user_id: i64) -> Result<SqliteQueryResult, ()> {
    match sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(pool)
        .await
    {
        // Should add actual error handling here, for now all we care about is whether or not
        // the query succeeded or failed
        Ok(res) => Ok(res),
        Err(_) => Err(()),
    }
}

pub async fn db_get_user_by_id(pool: &SqlitePool, id: i64) -> Result<User, DbError> {
    match sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(pool)
        .await
    {
        Ok(user) => Ok(user),
        Err(e) => match e {
            Error::RowNotFound => Err(DbError::NotFound("user".to_owned(), id.to_string())),
            _ => Err(DbError::UnhandledException),
        },
    }
}
