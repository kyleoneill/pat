use crate::models::games::{
    ConnectionGame, ConnectionGameSchema, MinimalConnectionsGame, PlayConnectionGame, TrySolveRow,
};
use crate::testing::helpers::read_error_message;
use crate::testing::json_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
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
    let req = Request::builder()
        .uri(format!("http://{addr}/api/games/connections"))
        .method("POST")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(json_bytes(json!(connection_game))))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::CREATED => (),
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            return Err((status, message));
        }
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn list_connections_games(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    my_connections_games: bool,
) -> Result<Vec<MinimalConnectionsGame>, (StatusCode, String)> {
    let uri = match my_connections_games {
        true => format!("http://{addr}/api/games/connections/mine"),
        false => format!("http://{addr}/api/games/connections"),
    };
    let req = Request::builder()
        .uri(uri)
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::empty())
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::OK => (),
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            return Err((status, message));
        }
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn get_game_to_play(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    game_slug: &str,
) -> Result<PlayConnectionGame, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!(
            "http://{addr}/api/games/connections/play/{game_slug}"
        ))
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::empty())
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::OK => (),
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            return Err((status, message));
        }
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}

pub async fn try_connections_solution(
    client: &Client<HttpConnector, Body>,
    addr: &SocketAddr,
    token: &str,
    game_slug: &str,
    guess: [String; 4],
) -> Result<TrySolveRow, (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!(
            "http://{addr}/api/games/connections/play/{game_slug}/try_solve"
        ))
        .method("PUT")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(json_bytes(json!(guess))))
        .unwrap();
    let res = client.request(req).await.unwrap();
    match res.status() {
        StatusCode::OK => (),
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            return Err((status, message));
        }
    }
    let body = res.into_body().collect().await.unwrap().to_bytes();
    Ok(serde_json::from_slice(body.as_ref()).unwrap())
}
