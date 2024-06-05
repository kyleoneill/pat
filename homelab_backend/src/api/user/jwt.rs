use crate::api::user::db::db_get_user_by_id;
use crate::api::user::User;
use axum::http::header::{HeaderMap, AUTHORIZATION};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize, // UTC timestamp, expiration
    iat: usize, // UTC timestamp, time issued at
    sub: String,
}

pub fn encode_jwt(app_secret: &str, user_id: i64, jwt_lifetime: usize) -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time set to before UNIX_EPOCH")
        .as_secs() as usize;
    let expires_at = now + jwt_lifetime;
    let claims = Claims {
        exp: expires_at,
        iat: now,
        sub: user_id.to_string(),
    };
    // We are using the default Header algorithm so this should be infallible
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_secret.as_bytes()),
    )
    .unwrap()
}

fn decode_jwt(web_token: &str, app_secret: &str) -> Result<i64, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp", "iat", "sub"]);
    let claims = match decode::<Claims>(
        web_token,
        &DecodingKey::from_secret(app_secret.as_bytes()),
        &validation,
    ) {
        Ok(c) => c,
        Err(err) => {
            let err_msg = match *err.kind() {
                ErrorKind::InvalidToken => "Invalid JWT",
                ErrorKind::InvalidSubject => "Invalid JWT",
                ErrorKind::ExpiredSignature => "JWT has expired",
                ErrorKind::InvalidSignature => "Invalid JWT Signature",
                _ => "Unknown error while processing JWT",
            };
            return Err(err_msg.to_string());
        }
    };
    match claims.claims.sub.parse::<i64>() {
        Ok(id) => Ok(id),
        Err(_) => Err("Failed to parse user ID from JWT".to_string()),
    }
}

pub async fn get_user_from_token(
    pool: &SqlitePool,
    headers: &HeaderMap,
    app_secret: &str,
) -> Result<User, String> {
    let user_id = get_and_decode_auth_token(headers, app_secret)?;
    match db_get_user_by_id(pool, user_id).await {
        Some(user) => Ok(user),
        None => Err("No user found for the given authorization token".to_owned()),
    }
}

pub fn get_and_decode_auth_token(headers: &HeaderMap, app_secret: &str) -> Result<i64, String> {
    let token = match get_auth_token(headers) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };
    decode_jwt(token, app_secret)
}

pub fn get_auth_token(headers: &HeaderMap) -> Result<&str, String> {
    let token = match headers.get(AUTHORIZATION) {
        Some(t) => t,
        None => return Err("Missing 'Authorization' header".to_owned()),
    };
    match token.to_str() {
        Ok(as_str) => Ok(as_str),
        Err(_) => Err("Failed to read 'Authorization' header".to_owned()),
    }
}
