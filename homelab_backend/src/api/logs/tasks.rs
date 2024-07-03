use sqlx::{Pool, Sqlite};

pub struct LogCreationTask {
    method: String,
    uri: String,
    user_id: i64,
    date_time: i64,
}

impl LogCreationTask {
    pub fn new(method: String, uri: String, user_id: i64, date_time: i64) -> Self {
        Self {
            method,
            uri,
            user_id,
            date_time,
        }
    }
    pub async fn run_task(&self, pool: &Pool<Sqlite>) {
        let _res = sqlx::query!(
            "INSERT INTO logs (method, uri, user_id, date_time) VALUES (?, ?, ?, ?)",
            self.method,
            self.uri,
            self.user_id,
            self.date_time
        )
        .execute(pool)
        .await;
    }
}
