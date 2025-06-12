use std::sync::Arc;
use super::get_user_from_auth_header;
use super::return_data::ReturnData;
use crate::models::log::log_db::{db_get_log_by_id, db_get_logs_for_user};
use crate::models::log::Log;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::header::HeaderMap,
    routing::get,
    Router,
};

pub fn log_routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/logs", get(get_logs))
        .route("/logs/:log_id", get(get_log_by_id))
}

async fn get_logs(State(app_state): State<Arc<AppState>>, headers: HeaderMap) -> ReturnData<Vec<Log>> {
    // TODO: This has the potential to return a lot of data. There should be a hard limit for logs
    //       returned, and this endpoint should be paginated / sortable. Should have a query
    //       param to specify how many logs are returned.
    // TODO: This should optionally return all logs if the requester provides a flag for "all logs" and is an admin
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match db_get_logs_for_user(pool, user.get_id()).await {
        Ok(res) => ReturnData::ok(res),
        Err(e) => e.into(),
    }
}

async fn get_log_by_id(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(log_id): Path<String>,
) -> ReturnData<Log> {
    let pool = &app_state.db;
    // Get the user here just to auth them, we don't actually need the user data
    let _user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await
    {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match db_get_log_by_id(&app_state.db, log_id).await {
        Ok(log) => ReturnData::ok(log),
        Err(e) => e.into(),
    }
}
