use axum::http::header::HeaderMap;
use sqlx::SqlitePool;

pub mod logs;
pub mod notes;
pub mod user;
use user::User;

// TODO: This should return an error struct that impls a response so it can be ?'d in an endpoint
//       which would cause the endpoint to return a 403 like "Invalid authentication"
pub async fn get_user_from_token(
    pool: &SqlitePool,
    headers: &HeaderMap,
    app_secret: &str,
) -> Result<User, String> {
    let user_id = user::jwt::get_and_decode_auth_token(headers, app_secret)?;
    match user::db::db_get_user_by_id(pool, user_id).await {
        Some(user) => Ok(user),
        None => Err("No user found for the given authorization token".to_owned()),
    }
}
