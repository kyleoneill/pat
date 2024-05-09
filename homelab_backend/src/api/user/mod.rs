use std::collections::HashMap;
use axum::{
    Router,
    routing::{post, delete, get},
    extract::{State, Path, Query},
    Json,
    http::{StatusCode, header::{HeaderMap, AUTHORIZATION}}
};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Type};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Sha256, Digest};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use jsonwebtoken::errors::ErrorKind;
use std::time::SystemTime;
use sqlx::sqlite::SqliteQueryResult;
use crate::AppState;

const SALT_LENGTH: usize = 12;

#[derive(Deserialize, Type, Serialize, PartialEq)]
pub enum AuthLevel {
    User,
    Admin
}

impl From<i64> for AuthLevel {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::User,
            1 => Self::Admin,
            _ => panic!("Unsupported value when converting an i64 to an AuthLevel")
        }
    }
}

#[derive(Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub auth_level: AuthLevel,
    pub salt: String
}

#[derive(Serialize)]
pub struct ReturnUser {
    pub id: i64,
    pub username: String
}

impl From<User> for ReturnUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    username: String,
    password: String
}

// ------------------------------------------------------------------------------------------------
// JWT

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize, // UTC timestamp, expiration
    iat: usize, // UTC timestamp, time issued at
    sub: String
}

fn encode_jwt(app_secret: &str, user_id: i64, jwt_lifetime: usize) -> String {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("System time set to before UNIX_EPOCH").as_secs() as usize;
    let expires_at = now + jwt_lifetime;
    let claims = Claims {
        exp: expires_at,
        iat: now,
        sub: user_id.to_string()
    };
    // We are using the default Header algorithm so this should be infallible
    encode(&Header::default(), &claims, &EncodingKey::from_secret(app_secret.as_bytes())).unwrap()
}

fn decode_jwt(web_token: &str, app_secret: &str) -> Result<i64, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp", "iat", "sub"]);
    let claims = match decode::<Claims>(web_token, &DecodingKey::from_secret(app_secret.as_bytes()), &validation) {
        Ok(c) => c,
        Err(err) => {
            let err_msg = match *err.kind() {
                ErrorKind::InvalidToken => "Invalid JWT",
                ErrorKind::InvalidSubject => "Invalid JWT",
                ErrorKind::ExpiredSignature => "JWT has expired",
                ErrorKind::InvalidSignature => "Invalid JWT Signature",
                _ => "Unknown error while processing JWT"
            };
            return Err(err_msg.to_string())
        }
    };
    match claims.claims.sub.parse::<i64>() {
        Ok(id) => Ok(id),
        Err(_) => Err("Failed to parse user ID from JWT".to_string())
    }
}

// ------------------------------------------------------------------------------------------------
// Endpoints

// TODO: The error handling here is pretty bad. It was done quickly to get an MVP off the ground
//       but it really needs to be refactored

pub fn auth_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/user", post(create_user))
        .route("/user", get(get_user_by_username))
        .route("/user/:user_id", delete(delete_user_by_id))
        .route("/user/me", delete(delete_user_me))
        .route("/user/auth", post(auth_user))
}
// TODO: GET user/me and/or /user/:user_id
// TODO: PUT user/me and/or /user/:user_id

async fn auth_user(State(app_state): State<AppState>, Json(credentials): Json<LoginUserSchema>) -> Result<(StatusCode, Json<String>), (StatusCode, Json<String>)> {
    let pool = &app_state.db;
    match sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", credentials.username)
        .fetch_optional(pool)
        .await {
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
                }
                else {
                    Err((StatusCode::NOT_FOUND, Json(format!("No such user with username '{}', or invalid password", credentials.username))))
                }
            },
            None => Err((StatusCode::NOT_FOUND, Json(format!("No such user with username '{}', or invalid password", credentials.username))))
        }
        Err(_) => panic!("TODO: Auth user error")
    }
}

async fn create_user(State(app_state): State<AppState>, Json(credentials): Json<LoginUserSchema>) -> Result<(StatusCode, Json<ReturnUser>), (StatusCode, Json<String>)> {
    let pool = &app_state.db;

    // Check the database if this username is already taken
    if db_get_user_by_username(pool, credentials.username.as_str()).await.is_some() {
        return Err((StatusCode::BAD_REQUEST, Json(format!("Username '{}' is already taken", credentials.username))))
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
    ).execute(pool).await;

    match db_get_user_by_username(pool, credentials.username.as_str()).await {
        Some(user) => Ok((StatusCode::CREATED, Json(Into::<ReturnUser>::into(user)))),
        // TODO: Actual error handling here
        None => Err((StatusCode::INTERNAL_SERVER_ERROR, Json("Internal error".to_owned())))
    }
}

