use crate::models::user::ReturnUser;
use crate::testing::helpers::{delete_request, get_request, post_request};
use axum::body::Body;
use axum::http::StatusCode;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde_json::json;
use std::net::SocketAddr;

pub async fn create_user(
    client: &Client<HttpConnector, Body>,
    username: &str,
    password: &str,
    addr: &SocketAddr,
) -> Result<ReturnUser, (StatusCode, String)> {
    let data = json!({"username": username, "password": password});
    post_request(client, "/users", data, None, addr).await
}

pub async fn auth_user(
    client: &Client<HttpConnector, Body>,
    username: &str,
    password: &str,
    addr: &SocketAddr,
) -> Result<String, (StatusCode, String)> {
    let data = json!({"username": username, "password": password});
    post_request(client, "/users/auth", data, None, addr).await
}

pub async fn get_user_me(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
) -> Result<ReturnUser, (StatusCode, String)> {
    get_request(client, "/users/me", token, addr).await
}

pub async fn delete_user_me(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
) -> Result<(), (StatusCode, String)> {
    delete_request(client, "/users/me", token, addr).await
}
