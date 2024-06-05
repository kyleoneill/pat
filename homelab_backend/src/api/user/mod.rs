mod db;
mod jwt;
mod util;

use db::{db_delete_user, db_get_user_by_id, db_get_user_by_username};
use jwt::{encode_jwt, get_and_decode_auth_token, get_user_from_token};
use util::{generate_salt, hash_password};

use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header::HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::collections::HashMap;

#[derive(Deserialize, Type, Serialize, PartialEq)]
pub enum AuthLevel {
    User,
    Admin,
}

impl From<i64> for AuthLevel {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::User,
            1 => Self::Admin,
            _ => panic!("Unsupported value when converting an i64 to an AuthLevel"),
        }
    }
}

#[derive(Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub auth_level: AuthLevel,
    pub salt: String,
}

#[derive(Serialize)]
pub struct ReturnUser {
    pub id: i64,
    pub username: String,
}

impl From<User> for ReturnUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    username: String,
    password: String,
}

// ------------------------------------------------------------------------------------------------
// Endpoints

// TODO: The error handling here is pretty bad. It was done quickly to get an MVP off the ground
//       but it really needs to be refactored

pub fn auth_routes() -> Router<AppState> {
    // TODO: This routing is not terribly logically consistent and should be re-done
    // Should have a consistent GET/PUT/DELETE for /user/me and /user/:user_id
    //  each should point to their own custom endpoint function that handle the difference between
    //  /me and an id and then call a shared db/business-logic function
    Router::<AppState>::new()
        .route("/user", post(create_user))
        .route("/user", get(get_user_by_username))
        .route("/user/:user_id", delete(delete_user_by_id))
        .route("/user/me", delete(delete_user_me))
        .route("/user/me", get(get_user_me))
        .route("/user/auth", post(auth_user))
}
// TODO: GET user/me and/or /user/:user_id
// TODO: PUT user/me and/or /user/:user_id

async fn auth_user(
    State(app_state): State<AppState>,
    Json(credentials): Json<LoginUserSchema>,
) -> Result<(StatusCode, Json<String>), (StatusCode, Json<String>)> {
    let pool = &app_state.db;
    match sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = ?",
        credentials.username
    )
    .fetch_optional(pool)
    .await
    {
        Ok(maybe_user) => match maybe_user {
            Some(user) => {
                // Check if the users credentials are correct
                let hashed_password = hash_password(credentials.password, user.salt.as_str());
                if hashed_password == user.password {
                    // Generate a token for the user
                    let app_secret = app_state.config.app_secret.as_str();
                    let jwt_lifetime = app_state.config.jwt_max_age as usize;
                    let token = encode_jwt(app_secret, user.id, jwt_lifetime);
                    Ok((StatusCode::CREATED, Json(token)))
                } else {
                    Err((
                        StatusCode::NOT_FOUND,
                        Json(format!(
                            "No such user with username '{}', or invalid password",
                            credentials.username
                        )),
                    ))
                }
            }
            None => Err((
                StatusCode::NOT_FOUND,
                Json(format!(
                    "No such user with username '{}', or invalid password",
                    credentials.username
                )),
            )),
        },
        Err(_) => panic!("TODO: Auth user error"),
    }
}

async fn create_user(
    State(app_state): State<AppState>,
    Json(credentials): Json<LoginUserSchema>,
) -> Result<(StatusCode, Json<ReturnUser>), (StatusCode, Json<String>)> {
    let pool = &app_state.db;

    // Check the database if this username is already taken
    if db_get_user_by_username(pool, credentials.username.as_str())
        .await
        .is_some()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(format!(
                "Username '{}' is already taken",
                credentials.username
            )),
        ));
    };

    // Generate a salt for the user
    let salt = generate_salt();

    // Generate a password hash from the input password and the salt
    let hash = hash_password(credentials.password.clone(), salt.as_str());

    // Create a user in the database
    // TODO: Error handling here
    let username = credentials.username.clone();
    let _ = sqlx::query!(
        "INSERT INTO users (username, password, auth_level, salt) VALUES (?, ?, ?, ?)",
        username,
        hash,
        AuthLevel::User,
        salt
    )
    .execute(pool)
    .await;

    match db_get_user_by_username(pool, credentials.username.as_str()).await {
        Some(user) => Ok((StatusCode::CREATED, Json(Into::<ReturnUser>::into(user)))),
        // TODO: Actual error handling here
        None => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal error".to_owned()),
        )),
    }
}

async fn get_user_by_username(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    query_params: Query<HashMap<String, String>>,
) -> Result<(StatusCode, Json<ReturnUser>), (StatusCode, Json<String>)> {
    // Get the username out of query params
    let username = match query_params.get("username") {
        Some(username) => username,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json("Missing necessary 'username' query param".to_string()),
            ))
        }
    };

    // Get the JWT from the Authorization header and decode it
    // We don't need to do anything with this data, but we do need to validate that a real user made the request
    let _token_user_id =
        match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
            Ok(id) => id,
            Err(e) => return Err((StatusCode::BAD_REQUEST, Json(e))),
        };

    // Find and return the user
    match db_get_user_by_username(&app_state.db, username.as_str()).await {
        Some(user) => Ok((StatusCode::OK, Json(user.into()))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json("Could not find user with the given ID".to_string()),
        )),
    }
}

async fn get_user_me(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<ReturnUser>), (StatusCode, String)> {
    match get_user_from_token(&app_state.db, &headers, &app_state.config.app_secret).await {
        Ok(user) => Ok((StatusCode::OK, Json(user.into()))),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

async fn delete_user_by_id(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<String>)> {
    // Get the JWT from the Authorization header and decode it
    let token_user_id =
        match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
            Ok(id) => id,
            Err(e) => return Err((StatusCode::BAD_REQUEST, Json(e))),
        };

    // Get the account of the id from the path
    let user = match db_get_user_by_id(&app_state.db, user_id).await {
        Some(user) => user,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json("Could not find user with the given ID".to_string()),
            ))
        }
    };

    // Check if the requester id matches the account being deleted
    if user.id != token_user_id {
        // The token user's ID does not match the user being deleted
        // Get the token user's account and check to see if they are an admin
        match db_get_user_by_id(&app_state.db, token_user_id).await {
            Some(user) => {
                if user.auth_level != AuthLevel::Admin {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json("Cannot delete an account you do not have access to".to_string()),
                    ));
                }
            }
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json("Could not find user matching the Authorization token".to_string()),
                ))
            }
        };
    };

    // Delete the user
    match db_delete_user(&app_state.db, user.id).await {
        // TODO: Should check .rows_affected() on the return here and error if it is anything other than 1
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Failed to delete user".to_string()),
        )),
    }
}

async fn delete_user_me(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<String>)> {
    // Get the JWT from the Authorization header and decode it
    let token_user_id =
        match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
            Ok(id) => id,
            Err(e) => return Err((StatusCode::BAD_REQUEST, Json(e))),
        };

    // Delete the user
    match db_delete_user(&app_state.db, token_user_id).await {
        // TODO: Should check .rows_affected() on the return here and error if it is anything other than 1
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Failed to delete user".to_string()),
        )),
    }
}
