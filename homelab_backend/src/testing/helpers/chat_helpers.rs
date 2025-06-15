use crate::testing::helpers::{get_request, post_request, put_request};
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
    post_request(client, "/chat/channels", data, Some(token), addr).await
}

pub async fn subscribe_to_channel(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    channel_id: &str,
) -> Result<ChatChannel, (StatusCode, String)> {
    let data = json!({"channel_id": channel_id});
    put_request(client, "/chat/channels/subscribe", data, token, addr).await
}

pub async fn unsubscribe_from_channel(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    channel_id: &str,
) -> Result<ChatChannel, (StatusCode, String)> {
    let data = json!({"channel_id": channel_id});
    put_request(client, "/chat/channels/unsubscribe", data, token, addr).await
}

pub async fn list_channels(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    query_params: &str,
) -> Result<Vec<ChatChannel>, (StatusCode, String)> {
    let path = format!("/chat/channels{}", query_params);
    get_request(client, path.as_str(), token, addr).await
}
