use serde::{Deserialize, Serialize};

use super::message::{ChatMessage, CreateMessageSchema};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RequestMessagesSchema {
    pub message_count: i64,
    pub atomic_message_id: i64,
    pub channel_id: String,
}

// TODO: Define error codes, maybe just make an enum that serializes to ints?
// TODO: I should have a from/into to convert DbError into a WebSocketError
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WebSocketError {
    pub status_code: i64,
    pub msg: String,
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

impl From<CreateMessageSchema> for WebSocketRequest {
    fn from(value: CreateMessageSchema) -> Self {
        WebSocketRequest::CreateMessage(value)
    }
}

impl From<RequestMessagesSchema> for WebSocketRequest {
    fn from(value: RequestMessagesSchema) -> Self {
        WebSocketRequest::GetChatState(value)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketResponse {
    SendChatMessage(ChatMessage),
    SendChatState(Vec<ChatMessage>),
    SendError(WebSocketError),
}

impl WebSocketResponse {
    pub fn ws_error(status_code: i64, msg: &str) -> WebSocketResponse {
        WebSocketResponse::SendError(WebSocketError {
            status_code,
            msg: msg.to_owned(),
        })
    }
}

// TODO: React to message packet (receive and send)
// TODO: Pin message packet (receive and send)
// TODO: Edit message
