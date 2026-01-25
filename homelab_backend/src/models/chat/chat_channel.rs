use mongodb::bson::{doc, Bson};

use super::super::deserialize_id;
use crate::db::MongoModel;
use crate::models::user::ReturnUser;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, PartialEq, Clone, Debug)]
pub enum ChannelType {
    DirectMessage,
    Group,
    Server, // Stub type to support a more featured group chat eventually
}

fn deserialize_channel_type<'de, D>(deserializer: D) -> Result<ChannelType, D::Error>
where
    D: Deserializer<'de>,
{
    let bson = Bson::deserialize(deserializer)?;
    if let Bson::Int64(value) = bson {
        Ok(ChannelType::from(value))
    } else if let Bson::String(value) = bson {
        // Why is this sometimes being deserialized as a string here?
        Ok(ChannelType::from(value))
    } else {
        Err(serde::de::Error::custom("Expected an Int64 while deserializing ChannelType"))
    }
}

impl From<String> for ChannelType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "DirectMessage" => ChannelType::DirectMessage,
            "Group" => ChannelType::Group,
            "Server" => ChannelType::Server,
            _ => panic!("Tried to convert an invalid string to a ChannelType"),
        }
    }
}

impl From<ChannelType> for Bson {
    fn from(value: ChannelType) -> Self {
        match value {
            ChannelType::DirectMessage => Bson::Int64(0),
            ChannelType::Group => Bson::Int64(1),
            ChannelType::Server => Bson::Int64(2),
        }
    }
}

impl From<Bson> for ChannelType {
    fn from(value: Bson) -> Self {
        match value {
            Bson::Int64(int) => ChannelType::from(int),
            _ => panic!("Tried to deserialize a ChannelType that was stored as a non-int"),
        }
    }
}

impl From<i64> for ChannelType {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::DirectMessage,
            1 => Self::Group,
            2 => Self::Server,
            _ => panic!("Unsupported value when converting an i64 to a ChannelType"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChatChannel {
    #[serde(rename = "_id", deserialize_with = "deserialize_id")]
    pub id: String,
    pub slug: String,
    #[serde(deserialize_with = "deserialize_channel_type")]
    pub channel_type: ChannelType,
    pub name: Option<String>,
    pub pinned_messages: Vec<String>, // Vec of message IDs
    pub subscribers: Vec<String>,     // Vec of user IDs
    pub owner_id: String,
    pub created_at: i64,
    pub most_recent_message_id: i64,
}

impl MongoModel for ChatChannel {
    fn collection_name() -> &'static str {
        "chat_channels"
    }
    fn model_name() -> &'static str {
        "Chat Channel"
    }
}

#[derive(Serialize, Deserialize)]
pub struct ReturnChannel {
    pub _id: String,
    pub slug: String,
    #[serde(deserialize_with = "deserialize_channel_type")]
    pub channel_type: ChannelType,
    pub name: Option<String>,
    // TODO: Replace pinned_messages with a Vec<ChatMessage> when returning maybe?
    pub pinned_messages: Vec<String>, // Vec of message IDs
    pub subscribers: Vec<ReturnUser>,
    // TODO: Replace owner_id with a ReturnUser when returning
    pub owner_id: String,
    pub created_at: i64,
    pub most_recent_message_id: i64,
}

impl From<ChatChannel> for ReturnChannel {
    fn from(value: ChatChannel) -> Self {
        Self {
            _id: value.id,
            slug: value.slug,
            channel_type: value.channel_type,
            name: value.name,
            pinned_messages: value.pinned_messages,
            subscribers: Vec::new(),
            owner_id: value.owner_id,
            created_at: value.created_at,
            most_recent_message_id: value.most_recent_message_id,
        }
    }
}
