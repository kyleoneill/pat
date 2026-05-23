use crate::app::AppState;
use crate::logger;
use crate::models::chat::chat_channel_db::get_chat_channel_by_id;
use crate::models::chat::message_db::{get_chat_message_span, insert_chat_message};
use crate::models::chat::packet::{MessageCreatedResponse, WebSocketRequest, WebSocketResponse};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;

pub(super) async fn handle_socket(socket: WebSocket, _who: SocketAddr, user_id: String, app_state: Arc<AppState>) {
    // TODO: This function is way too large and should be broken up

    // Create a channel to send messages
    let (tx, mut rx) = mpsc::unbounded_channel::<WebSocketResponse>();

    // Register the connection
    {
        let mut connections = app_state.active_connections.write().await;

        // Check if this user already has a connection
        if connections.contains_key(user_id.as_str()) {
            connections
                .remove(user_id.as_str())
                .expect("Hashmap should contain a value when .contains_key was true");
        }

        connections.insert(user_id.clone(), tx);
    }

    let (mut sender, mut receiver) = socket.split();

    // Spawn a task to receive messages from the socket
    let user_id_read_task = user_id.clone();
    let cloned_state = app_state.clone();
    let read_task = tokio::spawn(async move {
        // Axum and the client will handle transmitting a heartbeat, so this will always return a Some
        // until the connection is closed. When receiver.next() gets a None, the while loop is terminated
        while let Some(Ok(msg)) = receiver.next().await {
            // Can make this connection more resilient by doing `while let Some(maybe_errd_msg) = receiver.next().await
            // and then matching on maybe_errd_msg to handle the non-fatal-error myself, but that might not matter for
            // a simple chat app. As is currently written, a non-fatal-error will cause the `while let` to fail,
            // ending the while loop and then the async block

            // TODO: Should I match here instead of using an if_let to handle binary data or a Close?
            if let Message::Text(text) = msg {
                match serde_json::from_str::<WebSocketRequest>(text.as_str()) {
                    Ok(WebSocketRequest::CreateMessage(msg_to_create)) => {
                        // Can this be made more readable

                        // Verify that the message is being sent to a valid channel that the sender is in
                        let channel_id = msg_to_create.channel_id.as_str();
                        let maybe_response: Option<WebSocketResponse> = match get_chat_channel_by_id(&cloned_state.db, channel_id).await {
                            Ok(channel) => {
                                // Verify that the current user is a subscriber of the channel
                                if channel.subscribers.contains(&user_id_read_task) {
                                    // Create a db entry for this message
                                    match insert_chat_message(&cloned_state.db, msg_to_create, user_id_read_task.as_str()).await {
                                        Ok(chat_message) => {
                                            // Check to see if any subscribers of the destination channel have active connections
                                            for subscriber in channel.subscribers {
                                                if let Some(tx) = cloned_state.active_connections.read().await.get(subscriber.as_str()) {
                                                    let _ = tx.send(WebSocketResponse::SendChatMessage(chat_message.clone()));
                                                }
                                            }
                                            let response = MessageCreatedResponse {
                                                atomic_message_id: chat_message.atomic_id,
                                                chat_channel_id: chat_message.channel_id,
                                            };
                                            Some(WebSocketResponse::MessageCreated(response))
                                        }
                                        Err(_e) => Some(WebSocketResponse::ws_error(500, "Unhandled failure while creating a chat message")),
                                    }
                                } else {
                                    Some(WebSocketResponse::ws_error(400, "You are not in this chat channel"))
                                }
                            }
                            Err(_) => {
                                // TODO: Actual error handling, status code should be an enum
                                Some(WebSocketResponse::ws_error(404, "Chat channel does not exist"))
                            }
                        };

                        if let Some(response) = maybe_response {
                            let connections = cloned_state.active_connections.read().await;
                            match connections.get(user_id_read_task.as_str()) {
                                Some(tx) => {
                                    let _ = tx.send(response);
                                }
                                None => {
                                    // Currently active connection is gone, log this?
                                }
                            };
                        }
                    }
                    Ok(WebSocketRequest::GetChatState(msg_request)) => {
                        let get_chat_state_res: WebSocketResponse = {
                            // This should be done in a validation step instead of being checked like this
                            if msg_request.message_count > 50 {
                                WebSocketResponse::ws_error(400, "Can only request a maximum of 50 messages at a time")
                            } else {
                                match get_chat_channel_by_id(&cloned_state.db, msg_request.channel_id.as_str()).await {
                                    // TODO: Getting the channel and checking if the user is in it is being repeated, this should be
                                    //       abstracted better
                                    Ok(channel) => {
                                        if channel.subscribers.contains(&user_id_read_task) {
                                            match get_chat_message_span(
                                                &cloned_state.db,
                                                msg_request.atomic_message_id,
                                                msg_request.channel_id.as_str(),
                                                msg_request.message_count,
                                            )
                                            .await
                                            {
                                                Ok(messages) => WebSocketResponse::SendChatState(messages),
                                                Err(_e) => WebSocketResponse::ws_error(500, "Unhandled error while reading chat messages"),
                                            }
                                        } else {
                                            WebSocketResponse::ws_error(400, "You are not in this chat channel")
                                        }
                                    }
                                    Err(_e) => WebSocketResponse::ws_error(404, "Chat channel does not exist"),
                                }
                            }
                        };
                        let connections = cloned_state.active_connections.read().await;
                        match connections.get(user_id_read_task.as_str()) {
                            Some(tx) => {
                                let _ = tx.send(get_chat_state_res);
                            }
                            None => {
                                // Currently active connection is gone, log this?
                            }
                        };
                    }
                    Err(_e) => {
                        // Would be nice to give more info to the user here about what failed
                        let websocket_error = WebSocketResponse::ws_error(400, "Failed to decode received data");
                        let connections = cloned_state.active_connections.read().await;
                        match connections.get(user_id_read_task.as_str()) {
                            Some(tx) => {
                                let _ = tx.send(websocket_error);
                            }
                            None => {
                                // Currently active connection is gone, log this?
                            }
                        };
                        logger::log_msg("Error while deserializing a websocket packet from a string");
                    }
                }
            }
        }

        // Clean up on disconnect
        cloned_state.active_connections.write().await.remove(user_id_read_task.as_str());
    });

    // Spawn a task to send messages to the socket
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(text)).await.is_err() {
                    // TODO: HANDLE ERROR HERE
                    logger::log_msg("Error while sending a WebsocketMessage to a sender");
                }
            }
        }
    });

    // Educational note for me as to what is actually happening right here:
    // When `handle_socked` is called, a task is spawned to run the function (since it is async).
    // This task then spawns two other tasks, the read/write tasks. The below `tokio::select!`
    // macro blocks the task running this function until either the read or write task is resolved.
    // When one is resolved the other is cancelled and execution of the task running this function
    // continues, hitting the cleanup and then ending

    // Wait until either task finishes, cancel the remaining one
    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    // Cleanup, in case the write_task is closed before the read_task
    app_state.active_connections.write().await.remove(user_id.as_str());
}
