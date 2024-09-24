use crate::AppState;
use axum::{
    extract::{Path, State},
    http::header::HeaderMap,
    routing::{delete, get, post, put},
    Json, Router,
};

use super::get_user_from_token;
use super::return_data::ReturnData;

use crate::models::reminder::{
    reminder_db::{
        db_delete_reminder, db_update_reminder, delete_category_by_id, get_categories_for_user,
        get_reminders_for_user, insert_category, insert_reminder,
    },
    Category, CategorySchema, Reminder, ReminderSchema, ReminderUpdateSchema,
};

pub fn reminder_routes() -> Router<AppState> {
    Router::<AppState>::new()
        // Reminders
        .route("/reminders", post(create_reminder))
        .route("/reminders", get(list_reminders))
        .route("/reminders/:reminder_id", put(update_reminder))
        .route("/reminders/:reminder_id", delete(delete_reminder))
        // Categories
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
    let user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match insert_category(pool, &category_data, user.get_id()).await {
        Ok(category) => ReturnData::created(category),
        Err(db_err) => db_err.into(),
    }
}

async fn get_categories(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> ReturnData<Vec<Category>, String> {
    let pool = &app_state.db;
    let user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match get_categories_for_user(pool, user.get_id()).await {
        Ok(categories) => ReturnData::ok(categories),
        Err(e) => e.into(),
    }
}

async fn delete_category(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(category_id): Path<i64>,
) -> ReturnData<(), String> {
    let pool = &app_state.db;
    let user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match delete_category_by_id(pool, category_id, user.get_id()).await {
        Ok(res) => match res {
            0 => ReturnData::not_found(
                "Could not find a category with the given id for the current user".to_owned(),
            ),
            1 => ReturnData::ok(()),
            _ => ReturnData::internal_error("Unhandled exception while deleting data".to_owned()),
        },
        Err(e) => e.into(),
    }
}

async fn create_reminder(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(reminder_data): Json<ReminderSchema>,
) -> ReturnData<Reminder, String> {
    let pool = &app_state.db;
    let user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match insert_reminder(pool, &reminder_data, user.get_id()).await {
        Ok(reminder) => ReturnData::created(reminder),
        Err(db_err) => db_err.into(),
    }
}

async fn list_reminders(
    State(app_state): State<AppState>,
    headers: HeaderMap, // TODO: QUERY SCHEMA
) -> ReturnData<Vec<Reminder>, String> {
    // TODO: Filter by category
    //       Pagination
    //       Sort
    let pool = &app_state.db;
    let user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match get_reminders_for_user(pool, user.get_id()).await {
        Ok(reminders) => ReturnData::ok(reminders),
        Err(db_err) => db_err.into(),
    }
}

// TODO: Get reminder by id

async fn update_reminder(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(reminder_id): Path<i64>,
    Json(update_data): Json<ReminderUpdateSchema>,
) -> ReturnData<Reminder, String> {
    let pool = &app_state.db;
    let _user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match db_update_reminder(pool, reminder_id, update_data).await {
        Ok(reminder) => ReturnData::ok(reminder),
        Err(e) => e.into(),
    }
}

async fn delete_reminder(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(reminder_id): Path<i64>,
) -> ReturnData<(), String> {
    let pool = &app_state.db;
    let user = match get_user_from_token(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match db_delete_reminder(pool, reminder_id, user.get_id()).await {
        Ok(_) => ReturnData::ok(()),
        Err(db_err) => db_err.into(),
    }
}
