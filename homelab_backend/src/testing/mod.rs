#![allow(dead_code)]

mod helpers;
mod log_testing;
mod user_testing;

use crate::generate_app;
use axum::body::Body;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct TestHelper {
    pub client: Client<HttpConnector, Body>,
    pub address: SocketAddr,
    pub pool: SqlitePool,
}

impl TestHelper {
    pub async fn init() -> Self {
        let database_url = dotenv!("TEST_DATABASE_URL").to_owned();
        let pool = SqlitePool::connect(database_url.as_str())
            .await
            .expect("Failed to connect to database");

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = generate_app(pool.clone()).await;
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();
        let helper = Self {
            client,
            address,
            pool,
        };
        helper.wipe_database().await;
        helper
    }

    pub async fn wipe_database(&self) {
        let _ = sqlx::query!(
            "\
            DELETE FROM users;\
            DELETE FROM logs;\
            UPDATE `main`.`sqlite_sequence` SET `seq` = '0' WHERE  `name` = 'users';\
            UPDATE `main`.`sqlite_sequence` SET `seq` = '0' WHERE  `name` = 'logs';\
            INSERT INTO users (username, password, auth_level, salt) VALUES ('admin', 'D600AD1AAEA6261F2B5923FE076AE08B42688CDF6051FEF2D8CC4ED303D19E22', 'Admin', 'zTGNpsiiXQ5f');\
            "
        ).execute(&self.pool).await;
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
