use super::get_user_from_auth_header;
use super::return_data::ReturnData;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::header::HeaderMap,
    routing::{get, post, put},
    Json, Router,
};

use crate::models::games::{
    games_db::{get_all_connections_games, get_connection_game_by_slug, insert_connections_game},
    ConnectionGame, ConnectionGameSchema, MinimalConnectionsGame, PlayConnectionGame, TrySolveRow,
};

pub fn games_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/games/connections", post(create_connections))
        .route("/games/connections", get(list_other_connections_games))
        .route("/games/connections/mine", get(list_my_connections_games))
        .route("/games/connections/play/:game_slug", get(get_game_to_play))
        .route(
            "/games/connections/play/:game_slug/try_solve",
            put(try_solve_row),
        )
}

async fn create_connections(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(connection_data): Json<ConnectionGameSchema>,
) -> ReturnData<ConnectionGame> {
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match insert_connections_game(pool, &connection_data, user.get_id()).await {
        Ok(connection_game) => ReturnData::created(connection_game),
        Err(db_err) => db_err.into(),
    }
}

async fn list_my_connections_games(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> ReturnData<Vec<MinimalConnectionsGame>> {
    // TODO: This should be paginated
    // TODO: This and list_other_connections_games should both just call a shared function passing it a true/false
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match get_all_connections_games(pool, user.get_id().as_str(), true).await {
        Ok(connections_games) => {
            let minimized_games = connections_games
                .into_iter()
                .map(|game| game.into())
                .collect();
            ReturnData::ok(minimized_games)
        }
        Err(db_err) => db_err.into(),
    }
}

async fn list_other_connections_games(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> ReturnData<Vec<MinimalConnectionsGame>> {
    // TODO: This should be paginated
    let pool = &app_state.db;
    let user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match get_all_connections_games(pool, user.get_id().as_str(), false).await {
        Ok(connections_games) => {
            let minimized_games = connections_games
                .into_iter()
                .map(|game| game.into())
                .collect();
            ReturnData::ok(minimized_games)
        }
        Err(db_err) => db_err.into(),
    }
}

async fn get_game_to_play(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(game_slug): Path<String>,
) -> ReturnData<PlayConnectionGame> {
    let pool = &app_state.db;
    let _user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await
    {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match get_connection_game_by_slug(pool, game_slug.as_str()).await {
        Ok(connections_game) => ReturnData::ok(connections_game.into()),
        Err(db_err) => db_err.into(),
    }
}

async fn try_solve_row(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Path(game_slug): Path<String>,
    Json(row_guess): Json<[String; 4]>,
) -> ReturnData<TrySolveRow> {
    let pool = &app_state.db;
    let _user = match get_user_from_auth_header(pool, &headers, &app_state.config.app_secret).await
    {
        Ok(user) => user,
        Err(e) => return e.into(),
    };
    match get_connection_game_by_slug(pool, game_slug.as_str()).await {
        Ok(connections_game) => {
            let (row_name, correct_guess) =
                check_if_solution_is_valid(&connections_game, &row_guess);
            ReturnData::ok(TrySolveRow {
                row_name,
                correct_guess,
            })
        }
        Err(db_err) => db_err.into(),
    }
}

fn check_if_solution_is_valid(
    game: &ConnectionGame,
    guess: &[String; 4],
) -> (Option<String>, bool) {
    'outer: for category in &game.connection_categories {
        for word in guess {
            if !category.category_clues.contains(word) {
                continue 'outer;
            }
        }
        return (Some(category.to_owned().category_name), true);
    }
    (None, false)
}
