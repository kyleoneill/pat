use crate::app::AppState;
use crate::logger;
use crate::models::chat::{
    chat_channel::ChatChannel,
    chat_channel_db::get_chat_channel_by_id,
    message::ChatMessage,
    message_db::{get_chat_message, get_chat_message_span, insert_chat_message, update_chat_message},
    packet::{MessageCreatedResponse, RequestMessagesSchema, WebSocketRequest, WebSocketResponse},
    validation::{CreateMessageSchema, EditMessageSchema},
};
use axum::extract::ws::{Message, WebSocket};
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;

const MAX_MSG_COUNT: usize = 50;

enum UpdateKind {
    NewMessage,
    UpdateMessage,
}

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
        When `handle_socket` is called, a task is spawned to run the function (since it is async).
        This task then spawns two other tasks, the read/write tasks. The below `tokio::select!`
        macro blocks the `handle_socket` task until either the read or write task is resolved.
        When one is resolved the other is canceled and execution of the `handle_socket` task
        continues, hitting the cleanup and then ending
    */

    // Wait until either task finishes, cancel the remaining one
    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    // Clean up on disconnect
    app_state.active_connections.write().await.remove(user_id.as_str());
}

async fn write_messages(mut rx: mpsc::UnboundedReceiver<WebSocketResponse>, mut sender: SplitSink<WebSocket, Message>) {
    while let Some(msg) = rx.recv().await {
        if let Ok(text) = serde_json::to_string(&msg)
            && sender.send(Message::Text(text.into())).await.is_err()
        {
            // Is there anything else to do here for error handling?
            logger::log_msg("Error while sending a WebsocketMessage to a sender");
        }
    }
}

/// Send an update to every subscriber of a channel who has an active connection
async fn update_channel_subscribers(app_state: &AppState, channel: &ChatChannel, update_kind: UpdateKind, message: &ChatMessage) {
    for subscriber in &channel.subscribers {
        if let Some(tx) = app_state.active_connections.read().await.get(subscriber.as_str()) {
            let _ = match update_kind {
                UpdateKind::NewMessage => tx.send(WebSocketResponse::SendChatMessage(message.clone())),
                UpdateKind::UpdateMessage => tx.send(WebSocketResponse::SendUpdatedChatMessage(message.clone())),
            };
        }
    }
}

async fn create_msg(app_state: &AppState, user_id: String, create_schema: CreateMessageSchema) -> WebSocketResponse {
    // Verify the user is in the channel this message would belong to
    let chat_channel = match get_chat_channel_by_id(&app_state.db, create_schema.channel_id.as_str()).await {
        Ok(channel) => channel,
        Err(e) => return e.into(),
    };
    if !chat_channel.subscribers.contains(&user_id) {
        return WebSocketResponse::bad_request("You are not in this chat channel");
    }

    let new_message = match insert_chat_message(&app_state.db, create_schema, user_id).await {
        Ok(new_message) => new_message,
        Err(e) => return e.into(),
    };

    update_channel_subscribers(app_state, &chat_channel, UpdateKind::NewMessage, &new_message).await;

    let response = MessageCreatedResponse {
        atomic_message_id: new_message.atomic_id,
        chat_channel_id: new_message.channel_id,
    };
    WebSocketResponse::MessageCreated(response)
}

async fn get_chat_state(app_state: &AppState, user_id: String, get_state_schema: RequestMessagesSchema) -> WebSocketResponse {
    if get_state_schema.message_count > MAX_MSG_COUNT {
        // This should probably be in a validation step rather than here?
        return WebSocketResponse::bad_request(format!("Can only request a maximum of {} messages at a time", MAX_MSG_COUNT));
    }

    let chat_channel = match get_chat_channel_by_id(&app_state.db, get_state_schema.channel_id.as_str()).await {
        Ok(channel) => channel,
        Err(e) => return e.into(),
    };
    if !chat_channel.subscribers.contains(&user_id) {
        return WebSocketResponse::bad_request("You are not in this chat channel");
    }

    match get_chat_message_span(
        &app_state.db,
        get_state_schema.atomic_message_id,
        get_state_schema.channel_id.as_str(),
        get_state_schema.message_count,
    )
    .await
    {
        Ok(messages) => WebSocketResponse::SendChatState(messages),
        Err(e) => e.into(),
    }
}

async fn edit_message(app_state: &AppState, user_id: String, edit_message_schema: EditMessageSchema) -> Option<WebSocketResponse> {
    let chat_message = match get_chat_message(&app_state.db, edit_message_schema.message_id.as_str()).await {
        Ok(message) => message,
        Err(e) => return Some(e.into()),
    };

    if chat_message.author_id.as_str() != user_id.as_str() {
        return Some(WebSocketResponse::forbidden("You must be the author of a message to edit it"));
    }

    // Get the channel prior to editing the message as we don't want to fail the request after the
    // edit has already been made
    let chat_channel = match get_chat_channel_by_id(&app_state.db, chat_message.channel_id.as_str()).await {
        Ok(channel) => channel,
        Err(e) => return Some(e.into()),
    };

    let edited_message = match update_chat_message(&app_state.db, edit_message_schema).await {
        Ok(edited_msg) => edited_msg,
        Err(e) => return Some(e.into()),
    };

    update_channel_subscribers(app_state, &chat_channel, UpdateKind::UpdateMessage, &edited_message).await;

    // TODO: Should this return something? Probably
    None
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
            let response_to_client: Option<WebSocketResponse> = match serde_json::from_str::<WebSocketRequest>(text.as_str()) {
                // user_id is passed as a clone to all of these rather than given as a borrow to appease a Send impl for the
                // read_task in `handle_socket`
                Ok(WebSocketRequest::CreateMessage(create_msg_schema)) => Some(create_msg(&app_state, user_id.clone(), create_msg_schema).await),
                Ok(WebSocketRequest::GetChatState(request_state_schema)) => {
                    Some(get_chat_state(&app_state, user_id.clone(), request_state_schema).await)
                }
                Ok(WebSocketRequest::EditMessage(edit_msg_schema)) => edit_message(&app_state, user_id.clone(), edit_msg_schema).await,
                // We got an error decoding the text packet using serde json
                Err(_e) => {
                    // TODO: ACTUAL ERROR HANDLING HERE WITH e
                    // Would be nice to give more info to the user here about what failed
                    logger::log_msg("Error while deserializing a websocket packet from a string");
                    Some(WebSocketResponse::bad_request("Failed to decode received data"))
                }
            };

            // Respond to the client who sent this request
            if let Some(response) = response_to_client {
                let connections = app_state.active_connections.read().await;
                match connections.get(user_id.as_str()) {
                    Some(tx) => {
                        let _ = tx.send(response);
                    }
                    None => {
                        // Currently active connection is gone, log this?
                    }
                };
            }
        }
    }

    // Clean up on disconnect
    app_state.active_connections.write().await.remove(user_id.as_str());
}
