use crate::models::user::{validation::UpdateUserSchema, ReturnUser};
use crate::testing::helpers::{delete_request, get_request, post_request, put_request};
use crate::testing::TestHelper;
use axum::http::StatusCode;
use serde_json::json;

pub async fn create_user(test_helper: &TestHelper, username: &str, password: &str) -> Result<String, (StatusCode, String)> {
    let data = json!({"username": username, "password": password});
    post_request(test_helper, "/users", data, None).await
}

pub async fn auth_user(test_helper: &TestHelper, username: &str, password: &str) -> Result<String, (StatusCode, String)> {
    let data = json!({"username": username, "password": password});
    post_request(test_helper, "/users/auth", data, None).await
}

pub async fn update_user(test_helper: &TestHelper, token: &str, update_data: UpdateUserSchema) -> Result<ReturnUser, (StatusCode, String)> {
    put_request(test_helper, "/users/me", update_data, token).await
}

pub async fn get_user_me(test_helper: &TestHelper, token: &str) -> Result<ReturnUser, (StatusCode, String)> {
    get_request(test_helper, "/users/me", token).await
}

pub async fn delete_user_me(test_helper: &TestHelper, token: &str) -> Result<(), (StatusCode, String)> {
    delete_request(test_helper, "/users/me", token).await
}
