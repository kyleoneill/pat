use super::Log;
use crate::db::resource_kinds::ResourceKind;
use crate::error_handler::DbError;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Bson};
use mongodb::{Collection, Database};
// TODO: There should be actual error handling here

pub async fn db_get_logs_for_user(pool: &Database, user_id: String) -> Result<Vec<Log>, DbError> {
    let collection: Collection<Log> = pool.collection("logs");
    let doc = doc! { "user_id": user_id };
    match collection.find(doc).await {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn db_get_log_by_id(pool: &Database, log_id: String) -> Result<Log, DbError> {
    let collection: Collection<Log> = pool.collection("logs");
    let bson_id: ObjectId = match log_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };
    match collection.find_one(doc).await {
        Ok(maybe_doc) => match maybe_doc {
            Some(log) => Ok(log),
            None => Err(DbError::NotFound(ResourceKind::Log, log_id.to_owned().to_string())),
        },
        Err(e) => Err(e.into()),
    }
}
