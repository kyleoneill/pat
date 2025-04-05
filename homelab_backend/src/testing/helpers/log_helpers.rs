use crate::models::log::Log;
use crate::testing::helpers::get_request;
use axum::body::Body;
use axum::http::StatusCode;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use std::net::SocketAddr;

pub async fn get_logs_for_user(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
) -> Result<Vec<Log>, (StatusCode, String)> {
    get_request(client, "/logs", token, addr).await
}
