use super::message::ChatMessage;
use super::validation::CreateMessageSchema;
use crate::{
    db::{str_to_object_id, MongoModel, PatDatabase},
    error_handler::DbError,
    models::chat::chat_channel::ChatChannel,
};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    error::{Error as MongoError, TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT},
    options::{ReadConcern, WriteConcern},
    ClientSession, Collection,
};

impl MongoModel for ChatMessage {
    fn collection_name() -> &'static str {
        "chat_messages"
    }
    fn model_name() -> &'static str {
        "Chat Message"
    }
    fn mongo_id(&self) -> Result<ObjectId, DbError> {
        match self.id.parse::<ObjectId>() {
            Ok(res) => Ok(res),
            Err(_) => Err(DbError::BadId),
        }
    }
}

pub async fn insert_chat_message(db_handle: &PatDatabase, data: CreateMessageSchema, user_id: &str) -> Result<ChatMessage, DbError> {
    let chat_message_collection: Collection<Document> = db_handle.get_type_agnostic_collection(ChatMessage::collection_name());
    let channel_collection: Collection<ChatChannel> = db_handle.get_collection();

    // TODO: SESSION HANDLING SHOULD BE DONE IN A STANDARD WAY AND NOT IN THIS METHOD
    // Begin a session to create a chat message and update the relevant chat channel
    let mut session = db_handle.pool_ref().client().start_session().await?;
    // Read/write concern taken from the docs, unsure how relevant they are here
    session
        .start_transaction()
        .read_concern(ReadConcern::majority())
        .write_concern(WriteConcern::majority())
        .await?;

    let new_message_id = execute_chat_message_transaction(&chat_message_collection, &channel_collection, &mut session, data.clone(), user_id).await?;
    let filter_doc = doc! {"_id": new_message_id};
    db_handle.find_one(filter_doc).await
}

// TODO: Session pattern should be more generic so I don't need to do this every time I use a session in the future
async fn execute_chat_message_transaction(
    chat_message_collection: &Collection<Document>,
    channel_collection: &Collection<ChatChannel>,
    session: &mut ClientSession,
    data: CreateMessageSchema,
    user_id: &str,
) -> Result<ObjectId, MongoError> {
    // Get the channel associated with this message
    let channel_id = str_to_object_id(data.channel_id.as_str())?;
    let filter_doc = doc! {"_id": Bson::ObjectId(channel_id)};
    let channel = match channel_collection.find_one(filter_doc).session(&mut *session).await? {
        Some(channel) => channel,
        None => return Err(MongoError::custom("Failed to find a chat channel with the given ID".to_string())),
    };

    // Create a message
    let new_atomic_id = channel.most_recent_message_id + 1;
    let insert_message_doc = data.create_message_doc(user_id, new_atomic_id);

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
    let mut loop_counter = 0;
    loop {
        // Emergency safety valve to stop an infinite hang if mongo behaves strangely
        loop_counter += 1;
        if loop_counter > 500 {
            return Err(MongoError::custom("Exceeded retries".to_string()));
        }
        let result = session.commit_transaction().await;
        if let Err(error) = result {
            if error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) || error.contains_label(TRANSIENT_TRANSACTION_ERROR) {
                continue;
            }
            return Err(error);
        }
        // TODO: Mongo docs here has `result?` which will go back to the start of the loop and try
        // the transaction again, but the transaction has already succeeded. This will cause an
        // error that the transaction has already ended, and will loop for forever. The `if let Err()`
        // is already capturing errors, so this has to be an Ok
        return Ok(message_id);
    }
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
