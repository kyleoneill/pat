use super::Log;
use sqlx::SqlitePool;

// TODO: There should be actual error handling here

pub async fn db_get_logs_for_user(pool: &SqlitePool, user_id: i64) -> Option<Vec<Log>> {
    match sqlx::query_as!(Log, "SELECT * FROM logs WHERE user_id = ?", user_id)
        .fetch_all(pool)
        .await
    {
        Ok(res) => Some(res),
        // TODO: Actual error handling
        Err(_) => None,
    }
}

pub async fn db_get_log_by_id(pool: &SqlitePool, log_id: i64) -> Option<Log> {
    match sqlx::query_as!(Log, "SELECT * FROM logs WHERE id = ?", log_id)
        .fetch_one(pool)
        .await
    {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}
