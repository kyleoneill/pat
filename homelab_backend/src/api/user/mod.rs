pub mod db;
pub(crate) mod jwt;
mod util;

use super::get_user_from_token;
use super::return_data::ReturnData;
use db::{db_delete_user, db_get_user_by_id, db_get_user_by_username};
use jwt::{encode_jwt, get_and_decode_auth_token};
use util::{generate_salt, hash_password};

use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::header::HeaderMap,
    routing::{delete, get, post},
    Json, Router,
};
use hyper::body::Bytes;
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
    id: i64,
    username: String,
    password: String,
    auth_level: AuthLevel,
    salt: String,
}

impl User {
    pub fn get_id(&self) -> i64 {
        self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

impl ReturnUser {
    #[allow(dead_code)] // used in test
    pub fn from_bytes(input: &Bytes) -> Self {
        serde_json::from_slice(input).unwrap()
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

pub fn user_routes() -> Router<AppState> {
    // TODO: This routing is not terribly logically consistent and should be re-done
    // Should have a consistent GET/PUT/DELETE for /user/me and /user/:user_id
    //  each should point to their own custom endpoint function that handle the difference between
    //  /me and an id and then call a shared db/business-logic function
    Router::<AppState>::new()
        .route("/users", post(create_user))
        .route("/users/auth", post(auth_user))
        .route("/users", get(get_user_by_username))
        .route("/users/me", get(get_user_me))
        .route("/users/:user_id", delete(delete_user_by_id))
        .route("/users/me", delete(delete_user_me))
}
// TODO: GET /users/:user_id
// TODO: PUT /users/me and /users/:user_id

async fn auth_user(
    State(app_state): State<AppState>,
    Json(credentials): Json<LoginUserSchema>,
) -> ReturnData<String, String> {
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
                    ReturnData::created(token)
                } else {
                    let ret_data = format!(
                        "No such user with username '{}', or invalid password",
                        credentials.username
                    );
                    ReturnData::not_found(ret_data)
                }
            }
            None => {
                let ret_data = format!(
                    "No such user with username '{}', or invalid password",
                    credentials.username
                );
                ReturnData::not_found(ret_data)
            }
        },
        Err(_) => panic!("TODO: Auth user error"),
    }
}

async fn create_user(
    State(app_state): State<AppState>,
    Json(credentials): Json<LoginUserSchema>,
) -> ReturnData<ReturnUser, String> {
    let pool = &app_state.db;

    // Check the database if this username is already taken
    if db_get_user_by_username(pool, credentials.username.as_str())
        .await
        .is_some()
    {
        return ReturnData::bad_request(format!(
            "Username '{}' is already taken",
            credentials.username
        ));
    };

    // Generate a salt for the user
    let salt = generate_salt();

    // Generate a password hash from the input password and the salt
    let hash = hash_password(credentials.password.clone(), salt.as_str());

    // Create a user in the database
    // TODO: Error handling here
    // TODO: This should be in the db file
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
        Some(user) => ReturnData::created(Into::<ReturnUser>::into(user)),
        // TODO: Actual error handling here
        None => ReturnData::internal_error("Internal Error".to_owned()),
    }
}

async fn get_user_by_username(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    query_params: Query<HashMap<String, String>>,
) -> ReturnData<ReturnUser, String> {
    // Get the username out of query params
    let username = match query_params.get("username") {
        Some(username) => username,
        None => {
            return ReturnData::bad_request("Missing necessary 'username' query param".to_owned())
        }
    };

    // Get the JWT from the Authorization header and decode it
    // We don't need to do anything with this data, but we do need to validate that a real user made the request
    let _token_user_id =
        match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
            Ok(id) => id,
            Err(e) => return ReturnData::bad_request(e),
        };

    // Find and return the user
    match db_get_user_by_username(&app_state.db, username.as_str()).await {
        Some(user) => ReturnData::ok(Into::<ReturnUser>::into(user)),
        None => ReturnData::not_found("Could not find user with the given ID".to_string()),
    }
}

async fn get_user_me(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> ReturnData<ReturnUser, String> {
    match get_user_from_token(&app_state.db, &headers, &app_state.config.app_secret).await {
        Ok(user) => ReturnData::ok(Into::<ReturnUser>::into(user)),
        Err(e) => ReturnData::not_found(e),
    }
}

async fn delete_user_by_id(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> ReturnData<(), String> {
    // Get the JWT from the Authorization header and decode it
    let token_user_id =
        match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
            Ok(id) => id,
            Err(e) => return ReturnData::bad_request(e),
        };

    // Get the account of the id from the path
    let user = match db_get_user_by_id(&app_state.db, user_id).await {
        Some(user) => user,
        None => return ReturnData::not_found("Could not find user with the given ID".to_string()),
    };

    // Check if the requester id matches the account being deleted
    if user.id != token_user_id {
        // The token user's ID does not match the user being deleted
        // Get the token user's account and check to see if they are an admin
        match db_get_user_by_id(&app_state.db, token_user_id).await {
            Some(user) => {
                if user.auth_level != AuthLevel::Admin {
                    return ReturnData::forbidden(
                        "Cannot delete an account you do not have access to".to_string(),
                    );
                }
            }
            None => {
                return ReturnData::not_found(
                    "Could not find user matching the Authorization token".to_string(),
                )
            }
        };
    };

    // Delete the user
    match db_delete_user(&app_state.db, user.id).await {
        // TODO: Should check .rows_affected() on the return here and error if it is anything other than 1
        Ok(_) => ReturnData::ok(()),
        Err(_) => ReturnData::internal_error("Failed to delete user".to_owned()),
    }
}

async fn delete_user_me(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> ReturnData<(), String> {
    // Get the JWT from the Authorization header and decode it
    let token_user_id =
        match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
            Ok(id) => id,
            Err(e) => return ReturnData::bad_request(e),
        };

    // Delete the user
    match db_delete_user(&app_state.db, token_user_id).await {
        // TODO: Should check .rows_affected() on the return here and error if it is anything other than 1
        Ok(_) => ReturnData::ok(()),
        Err(_) => ReturnData::internal_error("Failed to delete user".to_owned()),
    }
}
