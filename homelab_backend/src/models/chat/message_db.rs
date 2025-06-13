use super::super::id_to_string;
use super::message::{ChatMessage, CreateMessageSchema};
use crate::db::resource_kinds::ResourceKind;
use crate::error_handler::DbError;
use mongodb::bson::oid::ObjectId;
use mongodb::{
    bson::Bson,
    bson::{doc, Document},
    Collection, Database,
};

pub async fn insert_chat_message(
    pool: &Database,
    data: CreateMessageSchema,
    user_id: &str,
) -> Result<ChatMessage, DbError> {
    let collection: Collection<Document> = pool.collection("chat_messages");
    let doc = data.create_message_doc(user_id);
    let res = match collection.insert_one(doc).await {
        Ok(res) => res,
        Err(e) => {
            return match *e.kind {
                _ => Err(e.into()),
            }
        }
    };
    let id = id_to_string(res.inserted_id)
        .expect("InsertOneResult.inserted_id should always be an ObjectId");
    get_chat_message_by_id(pool, id.as_str()).await
}

pub async fn get_chat_message_by_id(
    pool: &Database,
    message_id: &str,
) -> Result<ChatMessage, DbError> {
    let collection: Collection<ChatMessage> = pool.collection("chat_messages");
    let msg_id: ObjectId = match message_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! {"_id": Bson::ObjectId(msg_id)};
    match collection.find_one(doc).await {
        Ok(maybe_record) => match maybe_record {
            Some(record) => Ok(record),
            None => Err(DbError::NotFound(
                ResourceKind::ChatMessage,
                message_id.to_owned(),
            )),
        },
        Err(e) => Err(e.into()),
    }
}
