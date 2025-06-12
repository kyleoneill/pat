use super::{get_user_from_auth_header, get_user_from_token};
use crate::api::return_data::ReturnData;
use crate::models::chat::{
    chat_channel::{ChatChannel, CreateChannelSchema},
    chat_channel_db::{insert_chat_channel, update_chat_channel_by_id, get_chat_channel_by_id},
    packet::{
        WebsocketMessage,
        WebsocketAck,
    },
};
use crate::{logger, AppState};
use axum::{
    body::Body,
    extract::{
        connect_info::ConnectInfo,
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header::HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use tokio::sync::mpsc;
use serde::{Deserialize, Deserializer, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use crate::error_handler::DbError;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Bson, Document};

pub fn chat_routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/chat/ws", get(chat_connect))
        .route("/chat/channel", post(create_channel))
        .route("/chat/channel/subscribe", put(channel_subscribe))
        .route("/chat/channel/unsubscribe", put(channel_unsubscribe))
}

async fn create_channel(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(channel_data): Json<CreateChannelSchema>,
) -> ReturnData<ChatChannel> {
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match insert_chat_channel(pool, &channel_data, user.get_id()).await {
        Ok(chat_channel) => ReturnData::created(chat_channel),
        Err(db_err) => db_err.into(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChannelSubscribeSchema {
    pub channel_id: String,
}

async fn channel_subscribe(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(channel_data): Json<ChannelSubscribeSchema>,
) -> ReturnData<ChatChannel> {
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    let user_id = user.get_id();

    let channel_id: ObjectId = match channel_data.channel_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return DbError::BadId.into(),
    };

    let filter_doc: Document = doc! {
        "_id": Bson::ObjectId(channel_id),
        "subscribers": {
            "$not": {
                "$elemMatch": {
                    "$eq": user_id.as_str()
                }
            }
        }
    };
    let update_doc: Document = doc! {
        "$push": {"subscribers": user_id.as_str()}
    };

    match update_chat_channel_by_id(
        pool,
        channel_data.channel_id.as_str(),
        filter_doc,
        update_doc,
    )
    .await
    {
        Ok(chat_channel) => ReturnData::ok(chat_channel),
        Err(db_err) => db_err.into(),
    }
}

async fn channel_unsubscribe(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(channel_data): Json<ChannelSubscribeSchema>,
) -> ReturnData<ChatChannel> {
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    let user_id = user.get_id();

    let channel_id: ObjectId = match channel_data.channel_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return DbError::BadId.into(),
    };

    // TODO: Should probably return a more meaningful error here than a 404 if a user tries to
    //       unsubscribe from a channel they aren't in, or their own channel
    let filter_doc: Document = doc! {
        "_id": Bson::ObjectId(channel_id),
        "subscribers": user_id.as_str(),
        "owner_id": {"$ne": user_id.as_str()},
    };
    let update_doc: Document = doc! {
        "$pull": {"subscribers": user_id.as_str()}
    };

    match update_chat_channel_by_id(
        pool,
        channel_data.channel_id.as_str(),
        filter_doc,
        update_doc,
    )
    .await
    {
        Ok(chat_channel) => ReturnData::ok(chat_channel),
        Err(db_err) => db_err.into(),
    }
}

// WEBSOCKET
#[derive(Deserialize, Debug)]
struct ChatConnectQueryParams {
    auth_token: String,
}

async fn chat_connect(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<AppState>>,
    query_params: Query<ChatConnectQueryParams>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    // TODO: Having the auth token in the request causes it to be logged, which is not ideal
    //       Maybe have the logger check for this specific query_param and replace it with '<redacted>'
    //       or something
    let pool = &app_state.db;
    let user = match get_user_from_token(
        pool,
        query_params.auth_token.as_str(),
        &app_state.config.app_secret,
    )
    .await
    {
        Ok(user) => user,
        Err(_e) => {
            // Must return a Response<Body> here due to the return value of ws.on_upgrade
            // TODO: Handle converting ws.on_upgrade response into a ReturnData response so I can
            //       just return `ResponseData::from(e)` here.
            //       ws.on_upgrade must return a 101 with specific headers set
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Failed to authorize websocket request"))
                .unwrap();
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, addr, user.get_id(), app_state))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, user_id: String, app_state: Arc<AppState>) {
    /*
        - While true, read bytes
        - When done reading bytes, determine what kind of packet we have
            - The first byte will determine the packet type
        - Deserialize the bytes into our packet
        - Handle the packet
     */
    // Create a channel to send messages
    let (tx, mut rx) = mpsc::unbounded_channel::<WebsocketMessage>();
    
    // Register the connection
    {
        let mut connections = app_state.active_connections.lock().await;
        
        // Check if this user already has a connection
        if connections.contains_key(user_id.as_str()) {
            connections.remove(user_id.as_str()).expect("Hashmap should contain a value when .contains_key was true");
        }
        
        connections.insert(user_id.clone(), tx);
    }

    let (mut sender, mut receiver) = socket.split();

    // TODO: Response to connection being established - send most recent message id in subscribed channels?

    // Spawn a task to receive messages from the socket
    let cloned_user_id = user_id.clone();
    let cloned_state = app_state.clone();
    let read_task = tokio::spawn(async move {
        // Axum and the client will handle transmitting a heartbeat, so this will always return a Some
        // until the connection is closed. When receiver.next() gets a None, the while loop is terminated
        while let Some(Ok(msg)) = receiver.next().await {
            // Can make this connection more resilient by doing `while let Some(maybe_errd_msg) = receiver.next().await
            // and then matching on maybe_errd_msg to handle the non-fatal-error myself, but that might not matter for
            // a simple chat app. As is currently written, a non-fatal-error will cause the `while let` to fail,
            // ending the while loop and then the async block
            
            // TODO: MATCH HERE INSTEAD OF IF_LET AND THEN HANDLE MESSAGE::CLOSE
            if let Message::Text(text) = msg {
                match serde_json::from_str::<WebsocketMessage>(text.as_str()) {
                    Ok(WebsocketMessage::ReceiveChatMessage(msg_to_create)) => {
                        // Construct a response that will be sent to the client which sent this request
                        let mut ack = WebsocketAck::new();

                        // Verify that the message is being sent to a valid channel that the sender is in
                        let channel_id = msg_to_create.channel_id;
                        match get_chat_channel_by_id(&cloned_state.db, channel_id.as_str()).await {
                            Ok(channel) => {
                                // Verify that the current user is a subscriber of the channel
                                // Create a db entry for this message
                                // Check active connections to see if any subscribers to the channel are connected
                                // for each connected user, get their tx and send a SendChatMessage to it
                            },
                            Err(_) => {
                                // TODO: Actual error handling
                                ack.status_code = 0;
                                ack.msg.push_str("Chat channel does not exist");
                            }
                        }

                        // Send the ack to the client who sent this request
                        let connections = cloned_state.active_connections.lock().await;
                        match connections.get(cloned_user_id.as_str()) {
                            Some(tx) => {
                                let _ = tx.send(WebsocketMessage::SendAck(ack));
                            },
                            None => {
                                // Currently active connection is gone, log this?
                            },
                        };


                        // TEMP EXAMPLE OF WHAT THIS LOOKS LIKE
                        // let connections = state_clone.connections.lock().await;
                        // for (uid, tx) in connections.iter() {
                        //     if *uid != user_id {
                        //         let _ = tx.send(WebSocketMessage::ChatMessage(chat_msg.clone()));
                        //     }
                        // }
                    },
                    Ok(WebsocketMessage::ReceiveChatUpdateRequest(msg_request)) => {
                        // Get messages from db
                        // Send messages to the requestor

                        // TEMP EXAMPLE OF WHAT THIS LOOKS LIKE
                        // let connections = state_clone.connections.lock().await;
                        // if let Some(tx) = connections.get(&user_id) {
                        //     let _ = tx.send(history);
                        // }
                    },
                    Ok(WebsocketMessage::SendChatMessage(_)) => {
                        // This should never be received, this variant is meant to be sent
                    },
                    Ok(WebsocketMessage::SendAck(_)) => {
                        // This should never be received, this variant is meant to be sent
                    },
                    Err(_e) => {
                        // TODO: Handle error
                        //       Send a SendErrorMessage to tx?
                        logger::log_msg("Error while deserializing a websocket packet from a string");
                    }
                }
            }
        }

        // Clean up on disconnect
        cloned_state.active_connections.lock().await.remove(cloned_user_id.as_str());
    });

    // Spawn a task to send messages to the socket
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // if msg is SendChatMessage
            //  - get subscribers from the channel
            //  - for each subscriber
            //    - if user_id in active_connections
            //      - send message to that connection
            // other response types
            //  - SendErrorMessage
            match msg {
                WebsocketMessage::ReceiveChatMessage(_) => {
                    // Should not occur
                },
                WebsocketMessage::ReceiveChatUpdateRequest(_) => {
                    // Should not occur
                },
                WebsocketMessage::SendChatMessage(message_to_send) => {
                    // TODO: SEND MESSAGE
                },
                WebsocketMessage::SendAck(ack) => {
                    // TODO: SEND ACK
                },
            }

            // this should be moved into WebSocketMessage specific variant handling
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(text)).await.is_err() {
                    // TODO: HANDLE ERROR HERE
                    logger::log_msg("Error while sending a message to a sender");
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
    app_state.active_connections.lock().await.remove(user_id.as_str());
}
