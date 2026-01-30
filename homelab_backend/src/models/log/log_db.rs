use crate::{
    db::{str_to_object_id, MongoModel, PatDatabase},
    error_handler::DbError,
    models::log::Log,
};
use mongodb::bson::{doc, oid::ObjectId, Bson};

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

pub async fn db_get_logs_for_user(db_handle: &PatDatabase, user_id: String) -> Result<Vec<Log>, DbError> {
    let doc = doc! { "user_id": user_id };
    db_handle.find(doc).await
}

pub async fn db_get_log_by_id(db_handle: &PatDatabase, log_id: &str) -> Result<Log, DbError> {
    let bson_id = str_to_object_id(log_id)?;
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };
    db_handle.find_one(doc).await
}
