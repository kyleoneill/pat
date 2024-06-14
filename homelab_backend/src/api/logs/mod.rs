mod db;

use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header::HeaderMap, StatusCode},
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Log {
    id: i64,
    method: String,
    uri: String,
    user_id: i64,
    date_time: i64,
}

// TODO: Should have a log retention policy/task which auto deletes old tasks.
//       Tasks older than a configurable age will be auto deleted by a task.
//       Might be fine without this for awhile with an app only being used by myself

pub fn log_routes() -> Router<AppState> {
    Router::<AppState>::new()
    //.route("/logs/:log_id", get(get_log_by_id))
}

#[allow(dead_code, unused_variables)]
async fn get_log_by_id(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(log_id): Path<i64>,
) -> Result<(StatusCode, Json<Log>), (StatusCode, Json<String>)> {
    todo!()
}

pub async fn create_log(pool: &SqlitePool, method: String, uri: String, user_id: i64) {
    let date_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    db::create_log(pool, method, uri, user_id, date_time).await;
}
