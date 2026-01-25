use futures::TryStreamExt;

use mongodb::{
    bson::{doc, Bson},
    Collection,
};

use crate::{
    db::{str_to_object_id, PatDatabase},
    error_handler::DbError,
    models::log::Log,
};

pub async fn db_get_logs_for_user(db_handle: &PatDatabase, user_id: String) -> Result<Vec<Log>, DbError> {
    let collection: Collection<Log> = db_handle.get_collection();
    let doc = doc! { "user_id": user_id };
    match collection.find(doc).await {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn db_get_log_by_id(db_handle: &PatDatabase, log_id: &str) -> Result<Log, DbError> {
    let bson_id = str_to_object_id(log_id)?;
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };
    db_handle.find_one(doc).await
}
