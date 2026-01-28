use super::super::deserialize_id;
use crate::{db::MongoModel, error_handler::DbError};
use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EmojiDetails {
    id: String,
    name: String,
}

impl From<EmojiDetails> for Bson {
    fn from(value: EmojiDetails) -> Self {
        Bson::Document(doc! {
            "id": value.id,
            "name": value.name,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Reactions {
    count: i64,
    emoji: EmojiDetails,
}

impl From<Reactions> for Bson {
    fn from(value: Reactions) -> Self {
        Bson::Document(doc! {
            "count": value.count,
            "emoji": value.emoji,
        })
    }
}

// TODO: How to handle attachments? What if the user wants to upload an image/gif with a message?
//       A message should be able to have text content _and_ an attachment (so the attachment should not be in `contents`)
//       and be binary data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatMessage {
    #[serde(rename = "_id", deserialize_with = "deserialize_id")]
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub contents: String,
    pub reply_to: Option<String>,
    pub reactions: Vec<Reactions>,
    pub pinned: bool,
    pub atomic_id: i64,
}

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
