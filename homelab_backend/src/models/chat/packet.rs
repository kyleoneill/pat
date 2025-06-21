use serde::{Deserialize, Serialize};

use super::message::{ChatMessage, CreateMessageSchema};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RequestMessagesSchema {
    // TODO: Replace this stuff with a mongodb cursor?
    message_count: i64,
    starting_message: String,
    channel_id: String,
}

// TODO: Define error codes, maybe just make an enum that serializes to ints?
#[derive(Clone, Serialize, Deserialize)]
pub struct WebsocketAck {
    pub status_code: i64,
    pub msg: String,
}

impl WebsocketAck {
    pub fn new() -> Self {
        Self {
            status_code: 0,
            msg: String::new(),
        }
    }
    pub fn is_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }
}

/*
#[serde(tag = "type", content = "data")]
The above will serialize the enum to look like
{
 "type": "CreateMessage",
 "data":{
  "chat_message_field": "value"
 }
}
 */
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketRequest {
    CreateMessage(CreateMessageSchema),
    GetChatState(RequestMessagesSchema),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketResponse {
    SendChatMessage(ChatMessage),
    SendAck(WebsocketAck),
}

// TODO: React to message packet (receive and send)
// TODO: Pin message packet (receive and send)
// TODO: ReceiveChatUpdateRequest needs a Send which sends a bundle of messages
// TODO: Edit message
