use crate::app::AppState;
use crate::models::chat::chat_channel_db::get_chat_channel_by_id;
use crate::models::chat::message_db::{get_chat_message_span, insert_chat_message};
use crate::models::chat::packet::{MessageCreatedResponse, WebSocketRequest, WebSocketResponse};
use axum::extract::ws::{Message, WebSocket};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{RwLock, mpsc};

const MAX_MSG_COUNT: i64 = 50;

pub(super) async fn handle_socket(mut socket: WebSocket, _who: SocketAddr, user_id: String, app_state: Arc<AppState>) {
    let active_connections = Arc::clone(&app_state.active_connections);

    // Create a channel to send messages
    let (tx, mut rx) = mpsc::unbounded_channel::<WebSocketResponse>();

    // Register the connection. Do this in a block so the RwLock lock is dropped asap
    {
        let mut connections = active_connections.write().await;

        // If the user already has a connection, remove the old one
        let _res = connections.remove(user_id.as_str());

        connections.insert(user_id.clone(), tx);
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                // Axum and the client will handle transmitting a heartbeat, so this will always return a Some
                // until the connection is closed. When socket.recv() gets a None the connection is closed
                match incoming {
                    Some(incoming_data) => match incoming_data {
                        Ok(ok_data) => match ok_data {
                            Message::Text(msg_text) => read_messages(msg_text.to_string(), user_id.clone(), Arc::clone(&app_state), Arc::clone(&active_connections)).await,
                            Message::Close(_) => break,
                            _ => ()
                        },
                        // TODO: Handle the error here? Does this break the connection?
                        Err(_) => break
                    },
                    None => break
                }
            },
            outgoing = rx.recv() => {
                match outgoing {
                    Some(response) => {
                        if let Ok(msg) = serde_json::to_string(&response) {
                            let converted = Message::Text(msg.into());
                            if socket.send(converted).await.is_err() {
                                break
                            }
                        }
                    },
                    None => break,
                }
            },
        }
    }

    // Clean up on disconnect
    active_connections.write().await.remove(user_id.as_str());
}

async fn read_messages(
    msg: String,
    user_id: String,
    app_state: Arc<AppState>,
    active_connections: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<WebSocketResponse>>>>,
) {
    let response_to_client: WebSocketResponse = match serde_json::from_str::<WebSocketRequest>(msg.as_str()) {
        // We got a request to create a message
        Ok(WebSocketRequest::CreateMessage(msg_to_create)) => {
            // Check if the message is going to a valid channel and that the user is a subscriber of it
            let chat_channel = get_chat_channel_by_id(&app_state.db, msg_to_create.channel_id.as_str()).await;
            match chat_channel {
                Ok(channel) => {
                    if channel.subscribers.contains(&user_id) {
                        // Create a db entry for this message
                        let result = insert_chat_message(&app_state.db, msg_to_create, user_id.clone()).await;
                        match result {
                            Ok(chat_message) => {
                                // Check to see if any subscribers of the destination channel have active connections and then send this message to them
                                let connection = active_connections.clone().read_owned().await;
                                let subscribers: Vec<_> = {
                                    channel
                                        .subscribers
                                        .iter()
                                        .filter_map(|subscriber| connection.get(subscriber.as_str()).cloned())
                                        .collect()
                                };
                                for tx in subscribers {
                                    let _ = tx.send(WebSocketResponse::SendChatMessage(chat_message.clone()));
                                }

                                let response = MessageCreatedResponse {
                                    atomic_message_id: chat_message.atomic_id,
                                    chat_channel_id: chat_message.channel_id,
                                };
                                WebSocketResponse::MessageCreated(response)
                            }
                            Err(e) => e.into(),
                        }
                    } else {
                        WebSocketResponse::bad_request("You are not in this chat channel".to_string())
                    }
                }
                Err(e) => e.into(),
            }
        }

        // We got a request for the current chat state
        Ok(WebSocketRequest::GetChatState(msg_request)) => {
            // This should be done in a validation step instead of being checked like this
            if msg_request.message_count > MAX_MSG_COUNT {
                WebSocketResponse::bad_request(format!("Can only request a maximum of {} messages at a time", MAX_MSG_COUNT))
            } else {
                let result = get_chat_channel_by_id(&app_state.db, msg_request.channel_id.as_str()).await;
                match result {
                    // TODO: Getting the channel and checking if the user is in it is being repeated, this should be
                    //       abstracted better
                    Ok(channel) => {
                        if channel.subscribers.contains(&user_id) {
                            let result = get_chat_message_span(
                                &app_state.db,
                                msg_request.atomic_message_id,
                                msg_request.channel_id.as_str(),
                                msg_request.message_count,
                            )
                            .await;
                            match result {
                                Ok(messages) => WebSocketResponse::SendChatState(messages),
                                Err(e) => e.into(),
                            }
                        } else {
                            WebSocketResponse::bad_request("You are not in this chat channel".to_string())
                        }
                    }
                    Err(e) => e.into(),
                }
            }
        }

        // We got an error decoding the text packet using serde json
        Err(_e) => {
            // TODO: ACTUAL ERROR HANDLING HERE WITH e
            // Would be nice to give more info to the user here about what failed
            WebSocketResponse::bad_request("Failed to decode received data".to_string())
        }
    };

    let tx = {
        let connections = active_connections.clone().read_owned().await;
        connections.get(user_id.as_str()).cloned()
    };
    if let Some(tx) = tx {
        let _ = tx.send(response_to_client);
    }
}
