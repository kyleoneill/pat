use super::{get_user_from_auth_header, get_user_from_token};
use crate::api::return_data::ReturnData;
use crate::models::chat::{
    chat_channel::{ChatChannel, CreateChannelSchema},
    chat_channel_db::{insert_chat_channel, update_chat_channel_by_id},
};
use crate::AppState;
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
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::error_handler::DbError;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Bson, Document};

pub fn chat_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/chat/ws", get(chat_connect))
        .route("/chat/channel", post(create_channel))
        .route("/chat/channel/subscribe", put(channel_subscribe))
        .route("/chat/channel/unsubscribe", put(channel_unsubscribe))
}

async fn create_channel(
    State(app_state): State<AppState>,
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
    State(app_state): State<AppState>,
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
    State(app_state): State<AppState>,
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

#[derive(Deserialize, Debug)]
struct ChatConnectQueryParams {
    auth_token: String,
}

async fn chat_connect(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
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
    println!("{:?}", user);
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    // TODO: HANDLE SOCKET
    // TODO: NEED SOMETHING IN STATE TO HANDLE SOCKETS???????
    // how to handle keeping a socket open, and when to close it?
}
