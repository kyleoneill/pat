mod chat_websocket;

use super::{get_user_from_auth_header, get_user_from_token};
use crate::api::return_data::ReturnData;
use crate::app::AppState;
use crate::error_handler::DbError;
use crate::models::chat::{
    chat_channel::ReturnChannel,
    chat_channel_db::{get_chat_channel_by_id, hydrate_chat_channel_subscribers, insert_chat_channel, list_chat_channels, update_chat_channel_by_id},
    validation::CreateChannelSchema,
};
use axum::{
    body::Body,
    extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade, Path, Query, State},
    http::{header::HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};

pub fn chat_routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/chat/ws", get(chat_connect))
        .route("/chat/channels", post(create_channel))
        .route("/chat/channels", get(list_channels))
        .route("/chat/channels/subscribe", put(channel_subscribe))
        .route("/chat/channels/unsubscribe", put(channel_unsubscribe))
        .route("/chat/channels/:channel_id", get(get_channel))
}

async fn create_channel(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(channel_data): Json<CreateChannelSchema>,
) -> ReturnData<ReturnChannel> {
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match insert_chat_channel(pool, &channel_data, user.get_id()).await {
        Ok(chat_channel) => ReturnData::created(hydrate_chat_channel_subscribers(pool, chat_channel).await),
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
) -> ReturnData<ReturnChannel> {
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

    // subscribers filter verifies that the user is not already subscribed to this channel
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

    match update_chat_channel_by_id(pool, filter_doc, update_doc).await {
        Ok(chat_channel) => ReturnData::ok(hydrate_chat_channel_subscribers(pool, chat_channel).await),
        Err(db_err) => db_err.into(),
    }
}

async fn channel_unsubscribe(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(channel_data): Json<ChannelSubscribeSchema>,
) -> ReturnData<ReturnChannel> {
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

    match update_chat_channel_by_id(pool, filter_doc, update_doc).await {
        Ok(chat_channel) => ReturnData::ok(hydrate_chat_channel_subscribers(pool, chat_channel).await),
        Err(db_err) => db_err.into(),
    }
}

#[derive(Deserialize)]
struct ListChannelsQueryParams {
    my_channels: Option<bool>,
    subscribed: Option<bool>,
}

async fn list_channels(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    query_params: Query<ListChannelsQueryParams>,
) -> ReturnData<Vec<ReturnChannel>> {
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    let user_id = user.get_id();

    // Build the filter for this listing
    let filter_doc = {
        let mut building_doc = doc! {};

        // Check if we want to filter to owned or un-owned channels
        if query_params.my_channels.is_some() {
            if query_params.my_channels.unwrap() {
                building_doc.insert("owner_id", user_id.clone());
            } else {
                building_doc.insert("owner_id", doc! {"$ne": user_id.clone()});
            }
        }

        // Check if we want to filter for documents that the requester is subscribed to or not
        if let Some(is_subscribed) = query_params.subscribed {
            match is_subscribed {
                true => {
                    building_doc.insert("subscribers", user_id.as_str());
                }
                false => {
                    building_doc.insert(
                        "subscribers",
                        doc! {
                            "$not": {
                                "$elemMatch": {
                                    "$eq": user_id.as_str()
                                }
                            }
                        },
                    );
                }
            }
        };

        building_doc
    };

    match list_chat_channels(pool, filter_doc).await {
        Ok(channels) => {
            let mut return_channels = Vec::new();
            for channel in channels {
                let return_channel = hydrate_chat_channel_subscribers(pool, channel).await;
                return_channels.push(return_channel);
            }
            ReturnData::ok(return_channels)
        }
        Err(db_err) => db_err.into(),
    }
}

async fn get_channel(State(app_state): State<Arc<AppState>>, headers: HeaderMap, Path(channel_id): Path<String>) -> ReturnData<ReturnChannel> {
    let pool = &app_state.db;
    let _user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match get_chat_channel_by_id(pool, channel_id.as_str()).await {
        Ok(channel) => ReturnData::ok(hydrate_chat_channel_subscribers(pool, channel).await),
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
    let pool = &app_state.db;
    let user = match get_user_from_token(pool, query_params.auth_token.as_str(), &app_state.config.app_secret).await {
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

    ws.on_upgrade(move |socket| chat_websocket::handle_socket(socket, addr, user.get_id(), app_state))
}
