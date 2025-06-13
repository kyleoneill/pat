use super::super::deserialize_id;
use crate::util::current_unix_time;
use mongodb::bson::{doc, Bson, Document};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateMessageSchema {
    pub channel_id: String,
    pub contents: String,
    pub reply_to: Option<String>,
}

impl CreateMessageSchema {
    pub fn create_message_doc(self, user_id: &str) -> Document {
        let current_time = current_unix_time();
        let mut doc = doc! {
            "channel_id": self.channel_id,
            "author_id": user_id.to_owned(),
            "created_at": current_time,
            "updated_at": current_time,
            "contents": self.contents,
            "reactions": Vec::<Reactions>::new(),
            "pinned": false,
        };
        if self.reply_to.is_some() {
            doc.insert(
                "reply_to",
                self.reply_to
                    .expect("Option should always have a value when is_some() is true"),
            );
        }
        doc
    }
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
