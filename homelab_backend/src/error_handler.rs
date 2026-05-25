use crate::api::return_data::ReturnData;
use crate::models::chat::packet::WebSocketResponse;
use mongodb::error::{ErrorKind, WriteFailure};
use std::sync::Arc;

#[derive(Debug)]
pub enum DbError {
    AlreadyExists,
    NotFound(&'static str),
    RelationshipViolation(&'static str, String),
    EmptyDbExpression(&'static str, String),
    BadId,
    AuthFailure,
    CustomMongoFailure(String),
    UnhandledException(String),
}

// Translate MongoDB errors into our error struct
// TODO: Handle more db errors
impl From<mongodb::error::Error> for DbError {
    fn from(value: mongodb::error::Error) -> Self {
        match *value.kind {
            ErrorKind::InvalidArgument { .. } => DbError::UnhandledException("Invalid argument to database".to_owned()),
            ErrorKind::Authentication { .. } => DbError::AuthFailure,
            ErrorKind::Write(write_failure) => match write_failure {
                WriteFailure::WriteConcernError(_write_concern_error) => DbError::UnhandledException("Unhandled error while writing data".to_owned()),
                WriteFailure::WriteError(write_error) => match write_error.code {
                    // TODO: write_error.message and write_error.details may have useful info to return
                    //       to the user here, check them?
                    11000 => DbError::AlreadyExists, // DuplicateKey error
                    _ => DbError::UnhandledException("Unhandled error while writing data".to_owned()),
                },
                _ => DbError::UnhandledException("Unhandled error while writing data".to_owned()),
            },
            ErrorKind::Custom(custom_message) => {
                if let Ok(custom_error_message) = custom_message.downcast::<String>() {
                    if let Some(owned_error_message) = Arc::into_inner(custom_error_message) {
                        return DbError::CustomMongoFailure(owned_error_message);
                    }
                }
                DbError::UnhandledException("Unhandled failure while resolving custom MongoDB error".to_string())
            }
            _ => DbError::UnhandledException("Unhandled failure".to_owned()),
        }
    }
}

impl<T> From<DbError> for ReturnData<T> {
    fn from(value: DbError) -> Self {
        match value {
            DbError::AlreadyExists => ReturnData::bad_request("Tried to create a resource which violated a unique constraint"),
            DbError::NotFound(resource_type) => ReturnData::not_found(format!("{resource_type} not found")),
            DbError::RelationshipViolation(resource_type, identifier) => ReturnData::bad_request(format!(
                "The request violates a relationship constraint on {resource_type} with identifier {identifier}"
            )),
            DbError::EmptyDbExpression(resource_type, operation) => {
                ReturnData::bad_request(format!("Received no data while {operation} {resource_type}, resulting in a no-op"))
            }
            DbError::BadId => ReturnData::not_found("The provided ID was not valid"),
            DbError::AuthFailure => ReturnData::unauthorized("Auth failure while reading database"),
            DbError::CustomMongoFailure(custom_message) => ReturnData::bad_request(custom_message),
            DbError::UnhandledException(error_while) => {
                ReturnData::internal_error(format!("Unhandled exception when making a database request: {error_while}"))
            }
        }
    }
}

impl From<DbError> for WebSocketResponse {
    fn from(value: DbError) -> Self {
        match value {
            DbError::AlreadyExists => WebSocketResponse::bad_request("Tried to create a resource which violated a unique constraint"),
            DbError::NotFound(resource_type) => WebSocketResponse::not_found(format!("{resource_type} not found")),
            DbError::RelationshipViolation(resource_type, identifier) => WebSocketResponse::bad_request(format!(
                "The request violates a relationship constraint on {resource_type} with identifier {identifier}"
            )),
            DbError::EmptyDbExpression(resource_type, operation) => {
                WebSocketResponse::bad_request(format!("Received no data while {operation} {resource_type}, resulting in a no-op"))
            }
            DbError::BadId => WebSocketResponse::bad_request("The provided ID was not valid"),
            DbError::AuthFailure => WebSocketResponse::unauthorized("Auth failure while reading database"),
            DbError::CustomMongoFailure(custom_message) => WebSocketResponse::bad_request(custom_message),
            DbError::UnhandledException(error_while) => {
                WebSocketResponse::internal_error(format!("Unhandled exception when making a database request: {error_while}"))
            }
        }
    }
}

pub enum ServerError {
    FailedAuthentication(String),
    InternalFailure(String),
}

impl<T> From<ServerError> for ReturnData<T> {
    fn from(value: ServerError) -> Self {
        match value {
            ServerError::FailedAuthentication(reason) => ReturnData::unauthorized(format!("Auth failure: {reason}")),
            ServerError::InternalFailure(failure_while) => ReturnData::internal_error(format!("Internal server error while: {failure_while}")),
        }
    }
}
