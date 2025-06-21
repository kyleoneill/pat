use axum::http::header::{HeaderMap, AUTHORIZATION};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize, // UTC timestamp, expiration
    iat: usize, // UTC timestamp, time issued at
    sub: String,
}

pub fn encode_jwt(app_secret: &str, user_id: String, jwt_lifetime: usize) -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time set to before UNIX_EPOCH")
        .as_secs() as usize;
    let expires_at = now + jwt_lifetime;
    let claims = Claims {
        exp: expires_at,
        iat: now,
        sub: user_id,
    };
    // We are using the default Header algorithm so this should be infallible
    encode(&Header::default(), &claims, &EncodingKey::from_secret(app_secret.as_bytes())).unwrap()
}

pub fn decode_jwt(web_token: &str, app_secret: &str) -> Result<String, String> {
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
                _ => "Unknown error while processing JWT",
            };
            return Err(err_msg.to_string());
        }
    };
    match claims.claims.sub.parse::<String>() {
        Ok(id) => Ok(id),
        Err(_) => Err("Failed to parse user ID from JWT".to_string()),
    }
}

pub fn get_and_decode_auth_token(headers: &HeaderMap, app_secret: &str) -> Result<String, String> {
    let token = get_auth_token(headers)?;
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
