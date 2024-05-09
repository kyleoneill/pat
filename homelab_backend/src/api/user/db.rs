use sqlx::sqlite::SqliteQueryResult;
use sqlx::SqlitePool;
use crate::api::user::User;

pub async fn db_get_user_by_username(pool: &SqlitePool, username: &str) -> Option<User> {
    match sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", username)
        .fetch_one(pool)
        .await {
        Ok(user) => Some(user),
        // TODO: Actually handle this error
        Err(_) => None
    }
}

pub async fn db_delete_user(pool: &SqlitePool, user_id: i64) -> Result<SqliteQueryResult, ()> {
    match sqlx::query!(
        "DELETE FROM users WHERE id = ?",
        user_id
    ).execute(pool).await {
        // Should add actual error handling here, for now all we care about is whether or not
        // the query succeeded or failed
        Ok(res) => Ok(res),
        Err(_) => Err(())
    }
}

pub async fn db_get_user_by_id(pool: &SqlitePool, id: i64) -> Option<User> {
    match sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(pool)
        .await {
        Ok(user) => Some(user),
        // TODO: Actually handle this error
        Err(_) => None
    }
}
