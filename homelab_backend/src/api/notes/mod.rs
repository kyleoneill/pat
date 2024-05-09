use axum::{
    Router,
    routing::{get}
};
use crate::AppState;

pub fn notes_routes() -> Router<AppState> {
    Router::<AppState>::new().route("/notes", get(get_notes()))
}

fn get_notes() -> &'static str {
    "test"
}
