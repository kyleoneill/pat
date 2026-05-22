use crate::models::games::{validation::CreateConnectionGameSchema, ConnectionGame, MinimalConnectionsGame, PlayConnectionGame, TrySolveRow};
use crate::testing::{
    helpers::{get_request, post_request, put_request},
    TestHelper,
};
use axum::http::StatusCode;
use serde_json::json;

pub async fn create_connections_game(
    test_helper: &TestHelper,
    token: &str,
    connection_game: &CreateConnectionGameSchema,
) -> Result<ConnectionGame, (StatusCode, String)> {
    let data = json!(connection_game);
    post_request(test_helper, "/games/connections", data, Some(token)).await
}

pub async fn list_connections_games(
    test_helper: &TestHelper,
    token: &str,
    my_connections_games: bool,
) -> Result<Vec<MinimalConnectionsGame>, (StatusCode, String)> {
    let uri = match my_connections_games {
        true => "/games/connections/mine",
        false => "/games/connections",
    };
    get_request(test_helper, uri, token).await
}

pub async fn get_game_to_play(test_helper: &TestHelper, token: &str, game_slug: &str) -> Result<PlayConnectionGame, (StatusCode, String)> {
    let path = format!("/games/connections/play/{game_slug}");
    get_request(test_helper, path.as_str(), token).await
}

pub async fn try_connections_solution(
    test_helper: &TestHelper,
    token: &str,
    game_slug: &str,
    guess: [String; 4],
) -> Result<TrySolveRow, (StatusCode, String)> {
    let path = format!("/games/connections/play/{game_slug}/try_solve");
    let data = json!(guess);
    put_request(test_helper, path.as_str(), data, token).await
}
