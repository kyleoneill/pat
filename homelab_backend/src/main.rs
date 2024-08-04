#[macro_use]
extern crate dotenv_codegen;

use sqlx::sqlite::SqlitePool;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod api;
pub mod error_handler;
mod models;
mod testing;

use api::{logs, notes, reminder_controller, user};

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

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub test_database_url: String,
    pub jwt_secret: String,
    pub jwt_max_age: i32,
    pub app_secret: String,
}

impl Config {
    pub fn init() -> Self {
        let database_url = dotenv!("DATABASE_URL").to_owned();
        let test_database_url = dotenv!("TEST_DATABASE_URL").to_owned();
        let jwt_secret = dotenv!("JWT_SECRET").to_owned();
        let jwt_max_age = dotenv!("JWT_MAX_AGE").to_owned();
        let app_secret = dotenv!("APP_SECRET").to_owned();
        Self {
            database_url,
            test_database_url,
            jwt_secret,
            jwt_max_age: jwt_max_age
                .parse::<i32>()
                .expect("JWT_MAX_AGE was not an i32"),
            app_secret,
        }
    }
}

pub async fn generate_app(pool: SqlitePool) -> Router {
    // Set up app config and the database pool
    let config = Config::init();

    // Copy the app secret and db handle as they are passed to tasks and on_request events
    let app_secret = config.app_secret.clone();
    let handle = pool.clone();

    // Define the API routes
    let api_routes = Router::<AppState>::new()
        .merge(notes::notes_routes())
        .merge(user::user_routes())
        .merge(logs::log_routes())
        .merge(reminder_controller::reminder_routes());

    // Create a channel to pass log information to the db write task
    let (log_tx, log_rx) = mpsc::channel();

    // Define the trace layer which handles request/response events
    let trace_layer = TraceLayer::new_for_http()
        //.make_span_with(|request: &Request<_>| {})
        .on_request(move |request: &Request<_>, _span: &Span| {
            // If a request does not have an associated user id, mark it as -1
            let user_id =
                user::jwt::get_and_decode_auth_token(request.headers(), app_secret.as_str())
                    .unwrap_or(-1);
            let date_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let new_task = logs::tasks::LogCreationTask::new(
                request.method().to_string(),
                request.uri().to_string(),
                user_id,
                date_time,
            );
            log_tx.send(new_task).unwrap();
        });
    // .on_response(|_response: &Response, _latency: Duration, _span: &Span| {})
    // .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {})
    // .on_eos(|_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {})
    // .on_failure(|_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {});

    // Set up middleware, currently a timeout and CORS config
    let middleware = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CorsLayer::new().allow_origin(Any).allow_headers(Any))
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
    let state = AppState { db: pool, config };
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
    let database_url = dotenv!("DATABASE_URL").to_owned();
    let pool = SqlitePool::connect(database_url.as_str())
        .await
        .expect("Failed to connect to database");

    // run our app with hyper, listening on 127.0.0.1:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, generate_app(pool).await)
        .await
        .unwrap();
}

// TODO: Replace root with something
async fn root() -> &'static str {
    "Hello, World!"
}
