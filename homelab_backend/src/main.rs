#[macro_use]
extern crate dotenv_codegen;

use std::time::Duration;
use sqlx::sqlite::SqlitePool;

mod api;
use api::{notes, user};

use axum::{
    routing::get,
    Router
};

use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    ServiceBuilderExt
};
// use tower_http::validate_request::ValidateRequestHeaderLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_max_age: i32,
    pub app_secret: String
}

impl Config {
    pub fn init() -> Self {
        let database_url = dotenv!("DATABASE_URL").to_owned();
        let jwt_secret = dotenv!("JWT_SECRET").to_owned();
        let jwt_max_age = dotenv!("JWT_MAX_AGE").to_owned();
        let app_secret = dotenv!("APP_SECRET").to_owned();
        Self {
            database_url,
            jwt_secret,
            jwt_max_age: jwt_max_age.parse::<i32>().expect("JWT_MAX_AGE was not an i32"),
            app_secret
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Config::init();
    let pool = SqlitePool::connect(config.database_url.as_str()).await.expect("Failed to connect to database");
    let state = AppState { db: pool, config };

    // initialize tracing
    tracing_subscriber::fmt::init();

    let cors = CorsLayer::new().allow_origin(Any);

    let api_routes = Router::<AppState>::new()
        .merge(notes::notes_routes())
        .merge(user::auth_routes());

    let middleware = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(cors)
        //.layer(ValidateRequestHeaderLayer::accept("application/json"))
        .compression();

    // build our application with a route
    let app: Router = Router::<AppState>::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .nest("/api", api_routes)
        .with_state(state)
        .layer(middleware);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

// TODO: Replace root with something
async fn root() -> &'static str {
    "Hello, World!"
}
