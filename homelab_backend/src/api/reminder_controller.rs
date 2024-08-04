use crate::AppState;
use axum::{extract::State, http::header::HeaderMap, routing::post, Json, Router};

use super::get_user_from_token;
use super::return_data::ReturnData;

pub fn reminder_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/reminders", post(create_reminder))
        .route("/reminders/category", post(create_category))
}

use crate::models::reminder::{
    reminder_db::{insert_category, insert_reminder},
    Category, CategorySchema, Reminder, ReminderSchema,
};

async fn create_category(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(category_data): Json<CategorySchema>,
) -> ReturnData<Category, String> {
    let pool = &app_state.db;
    match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => match insert_category(pool, &category_data, user.get_id()).await {
            Ok(category) => ReturnData::created(category),
            Err(db_err) => db_err.into(),
        },
        Err(e) => ReturnData::not_found(e),
    }
}

// TODO: Get categories
// TODO: Delete category

async fn create_reminder(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(reminder_data): Json<ReminderSchema>,
) -> ReturnData<Reminder, String> {
    let pool = &app_state.db;
    match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => match insert_reminder(pool, &reminder_data, user.get_id()).await {
            Ok(reminder) => ReturnData::created(reminder),
            Err(db_err) => db_err.into(),
        },
        Err(e) => ReturnData::not_found(e),
    }
}

// TODO: Get all reminders for a user, filterable
//       ^ THIS SHOULD PAGINATE
// TODO: Get reminder by id/slug
// TODO: Update reminder
// TODO: Delete reminder
