use super::message::Reactions;
use crate::util::current_unix_time;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateChannelSchema {
    pub name: Option<String>,
    pub channel_type: i64,
    pub slug: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateMessageSchema {
    pub channel_id: String,
    pub contents: String,
    pub reply_to: Option<String>,
}

impl CreateMessageSchema {
    pub fn create_message_doc(self, user_id: &str, atomic_id: i64) -> Document {
        let current_time = current_unix_time();
        let mut doc = doc! {
            "channel_id": self.channel_id,
            "author_id": user_id.to_owned(),
            "created_at": current_time,
            "updated_at": current_time,
            "contents": self.contents,
            "reactions": Vec::<Reactions>::new(),
            "pinned": false,
            "atomic_id": atomic_id,
        };
        if let Some(reply_to) = self.reply_to {
            doc.insert("reply_to", reply_to);
        };
        doc
    }
}
