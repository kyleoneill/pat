use crate::models::reminder::{Category, Priority, Reminder, ReminderUpdateSchema};
use crate::testing::helpers::{delete_request, get_request, post_request, put_request};
use axum::body::Body;
use axum::http::StatusCode;
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
    let data = json!({"slug": slug, "name": name});
    post_request(client, "/reminders/category", data, Some(token), addr).await
}

pub async fn get_categories(client: &Client<HttpConnector, Body>, token: &str, addr: &SocketAddr) -> Result<Vec<Category>, (StatusCode, String)> {
    get_request(client, "/reminders/category", token, addr).await
}

pub async fn delete_category_by_id(
    client: &Client<HttpConnector, Body>,
    token: &str,
    addr: &SocketAddr,
    category_id: String,
) -> Result<(), (StatusCode, String)> {
    let path = format!("/reminders/category/{category_id}");
    delete_request(client, path.as_str(), token, addr).await
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
    let data = json!({"name": name, "description": description, "categories": categories, "priority": priority});
    post_request(client, "/reminders", data, Some(token), addr).await
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
            format!("/reminders?{params}")
        }
        None => "/reminders".to_string(),
    };
    get_request(client, built_uri.as_str(), token, addr).await
}

pub async fn update_reminder_helper(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    reminder_id: String,
    reminder_updates: ReminderUpdateSchema,
) -> Result<Reminder, (StatusCode, String)> {
    let path = format!("/reminders/{reminder_id}");
    let data = json!(reminder_updates);
    put_request(client, path.as_str(), data, token, addr).await
}

pub async fn delete_reminder_helper(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    reminder_id: String,
) -> Result<(), (StatusCode, String)> {
    let path = format!("/reminders/{reminder_id}");
    delete_request(client, path.as_str(), token, addr).await
}
