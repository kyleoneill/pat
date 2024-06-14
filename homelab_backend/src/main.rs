#[macro_use]
extern crate dotenv_codegen;

use sqlx::sqlite::SqlitePool;
use std::time::Duration;

mod api;
mod testing;

use api::{logs, notes, user};

use axum::{http::Request, routing::get, Router};

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
        let test_database_url = dotenv!("DATABASE_URL").to_owned();
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

pub async fn generate_app(is_test_app: bool) -> Router {
    let config = Config::init();
    let databse_url = match is_test_app {
        true => config.database_url.as_str(),
        false => config.test_database_url.as_str(),
    };
    let pool = SqlitePool::connect(databse_url)
        .await
        .expect("Failed to connect to database");

    let app_secret = config.app_secret.clone();
    let handle = pool.clone();

    let state = AppState { db: pool, config };

    let api_routes = Router::<AppState>::new()
        .merge(notes::notes_routes())
        .merge(user::user_routes())
        .merge(logs::log_routes());

    let cors = CorsLayer::new().allow_origin(Any);

    let trace_layer = TraceLayer::new_for_http()
        //.make_span_with(|request: &Request<_>| {})
        .on_request(move |request: &Request<_>, _span: &Span| {
            // If a request does not have an associated user id, mark it as -1
            let user_id =
                user::jwt::get_and_decode_auth_token(request.headers(), app_secret.as_str())
                    .unwrap_or(-1);
            // TODO: This does not work. I need to await this in order for it to run, but the closure
            //       cannot be async. Will likely need to
            //       1. Create a task system
            //       2. Save this log to a vec which itself is in an Arc or some global
            //       3. Create a task which runs every x seconds and empties the log vec, writing
            //          each item in it to the database
            #[allow(clippy::let_underscore_future)]
            let _ = logs::create_log(
                &handle,
                request.method().to_string(),
                request.uri().to_string(),
                user_id,
            );
        });
    // .on_response(|_response: &Response, _latency: Duration, _span: &Span| {})
    // .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {})
    // .on_eos(|_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {})
    // .on_failure(|_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {});

    let middleware = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(cors)
        //.layer(ValidateRequestHeaderLayer::accept("application/json"))
        .compression();

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

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    let app = generate_app(false).await;
    axum::serve(listener, app).await.unwrap();
}

// TODO: Replace root with something
async fn root() -> &'static str {
    "Hello, World!"
}
