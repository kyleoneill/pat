use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::header::HeaderMap,
    routing::{delete, get, post, put},
    Json, Router,
};
use rand::{distr::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

use crate::{
    api::{get_user_from_auth_header, return_data::ReturnData},
    app::AppState,
    models::user::{
        jwt::encode_jwt,
        user_db::{db_create_user, db_delete_user, db_get_user_by_id, db_get_user_by_username, db_update_user},
        validation::{LoginUserSchema, UpdateUserSchema},
        AuthLevel, ReturnUser,
    },
};

const SALT_LENGTH: usize = 12;

pub fn hash_password(mut password: String, salt: &str) -> String {
    password.push_str(salt);
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    format!("{result:X}")
}

pub fn generate_salt() -> String {
    rand::rng().sample_iter(&Alphanumeric).take(SALT_LENGTH).map(char::from).collect()
}

pub fn user_routes() -> Router<Arc<AppState>> {
    // TODO: This routing is not terribly logically consistent and should be re-done
    // Should have a consistent GET/PUT/DELETE for /user/me and /user/:user_id
    //  each should point to their own custom endpoint function that handle the difference between
    //  /me and an id and then call a shared db/business-logic function
    Router::<Arc<AppState>>::new()
        .route("/users", post(create_user))
        .route("/users/auth", post(auth_user))
        .route("/users/me", put(update_user_me))
        .route("/users/:user_id", get(get_user_by_id))
        .route("/users/me", get(get_user_me))
        .route("/users/:user_id", delete(delete_user_by_id))
        .route("/users/me", delete(delete_user_me))
}
// TODO: PUT /users/me and /users/:user_id

fn create_token(app_state: &Arc<AppState>, user_id: String) -> String {
    let app_secret = app_state.config.app_secret.as_str();
    let jwt_lifetime = app_state.config.jwt_max_age as usize;
    encode_jwt(app_secret, user_id, jwt_lifetime)
}

async fn auth_user(State(app_state): State<Arc<AppState>>, Json(credentials): Json<LoginUserSchema>) -> ReturnData<String> {
    let pool = &app_state.db;
    match db_get_user_by_username(pool, credentials.username.as_str()).await {
        Ok(user) => {
            // Check if the users credentials are correct
            let hashed_password = hash_password(credentials.password, user.salt.as_str());
            if hashed_password == user.password {
                // Generate a token for the user
                let token = create_token(&app_state, user.get_id());
                ReturnData::created(token)
            } else {
                let ret_data = format!("No such user with username '{}', or invalid password", credentials.username);
                ReturnData::not_found(ret_data)
            }
        }
        Err(e) => e.into(),
    }
}

async fn create_user(State(app_state): State<Arc<AppState>>, Json(credentials): Json<LoginUserSchema>) -> ReturnData<String> {
    let pool = &app_state.db;

    // Check the database if this username is already taken
    if db_get_user_by_username(pool, credentials.username.as_str()).await.is_ok() {
        return ReturnData::bad_request(format!("Username '{}' is already taken", credentials.username));
    };

    // Generate a salt for the user
    let salt = generate_salt();

    // Generate a password hash from the input password and the salt
    let hash = hash_password(credentials.password.clone(), salt.as_str());

    // Create a user in the database
    let user = match db_create_user(pool, credentials.username.clone(), hash, AuthLevel::User, salt).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };

    // Create an auth token for the newly created user
    let token = create_token(&app_state, user.get_id());
    ReturnData::created(token)
}

async fn update_user_me(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut update_data): Json<UpdateUserSchema>,
) -> ReturnData<ReturnUser> {
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };

    // If another user creates or updates their username to this value at the _exact_ same time
    // then this could put the db in a bad state, there is a tiny moment here between this
    // validation and the db being updated (Also relevant in the create_user function).
    // Not really important for a low use app, but how do other services solve this? Transaction?
    if let Some(new_username) = &update_data.username {
        if db_get_user_by_username(pool, new_username.as_str()).await.is_ok() {
            return ReturnData::bad_request(format!("Username '{new_username}' is already taken"));
        };
    }

    // If the user is changing their password, swap out the new password for its hash
    if let Some(new_password) = update_data.password {
        let hash = hash_password(new_password, user.salt.as_str());
        update_data.password = Some(hash);
    }

    match db_update_user(pool, user, update_data).await {
        Ok(user) => ReturnData::ok(Into::<ReturnUser>::into(user)),
        Err(e) => e.into(),
    }
}

async fn get_user_by_id(State(app_state): State<Arc<AppState>>, headers: HeaderMap, Path(user_id): Path<String>) -> ReturnData<ReturnUser> {
    let user = match get_user_from_auth_header(&app_state.db, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };

    // TODO: Should have a general purpose permission handler
    // TODO: Should have a test for this
    if user.auth_level != AuthLevel::Admin {
        return ReturnData::forbidden("Cannot retrieve an account you do not have access to");
    }

    // Find and return the user
    match db_get_user_by_id(&app_state.db, user_id.as_str()).await {
        Ok(user) => ReturnData::ok(Into::<ReturnUser>::into(user)),
        Err(e) => e.into(),
    }
}

async fn get_user_me(State(app_state): State<Arc<AppState>>, headers: HeaderMap) -> ReturnData<ReturnUser> {
    let user = match get_user_from_auth_header(&app_state.db, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    ReturnData::ok(Into::<ReturnUser>::into(user))
}

async fn delete_user_by_id(State(app_state): State<Arc<AppState>>, headers: HeaderMap, Path(user_to_delete_id): Path<String>) -> ReturnData<()> {
    let user = match get_user_from_auth_header(&app_state.db, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };

    // Check if the requester matches the account being deleted, or if they're an admin
    if user.get_id() != user_to_delete_id && user.auth_level != AuthLevel::Admin {
        return ReturnData::forbidden("Cannot delete an account you do not have access to");
    };

    // Delete the user
    match db_delete_user(&app_state.db, user.get_id()).await {
        Ok(_) => ReturnData::ok(()),
        Err(e) => e.into(),
    }
}

async fn delete_user_me(State(app_state): State<Arc<AppState>>, headers: HeaderMap) -> ReturnData<()> {
    let user = match get_user_from_auth_header(&app_state.db, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };

    // Delete the user
    match db_delete_user(&app_state.db, user.get_id()).await {
        Ok(_) => ReturnData::ok(()),
        Err(e) => e.into(),
    }
}
