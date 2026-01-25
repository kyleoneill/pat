use mongodb::error::ErrorKind;

use crate::api::return_data::ReturnData;

#[derive(Debug)]
pub enum DbError {
    AlreadyExists(&'static str, String),
    NotFound(&'static str),
    RelationshipViolation(&'static str, String),
    EmptyDbExpression(&'static str, String),
    BadId,
    AuthFailure,
    UnhandledException(String),
}

// Translate MongoDB errors into our error struct
// TODO: Handle more db errors
impl From<mongodb::error::Error> for DbError {
    fn from(value: mongodb::error::Error) -> Self {
        match *value.kind {
            ErrorKind::InvalidArgument { .. } => DbError::UnhandledException("Invalid argument to database".to_owned()),
            ErrorKind::Authentication { .. } => DbError::AuthFailure,
            _ => DbError::UnhandledException("Unhandled failure".to_owned()),
        }
    }
}

impl<T> From<DbError> for ReturnData<T> {
    fn from(value: DbError) -> Self {
        match value {
            DbError::AlreadyExists(resource_type, identifier) => {
                ReturnData::bad_request(format!("A {resource_type} with identifier {identifier} already exists"))
            }
            DbError::NotFound(resource_type) => ReturnData::not_found(format!("{resource_type} not found")),
            DbError::RelationshipViolation(resource_type, identifier) => ReturnData::bad_request(format!(
                "The request violates a relationship constraint on {resource_type} with identifier {identifier}"
            )),
            DbError::EmptyDbExpression(resource_type, operation) => {
                ReturnData::bad_request(format!("Received no data while {operation} {resource_type}, resulting in a no-op"))
            }
            DbError::BadId => ReturnData::not_found("The provided ID was not valid".to_owned()),
            DbError::AuthFailure => ReturnData::unauthorized("Auth failure while reading database".to_owned()),
            DbError::UnhandledException(error_while) => {
                ReturnData::internal_error(format!("Unhandled exception when making a database request: {error_while}"))
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
