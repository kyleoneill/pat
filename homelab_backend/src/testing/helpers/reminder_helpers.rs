use crate::models::reminder::{Category, Priority, Reminder};
use crate::testing::json_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde_json::json;
use std::net::SocketAddr;

pub async fn create_category(
    client: &Client<HttpConnector, Body>,
    token: &str,
    slug: &str,
    name: &str,
    addr: &SocketAddr,
) -> Result<Category, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/reminders/category"))
        .method("POST")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(json_bytes(json!({"slug": slug, "name": name}))))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::CREATED => (),
        _ => return Err((res.status(), "Failed to create a category".to_owned())),
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn get_categories(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr
) -> Result<Vec<Category>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/reminders/category/all"))
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(()))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::OK => (),
        _ => return Err((res.status(), "Failed to get categories".to_owned())),
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn delete_category_by_id(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
    category_id: i64
) -> Result<(), (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/reminders/category/{category_id}"))
        .method("DELETE")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(()))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::OK => (),
        _ => return Err((res.status(), "Failed to delete category by id".to_owned())),
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_reminder(
    client: &Client<HttpConnector, Body>,
    token: &str,
    name: &str,
    description: &str,
    categories: Vec<i64>,
    priority: Priority,
    addr: &SocketAddr,
) -> Result<Reminder, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/reminders"))
        .method("POST")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(json_bytes(
            json!({"name": name, "description": description, "categories": categories, "priority": priority}),
        )))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::CREATED => (),
        _ => return Err((res.status(), "Failed to create a reminder".to_owned())),
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn list_reminders(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str
) -> Result<Vec<Reminder>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/reminders"))
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(()))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::OK => (),
        _ => return Err((res.status(), "Failed to get reminder list".to_owned())),
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}
