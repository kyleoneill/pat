use crate::testing::helpers::post_request;
use axum::body::Body;
use axum::http::StatusCode;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde_json::json;
use std::net::SocketAddr;

use crate::models::chat::chat_channel::{ChatChannel, CreateChannelSchema};

pub async fn create_chat_channel(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    chat_channel: &CreateChannelSchema,
) -> Result<ChatChannel, (StatusCode, String)> {
    let data = json!(chat_channel);
    post_request(client, "/chat/channel", data, Some(token), addr).await
}
