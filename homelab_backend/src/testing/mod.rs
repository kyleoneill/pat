#![allow(dead_code)]

mod chat_testing;
mod games_testing;
mod helpers;
mod log_testing;
mod reminder_testing;
mod user_testing;

use crate::models::chat::chat_channel::ChatChannel;
use crate::models::games::ConnectionGame;
use crate::models::log::Log;
use crate::models::reminder::{Category, Reminder};
use crate::models::user::User;
use crate::{db, generate_app};
use axum::body::Body;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct TestHelper {
    pub client: Client<HttpConnector, Body>,
    pub address: SocketAddr,
    pub database: Database,
}

impl TestHelper {
    pub async fn init() -> Self {
        let connection_string = dotenv!("CONNECTION_STRING").to_owned();
        let database = db::initialize_database_handle(connection_string, "test_db").await;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = generate_app(database.clone()).await;
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();
        let helper = Self {
            client,
            address,
            database,
        };
        helper.wipe_database().await;
        helper
    }

    pub async fn wipe_database(&self) {
        // TODO: This is not sustainable, what if 100 collections are added?

        let user_collection: Collection<User> = self.database.collection("users");
        let _res = user_collection.delete_many(doc! {}).await;

        let log_collection: Collection<Log> = self.database.collection("logs");
        let _res = log_collection.delete_many(doc! {}).await;

        let categories_collection: Collection<Category> = self.database.collection("categories");
        let _res = categories_collection.delete_many(doc! {}).await;

        let reminders_collection: Collection<Reminder> = self.database.collection("reminders");
        let _res = reminders_collection.delete_many(doc! {}).await;

        let games_collection: Collection<ConnectionGame> =
            self.database.collection("game_connections");
        let _res = games_collection.delete_many(doc! {}).await;

        let chat_channels_collection: Collection<ChatChannel> =
            self.database.collection("chat_channels");
        let _res = chat_channels_collection.delete_many(doc! {}).await;
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
