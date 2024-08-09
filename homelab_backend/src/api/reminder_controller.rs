use crate::AppState;
use axum::{extract::{State, Path}, http::header::HeaderMap, routing::{get, post, delete}, Json, Router};

use super::get_user_from_token;
use super::return_data::ReturnData;

use crate::models::reminder::{
    reminder_db::{insert_category, insert_reminder, get_categories_for_user, delete_category_by_id},
    Category, CategorySchema, Reminder, ReminderSchema,
};

pub fn reminder_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/reminders", post(create_reminder))
        .route("/reminders/category", post(create_category))
        .route("/reminders/category/all", get(get_categories))
        .route("/reminders/category/:category_id", delete(delete_category))
}

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

async fn get_categories(
    State(app_state): State<AppState>,
    headers: HeaderMap
) -> ReturnData<Vec<Category>, String> {
    let pool = &app_state.db;
    match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => {
            match get_categories_for_user(pool, user.get_id()).await {
                Ok(categories) => ReturnData::ok(categories),
                Err(e) => e.into()
            }
        },
        Err(e) => ReturnData::not_found(e)
    }
}

async fn delete_category(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(category_id): Path<i64>
) -> ReturnData<(), String> {
    let pool = &app_state.db;
    match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => {
            match delete_category_by_id(pool, category_id, user.get_id()).await {
                Ok(res) => {
                    match res {
                        0 => ReturnData::not_found("Could not find a category with the given id for the current user".to_owned()),
                        1 => ReturnData::ok(()),
                        _ => ReturnData::internal_error("Unhandled exception while deleting data".to_owned())
                    }
                },
                Err(e) => e.into()
            }
        },
        Err(e) => ReturnData::not_found(e)
    }
}

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
