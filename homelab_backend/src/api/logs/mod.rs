mod db;
pub mod tasks;

use super::get_user_from_token;
use super::return_data::ReturnData;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::header::HeaderMap,
    routing::get,
    Router,
};
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct Log {
    id: i64,
    pub method: String,
    pub uri: String,
    pub user_id: i64,
    date_time: i64,
}

impl Log {
    #[allow(dead_code)] // Used in test
    pub fn from_bytes_to_vec(input: &Bytes) -> Vec<Self> {
        serde_json::from_slice(input).unwrap()
    }
}

// TODO: Should have a log retention policy/task which auto deletes old tasks.
//       Tasks older than a configurable age will be auto deleted by a task.
//       Might be fine without this for awhile with an app only being used by myself

pub fn log_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/logs", get(get_logs))
        .route("/logs/:log_id", get(get_log_by_id))
}

async fn get_logs(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> ReturnData<Vec<Log>, String> {
    // TODO: This should optionally return all logs if the requester provides a flag for "all logs" and is an admin
    let pool = &app_state.db;
    let user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match db::db_get_logs_for_user(pool, user.get_id()).await {
        Some(res) => ReturnData::ok(res),
        None => ReturnData::internal_error("Internal error while accessing database".to_string()),
    }
}

async fn get_log_by_id(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(log_id): Path<i64>,
) -> ReturnData<Log, String> {
    let pool = &app_state.db;
    // Get the user here just to auth them, we don't actually need the user data
    let _user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match db::db_get_log_by_id(&app_state.db, log_id).await {
        Some(log) => ReturnData::ok(log),
        None => ReturnData::not_found("Log with given id was not found".to_string()),
    }
}
