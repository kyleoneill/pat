use mongodb::error::ErrorKind;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::resource_kinds::ResourceKind;
use crate::error_handler::DbError;

use super::chat_channel::{ChatChannel, CreateChannelSchema};

pub async fn insert_chat_channel(
    pool: &Database,
    data: &CreateChannelSchema,
    user_id: String,
) -> Result<ChatChannel, DbError> {
    let collection: Collection<Document> = pool.collection("chat_channels");
    let date_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
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
    };
    match collection.insert_one(doc).await {
        Ok(_res) => (),
        Err(e) => {
            return match *e.kind {
                ErrorKind::Write(_) => Err(DbError::AlreadyExists(
                    ResourceKind::ChatChannel,
                    data.slug.clone(),
                )),
                _ => Err(e.into()),
            }
        }
    }
    get_chat_channel_by_slug_and_user_id(pool, data.slug.as_str(), user_id.as_str()).await
}

pub async fn get_chat_channel_by_slug_and_user_id(
    pool: &Database,
    slug: &str,
    user_id: &str,
) -> Result<ChatChannel, DbError> {
    let collection: Collection<ChatChannel> = pool.collection("chat_channels");
    let doc = doc! { "slug": slug, "owner_id": user_id };
    match collection.find_one(doc).await {
        Ok(maybe_record) => match maybe_record {
            Some(chat_channel) => Ok(chat_channel),
            None => Err(DbError::NotFound(
                ResourceKind::ChatChannel,
                slug.to_owned(),
            )),
        },
        Err(e) => Err(e.into()),
    }
}
