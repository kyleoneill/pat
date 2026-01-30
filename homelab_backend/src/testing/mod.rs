#![allow(dead_code)]

mod chat_testing;
mod games_testing;
mod helpers;
mod log_testing;
mod reminder_testing;
mod user_testing;

use crate::{
    db::{db_setup, MongoModel},
    generate_app,
    models::{
        chat::{chat_channel::ChatChannel, message::ChatMessage},
        games::ConnectionGame,
        log::Log,
        reminder::{Category, Reminder},
        user::User,
    },
};
use axum::body::Body;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use mongodb::{bson::doc, Collection, Database};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

// This ID is almost guaranteed to never exist :)
const FAKE_MONGO_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaa";

pub struct TestHelper {
    pub client: Client<HttpConnector, Body>,
    pub address: SocketAddr,
    pub database: Database,
}

impl TestHelper {
    pub async fn init() -> Self {
        let connection_string = dotenv!("CONNECTION_STRING").to_owned();
        let database = db_setup::initialize_database_handle(connection_string, "test_db").await;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = generate_app(database.clone()).await;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .unwrap()
        });
        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();
        let helper = Self { client, address, database };
        helper.wipe_database().await;
        helper
    }

    pub async fn wipe_database(&self) {
        // TODO: This is not sustainable, what if 100 collections are added?

        let user_collection: Collection<User> = self.database.collection(User::collection_name());
        let _res = user_collection.delete_many(doc! {}).await;

        let log_collection: Collection<Log> = self.database.collection(Log::collection_name());
        let _res = log_collection.delete_many(doc! {}).await;

        let categories_collection: Collection<Category> = self.database.collection(Category::collection_name());
        let _res = categories_collection.delete_many(doc! {}).await;

        let reminders_collection: Collection<Reminder> = self.database.collection(Reminder::collection_name());
        let _res = reminders_collection.delete_many(doc! {}).await;

        let games_collection: Collection<ConnectionGame> = self.database.collection(ConnectionGame::collection_name());
        let _res = games_collection.delete_many(doc! {}).await;

        let chat_channels_collection: Collection<ChatChannel> = self.database.collection(ChatChannel::collection_name());
        let _res = chat_channels_collection.delete_many(doc! {}).await;

        let chat_messages_collection: Collection<ChatMessage> = self.database.collection(ChatMessage::collection_name());
        let _res = chat_messages_collection.delete_many(doc! {}).await;
    }
}

#[allow(dead_code)]
fn json_bytes<T>(structure: T) -> Vec<u8>
where
    T: Serialize,
{
    let mut bytes: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut bytes, &structure).unwrap();
    bytes
}
