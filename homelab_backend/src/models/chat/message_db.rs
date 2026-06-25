use super::message::ChatMessage;
use super::validation::{CreateMessageSchema, EditMessageSchema};
use crate::{
    db::{MongoModel, PatDatabase, str_to_object_id},
    error_handler::DbError,
    models::chat::chat_channel::ChatChannel,
};
use futures::StreamExt;
use mongodb::{
    Collection, bson,
    bson::{Bson, Document, doc},
    error::Error as MongoError,
};

impl MongoModel for ChatMessage {
    fn collection_name() -> &'static str {
        "chat_messages"
    }
    fn model_name() -> &'static str {
        "Chat Message"
    }
    fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

pub async fn insert_chat_message(db_handle: &PatDatabase, data: CreateMessageSchema, user_id: String) -> Result<ChatMessage, DbError> {
    let chat_message_collection: Collection<Document> = db_handle.get_type_agnostic_collection(ChatMessage::collection_name());
    let channel_collection: Collection<ChatChannel> = db_handle.get_collection();

    let mut session = db_handle.pool_ref().client().start_session().await?;
    let new_message_id = session
        .start_transaction()
        .and_run2(async move |session| {
            // Mongo docs say that the closure cannot consume owned values due to being called repeatedly, even if the callback owns the values
            let cloned_data = data.clone();

            let channel_id = str_to_object_id(cloned_data.channel_id.as_str())?;
            let filter_doc = doc! {"_id": Bson::ObjectId(channel_id)};
            let channel = match channel_collection.find_one(filter_doc).session(&mut *session).await? {
                Some(channel) => channel,
                None => return Err(MongoError::custom("Failed to find a chat channel with the given ID".to_string())),
            };

            // Create a message
            let new_atomic_id = channel.most_recent_message_id + 1;
            let insert_message_doc = cloned_data.create_message_doc(user_id.as_str(), new_atomic_id);

            // When a message is created, also increment the most recent message ID on the channel
            let channel_filter_doc = doc! { "_id": Bson::ObjectId(channel_id) };
            let channel_update = doc! { "$set": { "most_recent_message_id": new_atomic_id }};

            let insert_result = chat_message_collection.insert_one(insert_message_doc).session(&mut *session).await?;
            let message_id = insert_result
                .inserted_id
                .as_object_id()
                .ok_or(MongoError::custom("Failed to parse an insertion ID to ObjectID".to_string()))?;
            channel_collection
                .update_one(channel_filter_doc, channel_update)
                .session(&mut *session)
                .await?;
            Ok(message_id)
        })
        .await?;
    let filter_doc = doc! {"_id": new_message_id};
    db_handle.find_one(filter_doc).await
}

pub async fn get_chat_message(db_handle: &PatDatabase, message_id: &str) -> Result<ChatMessage, DbError> {
    let as_object_id = str_to_object_id(message_id)?;
    let filter_doc = doc! {"_id": Bson::ObjectId(as_object_id)};
    db_handle.find_one(filter_doc).await
}

pub async fn update_chat_message(db_handle: &PatDatabase, message_id: &str, edit_msg_data: &EditMessageSchema) -> Result<ChatMessage, DbError> {
    let as_object_id = str_to_object_id(message_id)?;
    let filter_doc = doc! {"_id": Bson::ObjectId(as_object_id)};
    let mut update_doc = match bson::to_document(edit_msg_data) {
        Ok(res) => res,
        Err(_) => return Err(DbError::UnhandledException("Failed to serialize update request data".to_string())),
    };
    // Remove the message_id from the edit packet. We cannot use serde(skip_serializing) for the field
    // since it is serialized in tests
    update_doc.remove("message_id");
    db_handle.find_and_update_one(filter_doc, update_doc).await
}

/// Get message_count messages starting from the passed in atomic_id going backwards. If no message
/// equal to the atomic_id exists, get message_count starting from the first existing message older
/// than it.
///
/// e.g. if atomic_id 1000000 is passed with message_count=2 in but the most recent is 10, this will return
/// [9, 10]
pub async fn get_chat_message_span(
    db_handle: &PatDatabase,
    atomic_id: i64,
    channel_id: &str,
    message_count: usize,
) -> Result<Vec<ChatMessage>, DbError> {
    let collection: Collection<ChatMessage> = db_handle.get_collection();

    // Get messages from this channel starting from the given atomic id going backwards
    let doc = doc! {
        "channel_id": channel_id,
        "atomic_id": {"$lte": atomic_id},
    };
    let sort = doc! {"atomic_id": -1};

    // Add messages to the vec until we either have the message_count number of them or we exhaust
    // the cursor
    match collection.find(doc).sort(sort).await {
        Ok(mut cursor) => {
            let mut messages = Vec::new();

            while let Some(pulled_message) = cursor.next().await {
                if messages.len() == message_count {
                    break;
                }
                match pulled_message {
                    Ok(msg) => messages.push(msg),
                    Err(e) => return Err(e.into()),
                }
            }
            // Reverse the vec before returning so it ends with the newest message
            messages.reverse();
            Ok(messages)
        }
        Err(e) => Err(e.into()),
    }
}
