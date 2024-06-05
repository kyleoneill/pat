use crate::AppState;
use axum::{routing::get, Router};

pub fn notes_routes() -> Router<AppState> {
    Router::<AppState>::new().route("/notes", get(get_notes()))
}

fn get_notes() -> &'static str {
    "test"
}
