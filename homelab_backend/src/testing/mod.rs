#![allow(dead_code)]

mod chat_testing;
mod games_testing;
mod helpers;
mod log_testing;
mod reminder_testing;
mod user_testing;

use crate::{app::generate_app, db::db_setup, tasks::task_manager::TaskManager};
use axum::body::Body;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use serde::Serialize;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

// This ID is almost guaranteed to never exist :)
const FAKE_MONGO_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaa";

pub struct TestHelper {
    pub client: Client<HttpConnector, Body>,
    pub address: SocketAddr,
    pub database: Database,
    pub task_manager: Arc<Mutex<TaskManager>>,
}

impl TestHelper {
    pub async fn init() -> Self {
        let connection_string = dotenv!("CONNECTION_STRING").to_owned();
        let database = db_setup::initialize_database_handle(connection_string, "test_db").await;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (app, task_manager) = generate_app(database.clone()).await;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .unwrap()
        });
        let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();
        let helper = Self {
            client,
            address,
            database,
            task_manager,
        };
        helper.wipe_database().await;
        helper
    }

    pub async fn wipe_database(&self) {
        // Get all collections in the test database and wipe each of them
        let collections: Vec<String> = self
            .database
            .list_collection_names()
            .await
            .expect("Failed to get collection data while wiping database during test setup");
        for collection_name in collections {
            let collection: Collection<Document> = self.database.collection(collection_name.as_str());
            // An empty filter doc will grab all documents in the collection
            let _res = collection.delete_many(doc! {}).await;
        }
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
