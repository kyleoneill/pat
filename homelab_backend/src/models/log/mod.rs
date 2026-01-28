pub mod log_db;

use super::deserialize_id;
use crate::{db::MongoModel, error_handler::DbError};
use hyper::body::Bytes;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Log {
    #[serde(rename = "_id", deserialize_with = "deserialize_id")]
    id: String,
    pub method: String,
    pub uri: String,
    pub user_id: String,
    date_time: i64,
}

impl MongoModel for Log {
    fn collection_name() -> &'static str {
        "logs"
    }
    fn model_name() -> &'static str {
        "Log"
    }
    fn mongo_id(&self) -> Result<ObjectId, DbError> {
        match self.id.parse::<ObjectId>() {
            Ok(res) => Ok(res),
            Err(_) => Err(DbError::BadId),
        }
    }
}

// TODO: Should have a log retention policy/task which auto deletes old tasks.
//       Tasks older than a configurable age will be auto deleted by a task.
//       Might be fine without this for awhile with an app only being used by myself

impl Log {
    #[allow(dead_code)] // Used in test
    pub fn from_bytes_to_vec(input: &Bytes) -> Vec<Self> {
        serde_json::from_slice(input).unwrap()
    }
}
