use crate::models::log::Log;
use crate::testing::helpers::read_error_message;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use std::net::SocketAddr;

pub async fn get_logs_for_user(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
) -> Result<Vec<Log>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/logs"))
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::empty())
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::OK => (),
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            return Err((status, message));
        }
    };
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(Log::from_bytes_to_vec(&body))
}
