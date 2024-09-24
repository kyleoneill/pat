use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

pub struct ReturnData<T, E> {
    status_code: StatusCode,
    data: Result<T, E>,
}

impl<T, E> ReturnData<T, E> {
    // Failures
    pub fn not_found(error: E) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            data: Err(error),
        }
    }
    pub fn internal_error(error: E) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            data: Err(error),
        }
    }
    pub fn bad_request(error: E) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            data: Err(error),
        }
    }
    pub fn forbidden(error: E) -> Self {
        Self {
            status_code: StatusCode::FORBIDDEN,
            data: Err(error),
        }
    }

    pub fn unauthorized(error: E) -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            data: Err(error),
        }
    }

    // Success
    pub fn ok(data: T) -> Self {
        Self {
            status_code: StatusCode::OK,
            data: Ok(data),
        }
    }
    pub fn created(data: T) -> Self {
        Self {
            status_code: StatusCode::CREATED,
            data: Ok(data),
        }
    }
}

impl<T: Serialize, E: Serialize> IntoResponse for ReturnData<T, E> {
    fn into_response(self) -> Response {
        match self.data {
            Ok(data) => (self.status_code, Json(data)).into_response(),
            Err(error) => (self.status_code, Json(error)).into_response(),
        }
    }
}
