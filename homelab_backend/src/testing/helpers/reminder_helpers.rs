use crate::models::reminder::{validation::UpdateReminderSchema, Category, Priority, Reminder};
use crate::testing::{
    helpers::{delete_request, get_request, post_request, put_request},
    TestHelper,
};
use axum::http::StatusCode;
use serde_json::json;

pub async fn create_category(test_helper: &TestHelper, token: &str, slug: &str, name: &str) -> Result<Category, (StatusCode, String)> {
    let data = json!({"slug": slug, "name": name});
    post_request(test_helper, "/reminders/category", data, Some(token)).await
}

pub async fn get_categories(test_helper: &TestHelper, token: &str) -> Result<Vec<Category>, (StatusCode, String)> {
    get_request(test_helper, "/reminders/category", token).await
}

pub async fn delete_category_by_id(test_helper: &TestHelper, token: &str, category_id: String) -> Result<(), (StatusCode, String)> {
    let path = format!("/reminders/category/{category_id}");
    delete_request(test_helper, path.as_str(), token).await
}

#[allow(clippy::too_many_arguments)]
pub async fn create_reminder(
    test_helper: &TestHelper,
    token: &str,
    name: &str,
    description: &str,
    categories: Vec<String>,
    priority: Priority,
) -> Result<Reminder, (StatusCode, String)> {
    let data = json!({"name": name, "description": description, "categories": categories, "priority": priority});
    post_request(test_helper, "/reminders", data, Some(token)).await
}

pub async fn list_reminders(test_helper: &TestHelper, token: &str, categories: Option<Vec<String>>) -> Result<Vec<Reminder>, (StatusCode, String)> {
    let built_uri = match categories {
        Some(filter_categories) => {
            let params = super::list_to_query_params("categories", filter_categories);
            format!("/reminders?{params}")
        }
        None => "/reminders".to_string(),
    };
    get_request(test_helper, built_uri.as_str(), token).await
}

pub async fn update_reminder_helper(
    test_helper: &TestHelper,
    token: &str,
    reminder_id: String,
    reminder_updates: UpdateReminderSchema,
) -> Result<Reminder, (StatusCode, String)> {
    let path = format!("/reminders/{reminder_id}");
    let data = json!(reminder_updates);
    put_request(test_helper, path.as_str(), data, token).await
}

pub async fn delete_reminder_helper(test_helper: &TestHelper, token: &str, reminder_id: String) -> Result<(), (StatusCode, String)> {
    let path = format!("/reminders/{reminder_id}");
    delete_request(test_helper, path.as_str(), token).await
}
