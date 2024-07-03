mod db;
pub mod tasks;

use super::get_user_from_token;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header::HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Deserialize, Serialize)]
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
        .route("/logs", get(get_logs))
        .route("/logs/:log_id", get(get_log_by_id))
}

async fn get_logs(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Vec<Log>>), (StatusCode, Json<String>)> {
    // TODO: This should optionally return all logs if the requester provides a flag for "all logs" and is an admin
    let user = match get_user_from_token(
        &app_state.db,
        &headers,
        app_state.config.app_secret.as_str(),
    )
    .await
    {
        Ok(user) => user,
        Err(_e) => {
            return Err((
                StatusCode::FORBIDDEN,
                Json("Invalid or missing authentication".to_string()),
            ))
        }
    };
    match db::db_get_logs_for_user(&app_state.db, user.get_id()).await {
        Some(res) => Ok((StatusCode::OK, Json(res))),
        None => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal error while accessing database".to_string()),
        )),
    }
}

async fn get_log_by_id(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(log_id): Path<i64>,
) -> Result<(StatusCode, Json<Log>), (StatusCode, Json<String>)> {
    // This match is just made for auth, we don't care about the user
    match get_user_from_token(
        &app_state.db,
        &headers,
        app_state.config.app_secret.as_str(),
    )
    .await
    {
        Ok(_) => match db::db_get_log_by_id(&app_state.db, log_id).await {
            Some(log) => Ok((StatusCode::OK, Json(log))),
            None => Err((
                StatusCode::NOT_FOUND,
                Json("Log with given id was not found".to_string()),
            )),
        },
        Err(_e) => {
            return Err((
                StatusCode::FORBIDDEN,
                Json("Invalid or missing authentication".to_string()),
            ))
        }
    }
}
