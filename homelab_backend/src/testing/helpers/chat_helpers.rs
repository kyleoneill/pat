use futures::StreamExt;
use crate::testing::helpers::{get_request, post_request, put_request};
use axum::body::Body;
use axum::http::StatusCode;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};
use crate::models::chat::chat_channel::{CreateChannelSchema, ReturnChannel};
use crate::models::chat::message::ChatMessage;
use crate::models::chat::packet::WebSocketResponse;

pub async fn create_chat_channel(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    chat_channel: &CreateChannelSchema,
) -> Result<ReturnChannel, (StatusCode, String)> {
    let data = json!(chat_channel);
    post_request(client, "/chat/channels", data, Some(token), addr).await
}

pub async fn subscribe_to_channel(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    channel_id: &str,
) -> Result<ReturnChannel, (StatusCode, String)> {
    let data = json!({"channel_id": channel_id});
    put_request(client, "/chat/channels/subscribe", data, token, addr).await
}

pub async fn unsubscribe_from_channel(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    channel_id: &str,
) -> Result<ReturnChannel, (StatusCode, String)> {
    let data = json!({"channel_id": channel_id});
    put_request(client, "/chat/channels/unsubscribe", data, token, addr).await
}

pub async fn list_channels(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    query_params: &str,
) -> Result<Vec<ReturnChannel>, (StatusCode, String)> {
    let path = format!("/chat/channels{}", query_params);
    get_request(client, path.as_str(), token, addr).await
}

pub async fn receive_chat_message(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> ChatMessage {
    match socket.next().await {
        Some(server_res) => match server_res {
            Ok(server_message) => match server_message {
                tungstenite::Message::Text(msg) => {
                    match serde_json::from_str::<WebSocketResponse>(msg.as_str()).expect("Failed to deserialize chat creation response into a WebSocketResponse") {
                        WebSocketResponse::SendChatMessage(chat_message) => chat_message,
                        _ =>  panic!("WebSocketResponse to a chat creation must be a SendChatMessage")
                    }
                },
                _ => panic!("Server should respond with a Message::Text variant when creating a chat message"),
            },
            Err(e) => panic!("Server responded to a chat creation with error: {e}"),
        },
        None => panic!("Failed to get a response from the server after creating a chat message"),
    }
}
