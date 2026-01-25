use axum::http::header::HeaderMap;

pub mod chat_controller;
pub mod games_controller;
pub mod log_controller;
pub mod reminder_controller;
pub mod return_data;
pub mod user_controller;

use crate::{
    db::PatDatabase,
    error_handler::{DbError, ServerError},
    models::user::{
        jwt::{decode_jwt, get_and_decode_auth_token},
        user_db::db_get_user_by_id,
        User,
    },
};

async fn get_user(db_handle: &PatDatabase, maybe_user_id: Result<String, String>) -> Result<User, ServerError> {
    match maybe_user_id {
        Ok(user_id) => match db_get_user_by_id(db_handle, user_id.as_str()).await {
            Ok(user) => Ok(user),
            Err(e) => match e {
                DbError::NotFound(resource_kind) => Err(ServerError::FailedAuthentication(format!("Could not find {resource_kind}"))),
                _ => Err(ServerError::InternalFailure("Authenticating".to_owned())),
            },
        },
        Err(failed_auth_reason) => Err(ServerError::FailedAuthentication(failed_auth_reason)),
    }
}

pub async fn get_user_from_auth_header(db_handle: &PatDatabase, headers: &HeaderMap, app_secret: &str) -> Result<User, ServerError> {
    let maybe_id = get_and_decode_auth_token(headers, app_secret);
    get_user(db_handle, maybe_id).await
}

pub async fn get_user_from_token(db_handle: &PatDatabase, token: &str, app_secret: &str) -> Result<User, ServerError> {
    let maybe_id = decode_jwt(token, app_secret);
    get_user(db_handle, maybe_id).await
}

// TODO:
// When the `Never` type and the `std::ops::FromResidual` trait are both stabilized (they are
// currently nightly) this can be made cleaner. get_user_from_auth_header can return a
// `Result<User, ReturnData<!, String>>`. When used, the method can be question marked like
// `let user = get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await?;`
// and the `FromResidual` trait will take care of converting the `ReturnData<!, String>` into a
// `ReturnData<T, String>` for the endpoint method
