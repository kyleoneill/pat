use crate::{
    db::{MongoModel, PatDatabase, str_to_object_id},
    error_handler::DbError,
    models::log::Log,
};
use mongodb::bson::{Bson, doc};

impl MongoModel for Log {
    fn collection_name() -> &'static str {
        "logs"
    }
    fn model_name() -> &'static str {
        "Log"
    }
    fn get_id(&self) -> &str {
        self.id.as_str()
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
