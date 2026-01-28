use mongodb::bson::{doc, Bson, Document};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    db::{str_to_object_id, MongoModel, PatDatabase},
    error_handler::DbError,
    logger::log_msg,
    models::{
        chat::{
            chat_channel::{ChatChannel, ReturnChannel},
            validation::CreateChannelSchema,
        },
        user::{user_db::db_get_user_by_id, ReturnUser},
    },
};

pub async fn insert_chat_channel(db_handle: &PatDatabase, data: &CreateChannelSchema, user_id: String) -> Result<ChatChannel, DbError> {
    let date_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let doc = doc! {
        "slug": data.slug.clone(),
        "channel_type": data.channel_type,
        "name": data.name.clone(),
        "pinned_messages": [],
        "subscribers": [
            user_id.clone(),
        ],
        "owner_id": user_id.clone(),
        "created_at": date_time,
        "most_recent_message_id": 0,
    };
    db_handle.insert_and_retrieve_one(ChatChannel::collection_name(), doc).await
}

// pub async fn get_chat_channel_by_slug_and_user_id(db_handle: &PatDatabase, slug: &str, user_id: &str) -> Result<ChatChannel, DbError> {
//     let doc = doc! { "slug": slug, "owner_id": user_id };
//     db_handle.find_one(doc).await
// }

pub async fn get_chat_channel_by_id(db_handle: &PatDatabase, id: &str) -> Result<ChatChannel, DbError> {
    let channel_id = str_to_object_id(id)?;
    let filter_doc = doc! {"_id": Bson::ObjectId(channel_id)};
    db_handle.find_one(filter_doc).await
}

pub async fn update_chat_channel_by_id(db_handle: &PatDatabase, filter_doc: Document, update_doc: Document) -> Result<ChatChannel, DbError> {
    db_handle.find_and_update_one(filter_doc, update_doc).await
}

pub async fn list_chat_channels(db_handle: &PatDatabase, filter_doc: Document) -> Result<Vec<ChatChannel>, DbError> {
    db_handle.find(filter_doc).await
}

// Might want to re-name this function as it converts one type to another and might change
// more fields in the future than just subscribers (like the owner)
pub async fn hydrate_chat_channel_subscribers(db_handle: &PatDatabase, chat_channel: ChatChannel) -> ReturnChannel {
    let mut users: Vec<ReturnUser> = Vec::new();
    for user_id in &chat_channel.subscribers {
        match db_get_user_by_id(db_handle, user_id.as_str()).await {
            Ok(user) => users.push(user.into()),
            Err(_e) => log_msg("Failed to get a user while hydrating chat channel subscribers"),
        }
    }
    let mut return_channel = ReturnChannel::from(chat_channel);
    return_channel.subscribers = users;
    return_channel
}
