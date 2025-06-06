#[macro_use]
extern crate dotenv_codegen;

use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod api;
mod db;
pub mod error_handler;
mod models;
mod tasks;
mod testing;

use models::user::jwt::get_and_decode_auth_token;

use api::{games_controller, log_controller, reminder_controller, user_controller};

use axum::http::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, AUTHORIZATION, CONNECTION, CONTENT_TYPE, DNT, HOST,
    ORIGIN, REFERER, USER_AGENT,
};
use axum::http::Method;
use axum::{http::Request, routing::get, Router};
use tokio::{task, time};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
    ServiceBuilderExt,
};

use tracing::Span;

use mongodb::Database;

const LOGGABLE_METHODS: [Method; 4] = [Method::GET, Method::PUT, Method::POST, Method::DELETE];

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub connection_string: String,
    pub jwt_secret: String,
    pub jwt_max_age: i32,
    pub app_secret: String,
}

impl Config {
    pub fn init() -> Self {
        let connection_string = dotenv!("CONNECTION_STRING").to_owned();
        let jwt_secret = dotenv!("JWT_SECRET").to_owned();
        let jwt_max_age = dotenv!("JWT_MAX_AGE").to_owned();
        let app_secret = dotenv!("APP_SECRET").to_owned();
        Self {
            connection_string,
            jwt_secret,
            jwt_max_age: jwt_max_age
                .parse::<i32>()
                .expect("JWT_MAX_AGE was not an i32"),
            app_secret,
        }
    }
}

pub async fn generate_app(database: Database) -> Router {
    // Set up app config and the database pool
    let config = Config::init();

    // Copy the app secret and db handle as they are passed to tasks and on_request events
    let app_secret = config.app_secret.clone();
    let handle = database.clone();

    // Define the API routes
    let api_routes = Router::<AppState>::new()
        .merge(user_controller::user_routes())
        .merge(log_controller::log_routes())
        .merge(reminder_controller::reminder_routes())
        .merge(games_controller::games_routes());

    // Create a channel to pass log information to the db write task
    let (log_tx, log_rx) = mpsc::channel();

    // Define the trace layer which handles request/response events
    let trace_layer = TraceLayer::new_for_http()
        //.make_span_with(|request: &Request<_>| {})
        .on_request(move |request: &Request<_>, _span: &Span| {
            if LOGGABLE_METHODS.contains(request.method()) {
                // If a request does not have an associated user id, mark it as -1
                let user_id = get_and_decode_auth_token(request.headers(), app_secret.as_str())
                    .unwrap_or("-1".to_string());
                let date_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                let new_task = tasks::log_creation_task::LogCreationTask::new(
                    request.method().to_string(),
                    request.uri().to_string(),
                    user_id,
                    date_time,
                );
                log_tx.send(new_task).unwrap();
            }
        });
    // .on_response(|_response: &Response, _latency: Duration, _span: &Span| {})
    // .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {})
    // .on_eos(|_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {})
    // .on_failure(|_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {});

    // Set up middleware, currently a timeout and CORS config
    let middleware = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        // TODO: Getting a cross origin warning on the frontend that allow_headers(Any) will not be supported "soon".
        //       Will need to explicitly list headers that are allowed, like Authorization
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers([
                    AUTHORIZATION,
                    ACCEPT,
                    ACCEPT_ENCODING,
                    ACCEPT_LANGUAGE,
                    CONNECTION,
                    DNT,
                    HOST,
                    ORIGIN,
                    REFERER,
                    USER_AGENT,
                    CONTENT_TYPE,
                ])
                .allow_methods(Any),
        )
        //.layer(ValidateRequestHeaderLayer::accept("application/json"))
        .compression();

    // Set up tasks
    #[allow(clippy::let_underscore_future)]
    let _create_request_logs = task::spawn(async move {
        // Log creation task will run every 5 seconds
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            // TODO: This is making one database op per request, I should be batching these inserts
            // Ex, make a collection after the tick().await, fill it in the while loop, and then
            //     after the while loop make one batched db insert
            while let Ok(rec) = log_rx.try_recv() {
                rec.run_task(&handle).await;
            }
        }
    });

    // Create app state and the router
    let state = AppState {
        db: database,
        config,
    };
    Router::<AppState>::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .nest("/api", api_routes)
        .with_state(state)
        .layer(middleware)
        .layer(trace_layer)
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Set up database pool
    let connection_string = dotenv!("CONNECTION_STRING").to_owned();
    let database = db::initialize_database_handle(connection_string, "home_server_db").await;

    // run our app with hyper, listening on 127.0.0.1:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, generate_app(database).await)
        .await
        .unwrap();
}

// TODO: Replace root with something
async fn root() -> &'static str {
    "Hello, World!"
}
