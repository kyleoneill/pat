use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;
use serde_json::value::Value;

pub struct ReturnData<T> {
    status_code: StatusCode,
    data: Result<T, Value>,
}

impl<T> ReturnData<T> {
    // 2xx
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

    // 4xx
    pub fn not_found(error: String) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            data: Err(json!({"msg": error})),
        }
    }
    pub fn bad_request(error: String) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            data: Err(json!({"msg": error})),
        }
    }
    pub fn forbidden(error: String) -> Self {
        Self {
            status_code: StatusCode::FORBIDDEN,
            data: Err(json!({"msg": error})),
        }
    }
    pub fn unauthorized(error: String) -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            data: Err(json!({"msg": error})),
        }
    }

    // 500
    pub fn internal_error(error: String) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            data: Err(json!({"msg": error})),
        }
    }
}

impl<T: Serialize> IntoResponse for ReturnData<T> {
    fn into_response(self) -> Response {
        match self.data {
            Ok(data) => (self.status_code, Json(data)).into_response(),
            Err(error) => (self.status_code, Json(error)).into_response(),
        }
    }
}
