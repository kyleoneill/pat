use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::message::ChatMessage;
use super::validation::CreateMessageSchema;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RequestMessagesSchema {
    pub message_count: i64,
    pub atomic_message_id: i64,
    pub channel_id: String,
}

// TODO: Define error codes, maybe just make an enum that serializes to ints?
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
pub struct MessageCreatedResponse {
    pub atomic_message_id: i64,
    pub chat_channel_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketResponse {
    MessageCreated(MessageCreatedResponse),
    SendChatMessage(ChatMessage),
    SendChatState(Vec<ChatMessage>),
    SendError(WebSocketError),
}

impl WebSocketResponse {
    pub fn bad_request(msg: impl Display) -> Self {
        Self::SendError(WebSocketError {
            status_code: 400,
            msg: msg.to_string(),
        })
    }

    pub fn unauthorized(msg: impl Display) -> Self {
        Self::SendError(WebSocketError {
            status_code: 401,
            msg: msg.to_string(),
        })
    }

    pub fn forbidden(msg: impl Display) -> Self {
        Self::SendError(WebSocketError {
            status_code: 403,
            msg: msg.to_string(),
        })
    }

    pub fn not_found(msg: impl Display) -> Self {
        Self::SendError(WebSocketError {
            status_code: 404,
            msg: msg.to_string(),
        })
    }

    pub fn internal_error(msg: impl Display) -> Self {
        Self::SendError(WebSocketError {
            status_code: 500,
            msg: msg.to_string(),
        })
    }
}

// TODO: React to message packet (receive and send)
// TODO: Pin message packet (receive and send)
// TODO: Edit message
