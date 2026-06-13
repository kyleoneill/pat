use super::message::ChatMessage;
use super::validation::CreateMessageSchema;
use crate::{
    db::{MongoModel, PatDatabase, str_to_object_id},
    error_handler::DbError,
    models::chat::chat_channel::ChatChannel,
};
use futures::TryStreamExt;
use mongodb::{
    Collection,
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

pub async fn get_chat_message_span(
    db_handle: &PatDatabase,
    atomic_id: i64,
    channel_id: &str,
    message_count: i64,
) -> Result<Vec<ChatMessage>, DbError> {
    let collection: Collection<ChatMessage> = db_handle.get_collection();
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
