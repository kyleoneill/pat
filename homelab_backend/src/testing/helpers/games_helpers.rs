use crate::models::games::{
    ConnectionGame, ConnectionGameSchema, MinimalConnectionsGame, PlayConnectionGame, TrySolveRow,
};
use crate::testing::helpers::{get_request, post_request, put_request};
use axum::body::Body;
use axum::http::StatusCode;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde_json::json;
use std::net::SocketAddr;

pub async fn create_connections_game(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    connection_game: &ConnectionGameSchema,
) -> Result<ConnectionGame, (StatusCode, String)> {
    let data = json!(connection_game);
    post_request(client, "/games/connections", data, Some(token), addr).await
}

pub async fn list_connections_games(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    my_connections_games: bool,
) -> Result<Vec<MinimalConnectionsGame>, (StatusCode, String)> {
    let uri = match my_connections_games {
        true => "/games/connections/mine",
        false => "/games/connections",
    };
    get_request(client, uri, token, addr).await
}

pub async fn get_game_to_play(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    game_slug: &str,
) -> Result<PlayConnectionGame, (StatusCode, String)> {
    let path = format!("/games/connections/play/{game_slug}");
    get_request(client, path.as_str(), token, addr).await
}

pub async fn try_connections_solution(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    game_slug: &str,
    guess: [String; 4],
) -> Result<TrySolveRow, (StatusCode, String)> {
    let path = format!("/games/connections/play/{game_slug}/try_solve");
    let data = json!(guess);
    put_request(client, path.as_str(), data, token, addr).await
}
