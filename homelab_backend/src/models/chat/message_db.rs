use super::chat_channel_db::get_chat_channel_by_id;
use super::message::{ChatMessage, CreateMessageSchema};
use crate::db::resource_kinds::ResourceKind;
use crate::error_handler::DbError;
use crate::models::chat::chat_channel::ChatChannel;
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::error::{TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT};
use mongodb::options::{ReadConcern, WriteConcern};
use mongodb::{
    bson::Bson,
    bson::{doc, Document},
    error::Error as MongoError,
    ClientSession, Collection, Database,
};

pub async fn insert_chat_message(pool: &Database, data: CreateMessageSchema, user_id: &str) -> Result<ChatMessage, DbError> {
    let chat_message_collection: Collection<Document> = pool.collection("chat_messages");
    let channel_collection: Collection<ChatChannel> = pool.collection("chat_channels");

    // Begin a session to create a chat message and update the relevant chat channel
    let mut session = pool.client().start_session().await?;
    // Read/write concern taken from the docs, unsure how relevant they are here
    session
        .start_transaction()
        .read_concern(ReadConcern::majority())
        .write_concern(WriteConcern::majority())
        .await?;

    let mut loop_counter = 0;
    while let Err(error) = execute_chat_message_transaction(&chat_message_collection, &channel_collection, &mut session, data.clone(), user_id).await
    {
        // Emergency safety valve to stop an infinite hang if mongo behaves strangely
        loop_counter += 1;
        if loop_counter > 500 {
            return Err(MongoError::custom("Exceeded retries").into());
        }
        if !error.contains_label(TRANSIENT_TRANSACTION_ERROR) {
            return Err(error.into());
        }
    }

    let channel = get_chat_channel_by_id(pool, data.channel_id.as_str()).await?;

    get_chat_message_by_atomic_id(pool, channel.id.as_str(), channel.most_recent_message_id).await
}

// TODO: Session pattern should be more generic so I don't need to do this every time I use a session in the future
async fn execute_chat_message_transaction(
    chat_message_collection: &Collection<Document>,
    channel_collection: &Collection<ChatChannel>,
    session: &mut ClientSession,
    data: CreateMessageSchema,
    user_id: &str,
) -> Result<(), MongoError> {
    // Get the channel associated with this message
    let channel_id: ObjectId = match data.channel_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(MongoError::custom("Failed to parse ObjectId of a channel")),
    };
    let filter_doc = doc! {"_id": Bson::ObjectId(channel_id.clone())};
    let channel = match channel_collection.find_one(filter_doc).session(&mut *session).await? {
        Some(channel) => channel,
        None => return Err(MongoError::custom("Failed to find a chat channel with the given ID")),
    };

    // Create a message
    let new_atomic_id = channel.most_recent_message_id + 1;
    let insert_message_doc = data.create_message_doc(user_id, new_atomic_id);

    // When a message is created, also increment the most recent message ID on the channel
    let channel_filter_doc = doc! { "_id": Bson::ObjectId(channel_id) };
    let channel_update = doc! { "$set": { "most_recent_message_id": new_atomic_id }};

    chat_message_collection.insert_one(insert_message_doc).session(&mut *session).await?;
    channel_collection
        .update_one(channel_filter_doc, channel_update)
        .session(&mut *session)
        .await?;
    let mut loop_counter = 0;
    loop {
        // Emergency safety valve to stop an infinite hang if mongo behaves strangely
        loop_counter += 1;
        if loop_counter > 500 {
            return Err(MongoError::custom("Exceeded retries"));
        }
        let result = session.commit_transaction().await;
        if let Err(error) = result {
            if error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                continue;
            }
            return Err(error);
        }
        // TODO: Mongo docs here has `result?` which will go back to the start of the loop and try
        // the transaction again, but the transaction has already succeeded. This will cause an
        // error that the transaction has already ended, and will loop for forever. The `if let Err()`
        // is already capturing errors, so this has to be an Ok
        return result;
    }
}

pub async fn get_chat_message_by_atomic_id(pool: &Database, channel_id: &str, atomic_id: i64) -> Result<ChatMessage, DbError> {
    let collection: Collection<ChatMessage> = pool.collection("chat_messages");
    let doc = doc! {"channel_id": channel_id, "atomic_id": atomic_id};
    match collection.find_one(doc).await {
        Ok(maybe_record) => match maybe_record {
            Some(record) => Ok(record),
            None => Err(DbError::NotFound(ResourceKind::ChatMessage, atomic_id.to_string())),
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
