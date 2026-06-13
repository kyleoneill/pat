use crate::models::log::Log;
use crate::testing::{TestHelper, helpers::get_request};
use axum::http::StatusCode;

pub async fn get_logs_for_user(test_helper: &TestHelper, token: &str) -> Result<Vec<Log>, (StatusCode, String)> {
    get_request(test_helper, "/logs", token).await
}
