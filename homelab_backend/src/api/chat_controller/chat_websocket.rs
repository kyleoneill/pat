use crate::app::AppState;
use crate::logger;
use crate::models::chat::chat_channel_db::get_chat_channel_by_id;
use crate::models::chat::message_db::{get_chat_message_span, insert_chat_message};
use crate::models::chat::packet::{MessageCreatedResponse, WebSocketRequest, WebSocketResponse};
use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;

pub(super) async fn handle_socket(socket: WebSocket, _who: SocketAddr, user_id: String, app_state: Arc<AppState>) {
    // Create a channel to send messages
    let (tx, rx) = mpsc::unbounded_channel::<WebSocketResponse>();

    // Register the connection. Do this in a block so the RwLock lock is dropped asap
    {
        let mut connections = app_state.active_connections.write().await;

        // If the user already has a connection, remove the old one
        let _res = connections.remove(user_id.as_str());

        connections.insert(user_id.clone(), tx);
    }

    let (sender, receiver) = socket.split();

    // Spawn a task to receive messages from the socket
    let read_task = tokio::spawn(read_messages(receiver, user_id.clone(), app_state.clone()));

    // Spawn a task to send messages to the socket
    let write_task = tokio::spawn(write_messages(rx, sender));

    /*
    Educational note for me as to what is actually happening right here:
    When `handle_socked` is called, a task is spawned to run the function (since it is async).
    This task then spawns two other tasks, the read/write tasks. The below `tokio::select!`
    macro blocks the task running this function until either the read or write task is resolved.
    When one is resolved the other is canceled and execution of the task running this function
    continues, hitting the cleanup and then ending
    */

    // Wait until either task finishes, cancel the remaining one
    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    // Cleanup, in case the write_task is closed before the read_task
    app_state.active_connections.write().await.remove(user_id.as_str());
}

async fn read_messages(mut receiver: SplitStream<WebSocket>, user_id: String, app_state: Arc<AppState>) {
    // Axum and the client will handle transmitting a heartbeat, so this will always return a Some
    // until the connection is closed. When receiver.next() gets a None, the while loop is terminated
    while let Some(Ok(msg)) = receiver.next().await {
        // Can make this connection more resilient by doing `while let Some(maybe_errd_msg) = receiver.next().await
        // and then matching on maybe_errd_msg to handle the non-fatal-error myself, but that might not matter for
        // a simple chat app. As is currently written, a non-fatal-error will cause the `while let` to fail,
        // ending the while loop and then the async block

        // Should I match here instead of using an if_let to handle binary data or a Close? Need
        // to see what happens if I include Ping/Pong messages here on a match, as axum as-is
        // will automatically handle them for me
        if let Message::Text(text) = msg {
            let response_to_client: WebSocketResponse = match serde_json::from_str::<WebSocketRequest>(text.as_str()) {
                // We got a request to create a message
                Ok(WebSocketRequest::CreateMessage(msg_to_create)) => {
                    // Check if the message is going to a valid channel and that the user is a subscriber of it
                    match get_chat_channel_by_id(&app_state.db, msg_to_create.channel_id.as_str()).await {
                        Ok(channel) => {
                            if channel.subscribers.contains(&user_id) {
                                // Create a db entry for this message
                                match insert_chat_message(&app_state.db, msg_to_create, user_id.as_str()).await {
                                    Ok(chat_message) => {
                                        // Check to see if any subscribers of the destination channel have active connections
                                        for subscriber in channel.subscribers {
                                            if let Some(tx) = app_state.active_connections.read().await.get(subscriber.as_str()) {
                                                // Send a copy of this message to every connected client who is meant to receive it
                                                let _ = tx.send(WebSocketResponse::SendChatMessage(chat_message.clone()));
                                            }
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
                                WebSocketResponse::bad_request("You are not in this chat channel")
                            }
                        }
                        Err(e) => e.into(),
                    }
                }

                // We got a request for the current chat state
                Ok(WebSocketRequest::GetChatState(msg_request)) => {
                    // This should be done in a validation step instead of being checked like this
                    if msg_request.message_count > 50 {
                        WebSocketResponse::bad_request("Can only request a maximum of 50 messages at a time")
                    } else {
                        match get_chat_channel_by_id(&app_state.db, msg_request.channel_id.as_str()).await {
                            // TODO: Getting the channel and checking if the user is in it is being repeated, this should be
                            //       abstracted better
                            Ok(channel) => {
                                if channel.subscribers.contains(&user_id) {
                                    match get_chat_message_span(
                                        &app_state.db,
                                        msg_request.atomic_message_id,
                                        msg_request.channel_id.as_str(),
                                        msg_request.message_count,
                                    )
                                    .await
                                    {
                                        Ok(messages) => WebSocketResponse::SendChatState(messages),
                                        Err(e) => e.into(),
                                    }
                                } else {
                                    WebSocketResponse::bad_request("You are not in this chat channel")
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
                    logger::log_msg("Error while deserializing a websocket packet from a string");
                    WebSocketResponse::bad_request("Failed to decode received data")
                }
            };

            // Respond to the client who sent this request
            let connections = app_state.active_connections.read().await;
            match connections.get(user_id.as_str()) {
                Some(tx) => {
                    let _ = tx.send(response_to_client);
                }
                None => {
                    // Currently active connection is gone, log this?
                }
            };
        }
    }

    // Clean up on disconnect
    app_state.active_connections.write().await.remove(user_id.as_str());
}

async fn write_messages(mut rx: mpsc::UnboundedReceiver<WebSocketResponse>, mut sender: SplitSink<WebSocket, Message>) {
    while let Some(msg) = rx.recv().await {
        if let Ok(text) = serde_json::to_string(&msg) {
            if sender.send(Message::Text(text)).await.is_err() {
                // Is there anything else to do here for error handling?
                logger::log_msg("Error while sending a WebsocketMessage to a sender");
            }
        }
    }
}
