use crate::api::return_data::ReturnData;

pub enum DbError {
    AlreadyExists(String, String),
    NotFound(String, String),
    UnhandledException,
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
            DbError::UnhandledException => ReturnData::internal_error(
                "Unhandled exception when making a database request".to_owned(),
            ),
        }
    }
}