async fn get_user_by_username(State(app_state): State<AppState>, headers: HeaderMap, query_params: Query<HashMap<String, String>>) -> Result<(StatusCode, Json<ReturnUser>), (StatusCode, Json<String>)> {
    // Get the username out of query params
    let username = match query_params.get("username") {
        Some(username) => username,
        None => return Err((StatusCode::BAD_REQUEST, Json("Missing necessary 'username' query param".to_string())))
    };

    // Get the JWT from the Authorization header and decode it
    // We don't need to do anything with this data, but we do need to validate that a real user made the request
    let _token_user_id = match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::BAD_REQUEST, Json(e)))
    };

    // Find and return the user
    match db_get_user_by_username(&app_state.db, username.as_str()).await {
        Some(user) => Ok((StatusCode::OK, Json(user.into()))),
        None => Err((StatusCode::NOT_FOUND, Json("Could not find user with the given ID".to_string())))
    }
}

async fn delete_user_by_id(State(app_state): State<AppState>, headers: HeaderMap, Path(user_id): Path<i64>) -> Result<StatusCode, (StatusCode, Json<String>)> {
    // Get the JWT from the Authorization header and decode it
    let token_user_id = match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::BAD_REQUEST, Json(e)))
    };

    // Get the account of the id from the path
    let user = match db_get_user_by_id(&app_state.db, user_id).await {
        Some(user) => user,
        None => return Err((StatusCode::NOT_FOUND, Json("Could not find user with the given ID".to_string())))
    };

    // Check if the requester id matches the account being deleted
    if user.id != token_user_id {
        // The token user's ID does not match the user being deleted
        // Get the token user's account and check to see if they are an admin
        match db_get_user_by_id(&app_state.db, token_user_id).await {
            Some(user) => {
                if user.auth_level != AuthLevel::Admin {
                    return Err((StatusCode::FORBIDDEN, Json("Cannot delete an account you do not have access to".to_string())))
                }
            },
            None => return Err((StatusCode::NOT_FOUND, Json("Could not find user matching the Authorization token".to_string())))
        };
    };

    // Delete the user
    match db_delete_user(&app_state.db, user.id).await {
        // TODO: Should check .rows_affected() on the return here and error if it is anything other than 1
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to delete user".to_string())))
    }
}

async fn delete_user_me(State(app_state): State<AppState>, headers: HeaderMap) -> Result<StatusCode, (StatusCode, Json<String>)>  {
    // Get the JWT from the Authorization header and decode it
    let token_user_id = match get_and_decode_auth_token(&headers, app_state.config.app_secret.as_str()) {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::BAD_REQUEST, Json(e)))
    };

    // Delete the user
    match db_delete_user(&app_state.db, token_user_id).await {
        // TODO: Should check .rows_affected() on the return here and error if it is anything other than 1
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to delete user".to_string())))
    }
}

// ------------------------------------------------------------------------------------------------
// Misc

fn get_and_decode_auth_token(headers: &HeaderMap, app_secret: &str) -> Result<i64, String> {
    let token = match get_auth_token(&headers) {
        Ok(t) => t,
        Err(e) => return Err(e)
    };
    match decode_jwt(token, app_secret) {
        Ok(id) => Ok(id),
        Err(e) => return Err(e)
    }
}

fn get_auth_token(headers: &HeaderMap) -> Result<&str, String> {
    let token = match headers.get(AUTHORIZATION) {
        Some(t) => t,
        None => return Err("Missing 'Authorization' header".to_owned())
    };
    match token.to_str() {
        Ok(as_str) => Ok(as_str),
        Err(_) => Err("Failed to read 'Authorization' header".to_owned())
    }
}

async fn db_get_user_by_username(pool: &SqlitePool, username: &str) -> Option<User> {
    match sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", username)
        .fetch_one(pool)
        .await {
        Ok(user) => Some(user),
        // TODO: Actually handle this error
        Err(_) => None
    }
}

async fn db_delete_user(pool: &SqlitePool, user_id: i64) -> Result<SqliteQueryResult, ()> {
    match sqlx::query!(
        "DELETE FROM users WHERE id = ?",
        user_id
    ).execute(pool).await {
        // Should add actual error handling here, for now all we care about is whether or not
        // the query succeeded or failed
        Ok(res) => Ok(res),
        Err(_) => Err(())
    }
}

async fn db_get_user_by_id(pool: &SqlitePool, id: i64) -> Option<User> {
    match sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(pool)
        .await {
        Ok(user) => Some(user),
        // TODO: Actually handle this error
        Err(_) => None
    }
}

fn hash_password(mut password: String, salt: &str) -> String {
    password.push_str(salt);
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    format!("{:X}", result)
}

fn generate_salt() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(SALT_LENGTH)
        .map(char::from)
        .collect()
}
