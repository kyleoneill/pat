use super::get_user_from_auth_header;
use super::return_data::ReturnData;
use crate::models::user::jwt::encode_jwt;
use crate::models::user::user_db::{db_create_user, db_delete_user, db_get_user_by_username};
use crate::models::user::{AuthLevel, LoginUserSchema, ReturnUser};
use crate::AppState;

use axum::{
    extract::{Path, Query, State},
    http::header::HeaderMap,
    routing::{delete, get, post},
    Json, Router,
};
use rand::{distr::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const SALT_LENGTH: usize = 12;

pub fn hash_password(mut password: String, salt: &str) -> String {
    password.push_str(salt);
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    format!("{:X}", result)
}

pub fn generate_salt() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(SALT_LENGTH)
        .map(char::from)
        .collect()
}

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
) -> ReturnData<String> {
    let pool = &app_state.db;
    match db_get_user_by_username(pool, credentials.username.as_str()).await {
        Ok(user) => {
            // Check if the users credentials are correct
            let hashed_password = hash_password(credentials.password, user.salt.as_str());
            if hashed_password == user.password {
                // Generate a token for the user
                let app_secret = app_state.config.app_secret.as_str();
                let jwt_lifetime = app_state.config.jwt_max_age as usize;
                let token = encode_jwt(app_secret, user.get_id(), jwt_lifetime);
                ReturnData::created(token)
            } else {
                let ret_data = format!(
                    "No such user with username '{}', or invalid password",
                    credentials.username
                );
                ReturnData::not_found(ret_data)
            }
        }
        Err(e) => e.into(),
    }
}

async fn create_user(
    State(app_state): State<AppState>,
    Json(credentials): Json<LoginUserSchema>,
) -> ReturnData<ReturnUser> {
    let pool = &app_state.db;

    // Check the database if this username is already taken
    if db_get_user_by_username(pool, credentials.username.as_str())
        .await
        .is_ok()
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
    match db_create_user(
        pool,
        credentials.username.clone(),
        hash,
        AuthLevel::User,
        salt,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => return e.into(),
    };

    match db_get_user_by_username(pool, credentials.username.as_str()).await {
        // TODO: This should probably return an auth token instead?
        Ok(user) => ReturnData::created(Into::<ReturnUser>::into(user)),
        Err(_) => {
            ReturnData::internal_error("Unhandled exception after creating a user".to_owned())
        }
    }
}

async fn get_user_by_username(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    query_params: Query<HashMap<String, String>>,
) -> ReturnData<ReturnUser> {
    // TODO: Should have a general purpose validator
    // Get the username out of query params
    let username = match query_params.get("username") {
        Some(username) => username,
        None => {
            return ReturnData::bad_request("Missing necessary 'username' query param".to_owned())
        }
    };

    let user = match get_user_from_auth_header(
        &app_state.db,
        &headers,
        &app_state.config.app_secret,
    )
    .await
    {
        Ok(user) => user,
        Err(e) => return e.into(),
    };

    // TODO: Should have a general purpose permission handler
    // TODO: Should have a test for this
    if user.auth_level != AuthLevel::Admin {
        return ReturnData::forbidden(
            "Cannot retrieve an account you do not have access to".to_owned(),
        );
    }

    // Find and return the user
    match db_get_user_by_username(&app_state.db, username.as_str()).await {
        Ok(user) => ReturnData::ok(Into::<ReturnUser>::into(user)),
        Err(e) => e.into(),
    }
}

async fn get_user_me(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> ReturnData<ReturnUser> {
    let user = match get_user_from_auth_header(
        &app_state.db,
        &headers,
        &app_state.config.app_secret,
    )
    .await
    {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    ReturnData::ok(Into::<ReturnUser>::into(user))
}

async fn delete_user_by_id(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(user_to_delete_id): Path<String>,
) -> ReturnData<()> {
    let user = match get_user_from_auth_header(
        &app_state.db,
        &headers,
        &app_state.config.app_secret,
    )
    .await
    {
        Ok(user) => user,
        Err(e) => return e.into(),
    };

    // Check if the requester matches the account being deleted, or if they're an admin
    if user.get_id() != user_to_delete_id && user.auth_level != AuthLevel::Admin {
        return ReturnData::forbidden(
            "Cannot delete an account you do not have access to".to_string(),
        );
    };

    // Delete the user
    match db_delete_user(&app_state.db, user.get_id()).await {
        Ok(_) => ReturnData::ok(()),
        Err(e) => e.into(),
    }
}

async fn delete_user_me(State(app_state): State<AppState>, headers: HeaderMap) -> ReturnData<()> {
    let user = match get_user_from_auth_header(
        &app_state.db,
        &headers,
        &app_state.config.app_secret,
    )
    .await
    {
        Ok(user) => user,
        Err(e) => return e.into(),
    };

    // Delete the user
    match db_delete_user(&app_state.db, user.get_id()).await {
        Ok(_) => ReturnData::ok(()),
        Err(e) => e.into(),
    }
}
