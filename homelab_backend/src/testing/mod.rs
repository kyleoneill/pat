mod user_testing;

use crate::generate_app;
use axum::body::Body;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use std::sync::OnceLock;
use tokio::net::TcpListener;

#[allow(dead_code)]
pub static SERVER: OnceLock<Client<HttpConnector, Body>> = OnceLock::new();
#[allow(dead_code)]
pub static ADDR: OnceLock<SocketAddr> = OnceLock::new();

#[allow(dead_code)]
pub async fn setup_database() {
    let database_url = dotenv!("DATABASE_URL").to_owned();
    let pool = SqlitePool::connect(database_url.as_str())
        .await
        .expect("Failed to connect to database");
    let _ = sqlx::query!(
            "\
            DELETE FROM users;\
            UPDATE `main`.`sqlite_sequence` SET `seq` = '0' WHERE  `name` = 'users';\
            INSERT INTO users (username, password, auth_level, salt) VALUES ('admin', 'D600AD1AAEA6261F2B5923FE076AE08B42688CDF6051FEF2D8CC4ED303D19E22', 'Admin', 'zTGNpsiiXQ5f');\
            "
        ).execute(&pool).await;
}

#[allow(dead_code)]
pub async fn initialize() {
    setup_database().await;
    match SERVER.get() {
        Some(_val) => (),
        None => {
            let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
            let addr = listener.local_addr().unwrap();
            // TODO: Might want to store this JoinHandle from spawn() so I can clean up resources
            //       correctly at the end of testing?
            tokio::spawn(async move {
                axum::serve(listener, generate_app(true).await)
                    .await
                    .unwrap()
            });
            let client =
                hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                    .build_http();
            SERVER.set(client).unwrap();
            ADDR.set(addr).unwrap();
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
