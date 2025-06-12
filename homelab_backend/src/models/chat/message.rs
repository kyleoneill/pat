use super::super::deserialize_id;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct EmojiDetails {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Reactions {
    count: i64,
    emoji: EmojiDetails,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateMessageSchema {
    pub channel_id: String,
    pub contents: String,
    pub reply_to: Option<String>,
}

// TODO: How to handle attachments? What if the user wants to upload an image/gif with a message?
//       A message should be able to have text content _and_ an attachment (so the attachment should not be in `contents`)
//       and be binary data.
#[derive(Serialize, Deserialize, Clone)]
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
}
