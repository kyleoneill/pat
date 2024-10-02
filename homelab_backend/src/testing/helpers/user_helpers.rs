use crate::models::user::ReturnUser;
use crate::testing::json_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde_json::json;
use std::net::SocketAddr;

pub async fn create_user(
    client: &Client<HttpConnector, Body>,
    username: &str,
    password: &str,
    addr: &SocketAddr,
) -> Result<(), (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/users"))
        .method("POST")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .body(Body::from(json_bytes(
            json!({"username": username, "password": password}),
        )))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::CREATED => Ok(()),
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = serde_json::from_slice(body.as_ref()).unwrap();
            Err((
                status,
                format!("Failed to create a user with error '{message}'"),
            ))
        }
    }
}

pub async fn auth_user(
    client: &Client<HttpConnector, Body>,
    username: &str,
    password: &str,
    addr: &SocketAddr,
) -> Result<String, (StatusCode, String)> {
    let auth_req = Request::builder()
        .uri(format!("http://{addr}/api/users/auth"))
        .method("POST")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .body(Body::from(json_bytes(
            json!({"username": username, "password": password}),
        )))
        .unwrap();
    let response = client.request(auth_req).await.unwrap();
    match response.status() {
        StatusCode::CREATED => (),
        _ => {
            return Err((
                response.status(),
                format!(
                    "Failed to auth as user {} with password {}",
                    username, password
                ),
            ))
        }
    };
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_as_ref = body.as_ref();
    let slice = &body_as_ref[1..body_as_ref.len() - 1];
    Ok(std::str::from_utf8(slice).unwrap().to_string())
}

pub async fn get_user_me(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
) -> Result<ReturnUser, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/users/me"))
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
            return Err((
                res.status(),
                format!("Failed to get a user using token {}", token),
            ))
        }
    };
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(ReturnUser::from_bytes(&body))
}
