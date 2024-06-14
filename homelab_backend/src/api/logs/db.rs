use sqlx::SqlitePool;

pub(super) async fn create_log(
    pool: &SqlitePool,
    method: String,
    uri: String,
    user_id: i64,
    date_time: i64,
) {
    let _ = sqlx::query!(
        "INSERT INTO logs (method, uri, user_id, date_time) VALUES (?, ?, ?, ?)",
        method,
        uri,
        user_id,
        date_time
    )
    .execute(pool)
    .await;
}
