use crate::models::reminder::{Category, Priority, Reminder, ReminderUpdateSchema};
use crate::testing::helpers::read_error_message;
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
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            return Err((status, message));
        }
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn get_categories(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
) -> Result<Vec<Category>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/reminders/category"))
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(()))
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
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn delete_category_by_id(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
    category_id: String,
) -> Result<(), (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!(
            "http://{addr}/api/reminders/category/{category_id}"
        ))
        .method("DELETE")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(()))
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
    }
    let _body = res.into_body().collect().await.unwrap().to_bytes();
    // serde_json::from_slice(body.as_ref()).unwrap();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_reminder(
    client: &Client<HttpConnector, Body>,
    token: &str,
    name: &str,
    description: &str,
    categories: Vec<String>,
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
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            return Err((status, message));
        }
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn list_reminders(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    categories: Option<Vec<String>>,
) -> Result<Vec<Reminder>, (StatusCode, String)> {
    let built_uri = match categories {
        Some(filter_categories) => {
            let params = super::list_to_query_params("categories", filter_categories);
            format!("http://{addr}/api/reminders?{params}")
        }
        None => format!("http://{addr}/api/reminders"),
    };
    let req = Request::builder()
        .uri(built_uri)
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(()))
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
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn update_reminder_helper(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    reminder_id: String,
    reminder_updates: ReminderUpdateSchema,
) -> Result<Reminder, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/reminders/{reminder_id}"))
        .method("PUT")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(json_bytes(json!(reminder_updates))))
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
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn delete_reminder_helper(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    reminder_id: String,
) -> Result<(), (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api/reminders/{reminder_id}"))
        .method("DELETE")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(()))
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
    }
    let _body = res.into_body().collect().await.unwrap().to_bytes();
    // serde_json::from_slice(body.as_ref()).unwrap();
    Ok(())
}
