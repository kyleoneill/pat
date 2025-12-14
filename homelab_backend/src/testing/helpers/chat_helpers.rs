use crate::models::chat::{
    chat_channel::ReturnChannel,
    message::ChatMessage,
    packet::{WebSocketError, WebSocketRequest, WebSocketResponse},
    validation::CreateChannelSchema,
};
use crate::testing::helpers::{get_request, post_request, put_request};
use axum::body::Body;
use axum::http::StatusCode;
use futures::{SinkExt, StreamExt};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde_json::json;
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpStream, time::timeout};
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};

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
    let path = format!("/chat/channels{query_params}");
    get_request(client, path.as_str(), token, addr).await
}

pub async fn get_channel_by_id(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    channel_id: &str,
) -> Result<ReturnChannel, (StatusCode, String)> {
    let path = format!("/chat/channels/{channel_id}");
    get_request(client, path.as_str(), token, addr).await
}

pub async fn receive_chat_message(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<ChatMessage, WebSocketError> {
    match guarded_receive_data_from_socket(socket).await {
        WebSocketResponse::SendChatMessage(chat_message) => Ok(chat_message),
        WebSocketResponse::SendError(ws_err) => Err(ws_err),
        WebSocketResponse::MessageCreated(_message_created) => {
            // When a user creates a message they are sent a MessageCreated, but it is not guaranteed which order
            // the two responses will come in. Try to poll again
            match guarded_receive_data_from_socket(socket).await {
                WebSocketResponse::SendChatMessage(chat_message) => Ok(chat_message),
                WebSocketResponse::SendError(ws_err) => Err(ws_err),
                _ => panic!("Should only SendChatMessage or SendError when getting a chat message"),
            }
        }
        _ => panic!("Should only receive SendChatMessage or SendError when getting a chat message"),
    }
}

pub async fn receive_chat_state(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<Vec<ChatMessage>, WebSocketError> {
    match guarded_receive_data_from_socket(socket).await {
        WebSocketResponse::SendChatState(chat_messages) => Ok(chat_messages),
        WebSocketResponse::SendError(ws_err) => Err(ws_err),
        _ => panic!("Should only receive SendChatState or SendError when getting chat state"),
    }
}

// Wrap the function which actually gets the message in a timeout so we panic if there is no data
// in the socket, rather than hang endlessly
async fn guarded_receive_data_from_socket(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> WebSocketResponse {
    match timeout(Duration::from_secs(10), receive_data_from_socket(socket)).await {
        Ok(ws_response_data) => ws_response_data,
        Err(_) => panic!("Failed to receive a chat message on a websocket client"),
    }
}

async fn receive_data_from_socket(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> WebSocketResponse {
    match socket.next().await {
        Some(server_res) => match server_res {
            Ok(server_message) => match server_message {
                tungstenite::Message::Text(msg) => serde_json::from_str::<WebSocketResponse>(msg.as_str())
                    .expect("Failed to deserialize chat creation response into a WebSocketResponse"),
                _ => panic!("Server should respond with a Message::Text variant when receiving data"),
            },
            Err(e) => panic!("Server responded with error: {e}"),
        },
        None => panic!("Failed to get a response from the server when one was expected"),
    }
}

// Wrap the function which actually sends the message in a timeout so we panic if the async
// operation fails
pub async fn send_websocket_request(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, data: &WebSocketRequest) {
    match timeout(Duration::from_secs(10), send_data_over_socket(socket, data)).await {
        Ok(()) => (),
        Err(_) => panic!("Failed to send data on a websocket client"),
    }
}

async fn send_data_over_socket(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, data: &WebSocketRequest) {
    let serialized = serde_json::to_string(data).expect("Failed to serialize WebSocketRequest");
    socket
        .send(tungstenite::Message::Text(serialized))
        .await
        .expect("Failed to send data over a websocket");
}

pub async fn send_arbitrary_data(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, data: String) {
    match timeout(Duration::from_secs(10), socket.send(tungstenite::Message::Text(data))).await {
        Ok(_) => (),
        Err(_) => panic!("Failed to send arbitrary data over the websocket"),
    }
}
