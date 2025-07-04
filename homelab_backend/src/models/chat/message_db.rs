use super::super::id_to_string;
use super::chat_channel_db::get_chat_channel_by_id;
use super::message::{ChatMessage, CreateMessageSchema};
use crate::db::resource_kinds::ResourceKind;
use crate::error_handler::DbError;
use crate::models::chat::chat_channel::ChatChannel;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::options::{ReadConcern, WriteConcern};
use mongodb::{
    bson::Bson,
    bson::{doc, Document},
    Collection, Database,
};

pub async fn insert_chat_message(pool: &Database, data: CreateMessageSchema, user_id: &str) -> Result<ChatMessage, DbError> {
    let collection: Collection<Document> = pool.collection("chat_messages");

    // Are there any edge case issues caused by getting the atomic ID outside a session? What happens
    // when two messages are received at the exact same time?
    let channel = get_chat_channel_by_id(pool, data.channel_id.as_str()).await?;

    // Begin a session to create a chat message and update the relevant chat channel
    let mut session = pool.client().start_session().await?;
    // Read/write concern taken from the docs, unsure how relevant they are here
    session
        .start_transaction()
        .read_concern(ReadConcern::majority())
        .write_concern(WriteConcern::majority())
        .await?;

    // Create a message
    let new_atomic_id = channel.most_recent_message_id + 1;
    let doc = data.create_message_doc(user_id, new_atomic_id);
    let res = match collection.insert_one(doc).session(&mut session).await {
        Ok(res) => res,
        Err(e) => {
            session.abort_transaction().await?;
            return Err(e.into());
        }
    };

    // When a message is created, increment the most recent message ID on the channel
    let channel_collection: Collection<ChatChannel> = pool.collection("chat_channels");
    // TODO: This string -> ObjectId is copy/pasted in multiple places, should be a method probably
    //       in the db file
    let bson_id: ObjectId = match channel.id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let filter_doc = doc! { "_id": Bson::ObjectId(bson_id) };
    let update = doc! { "$set": { "most_recent_message_id": new_atomic_id }};
    match channel_collection.update_one(filter_doc, update).await {
        Ok(_) => (),
        Err(e) => {
            session.abort_transaction().await?;
            return Err(e.into());
        }
    }

    session.commit_transaction().await?;

    let id = id_to_string(res.inserted_id).expect("InsertOneResult.inserted_id should always be an ObjectId");
    get_chat_message_by_id(pool, id.as_str()).await
}

pub async fn get_chat_message_by_id(pool: &Database, message_id: &str) -> Result<ChatMessage, DbError> {
    let collection: Collection<ChatMessage> = pool.collection("chat_messages");
    let msg_id: ObjectId = match message_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! {"_id": Bson::ObjectId(msg_id)};
    match collection.find_one(doc).await {
        Ok(maybe_record) => match maybe_record {
            Some(record) => Ok(record),
            None => Err(DbError::NotFound(ResourceKind::ChatMessage, message_id.to_owned())),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn get_chat_message_span(pool: &Database, atomic_id: i64, channel_id: &str, message_count: i64) -> Result<Vec<ChatMessage>, DbError> {
    let collection: Collection<ChatMessage> = pool.collection("chat_messages");
    let lower_range = (atomic_id - message_count).max(0);
    let doc = doc! {
        "channel_id": channel_id,
        "$and": [
            {"atomic_id": {"$lte": atomic_id} },
            {"atomic_id": {"$gt": lower_range} }
        ]
    };
    let sort = doc! {"atomic_id": 1};
    match collection.find(doc).sort(sort).await {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
    }
}
