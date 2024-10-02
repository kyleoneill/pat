use axum::http::header::HeaderMap;
use sqlx::SqlitePool;

pub mod log_controller;
pub mod reminder_controller;
pub mod return_data;
pub mod user_controller;

use crate::error_handler::InternalError;
use crate::models::user::jwt::get_and_decode_auth_token;
use crate::models::user::user_db::db_get_user_by_id;
use crate::models::user::User;

// TODO: This should return an error struct that impls a response so it can be ?'d in an endpoint
//       which would cause the endpoint to return a 403 like "Invalid authentication"
pub async fn get_user_from_token(
    pool: &SqlitePool,
    headers: &HeaderMap,
    app_secret: &str,
) -> Result<User, InternalError> {
    match get_and_decode_auth_token(headers, app_secret) {
        Ok(user_id) => match db_get_user_by_id(pool, user_id).await {
            Ok(user) => Ok(user),
            Err(_) => Err(InternalError::FailedAuthentication),
        },
        Err(_e) => Err(InternalError::FailedAuthentication),
    }
}

// TODO:
// When the `Never` type and the `std::ops::FromResidual` trait are both stabilized (they are
// currently nightly) this can be made cleaner. get_user_from_token can return a
// `Result<User, ReturnData<!, String>>`. When used, the method can be question marked like
// `let user = get_user_from_token(pool, &headers, &app_state.config.app_secret).await?;`
// and the `FromResidual` trait will take care of converting the `ReturnData<!, String>` into a
// `ReturnData<T, String>` for the endpoint method
