use crate::testing::ADDR;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use http_body_util::BodyExt;
use axum::body::Bytes;

pub async fn get_logs_for_user(
    client: &Client<HttpConnector, Body>,
    token: &str,
) -> Result<Bytes, (StatusCode, String)> {
    let addr = ADDR.get().unwrap();
    let req = Request::builder()
        .uri(format!("http://{addr}/api/logs"))
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(Body::empty()))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::OK => (),
        _ => {
            return Err((
                res.status(),
                format!("Failed to get logs using token {}", token),
            ))
        }
    };
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(body)
}
