use mongodb::bson::Bson;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::message::{ChatMessage, CreateMessageSchema};

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestMessageSchema {
    // TODO: Replace this stuff with a mongodb cursor?
    message_count: i64,
    starting_message: String,
}

// TODO: Define error codes, maybe just make an enum that serializes to ints?
#[derive(Clone, Serialize, Deserialize)]
pub struct WebsocketAck {
    pub status_code: i64,
    pub msg: String,
}

impl WebsocketAck {
    pub fn new() -> Self {
        Self { status_code: 0, msg: String::new() }
    }
    pub fn is_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }
}

/*
#[serde(tag = "type", content = "data")]
The above will serialize the enum to look like
{
 "type": "SendChatMessage",
 "data":{
  "chat_message_field": "value"
 }
}
 */
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebsocketMessage {
    ReceiveChatMessage(CreateMessageSchema),
    ReceiveChatUpdateRequest(RequestMessageSchema),
    SendChatMessage(ChatMessage),
    SendAck(WebsocketAck),
}

// TODO: React to message packet (receive and send)
// TODO: Pin message packet (receive and send)
