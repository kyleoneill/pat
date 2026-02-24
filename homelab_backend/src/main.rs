#[macro_use]
extern crate dotenv_codegen;

use std::net::SocketAddr;

mod api;
pub mod app;
mod db;
pub mod error_handler;
mod logger;
mod models;
mod tasks;
mod testing;
pub mod util;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Set up database pool
    let connection_string = dotenv!("CONNECTION_STRING").to_owned();
    let database = db::db_setup::initialize_database_handle(connection_string, "home_server_db").await;

    // run our app with hyper, listening on 127.0.0.1:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    logger::log_msg(format!("listening on {}", listener.local_addr().unwrap()));
    let (app, _) = app::generate_app(database).await;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
