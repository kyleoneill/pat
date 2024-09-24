use crate::api::return_data::ReturnData;

pub enum DbError {
    AlreadyExists(String, String),
    NotFound(String, String),
    RelationshipViolation(String, String),
    UnhandledException,
    EmptySQLExpression(String, String),
}

impl<T> From<DbError> for ReturnData<T, String> {
    fn from(value: DbError) -> Self {
        match value {
            DbError::AlreadyExists(resource_type, resource) => ReturnData::bad_request(format!(
                "A {} with value {} already exists",
                resource_type, resource
            )),
            DbError::NotFound(resource_type, resource_slug) => ReturnData::not_found(format!(
                "{} with identifier {} not found",
                resource_type, resource_slug
            )),
            DbError::RelationshipViolation(resource_type, identifier) => {
                ReturnData::bad_request(format!(
                    "The request violates a relationship constraint on {} with identifier {}",
                    resource_type, identifier
                ))
            }
            DbError::EmptySQLExpression(operation, data_type) => ReturnData::bad_request(format!(
                "Received no data while {} {}, resulting in a no-op",
                operation, data_type
            )),
            DbError::UnhandledException => ReturnData::internal_error(
                "Unhandled exception when making a database request".to_owned(),
            ),
        }
    }
}

pub enum InternalError {
    FailedAuthentication,
}

impl<T> From<InternalError> for ReturnData<T, String> {
    fn from(value: InternalError) -> Self {
        match value {
            InternalError::FailedAuthentication => {
                ReturnData::unauthorized("Invalid authorization token".to_owned())
            }
        }
    }
}
