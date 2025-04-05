use crate::testing::json_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use serde::{Deserialize, Serialize};
use serde_json::{value::Value, Map};
use std::fmt;
use std::net::SocketAddr;

pub mod games_helpers;
pub mod log_helpers;
pub mod reminder_helpers;
pub mod user_helpers;

pub fn list_to_query_params<T>(list_name: &str, items: Vec<T>) -> String
where
    T: fmt::Display,
{
    let mut res = items
        .into_iter()
        .map(|i| list_name.to_string() + "=" + &*i.to_string() + "&")
        .collect::<String>();
    res.pop();
    res
}

pub async fn post_request<T, U>(
    client: &Client<HttpConnector, Body>,
    path: &str,
    data: T,
    token: Option<&str>,
    addr: &SocketAddr,
) -> Result<U, (StatusCode, String)>
where
    T: Serialize,
    U: for<'a> Deserialize<'a>,
{
    let req_builder = Request::builder()
        .uri(format!("http://{addr}/api{path}"))
        .method("POST")
        .header("Host", "localhost")
        .header("Content-Type", "application/json");
    let req_builder = match token {
        Some(user_token) => req_builder.header("authorization", user_token),
        None => req_builder,
    };
    let req = req_builder
        .body(Body::from(json_bytes(data)))
        .expect("Failed to construct a POST request");
    let res = client
        .request(req)
        .await
        .expect("Failed to make a POST request");
    match res.status() {
        StatusCode::CREATED => {
            let body = res.into_body().collect().await.unwrap().to_bytes();
            Ok(serde_json::from_slice(body.as_ref()).unwrap())
        }
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            Err((status, message))
        }
    }
}

pub async fn get_request<U>(
    client: &Client<HttpConnector, Body>,
    path: &str,
    token: &str,
    addr: &SocketAddr,
) -> Result<U, (StatusCode, String)>
where
    U: for<'a> Deserialize<'a>,
{
    let req = Request::builder()
        .uri(format!("http://{addr}/api{path}"))
        .method("GET")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::empty())
        .expect("Failed to construct a GET request");
    let res = client
        .request(req)
        .await
        .expect("Failed to make a GET request");
    match res.status() {
        StatusCode::OK => {
            let body = res.into_body().collect().await.unwrap().to_bytes();
            Ok(serde_json::from_slice(body.as_ref()).unwrap())
        }
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            Err((status, message))
        }
    }
}

pub async fn put_request<T, U>(
    client: &Client<HttpConnector, Body>,
    path: &str,
    data: T,
    token: &str,
    addr: &SocketAddr,
) -> Result<U, (StatusCode, String)>
where
    T: Serialize,
    U: for<'a> Deserialize<'a>,
{
    let req = Request::builder()
        .uri(format!("http://{addr}/api{path}"))
        .method("PUT")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::from(json_bytes(data)))
        .expect("Failed to construct a PUT request");
    let res = client
        .request(req)
        .await
        .expect("Failed to make a PUT request");
    match res.status() {
        StatusCode::OK => {
            let body = res.into_body().collect().await.unwrap().to_bytes();
            Ok(serde_json::from_slice(body.as_ref()).unwrap())
        }
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            Err((status, message))
        }
    }
}

pub async fn delete_request(
    client: &Client<HttpConnector, Body>,
    path: &str,
    token: &str,
    addr: &SocketAddr,
) -> Result<(), (StatusCode, String)> {
    let req = Request::builder()
        .uri(format!("http://{addr}/api{path}"))
        .method("DELETE")
        .header("Host", "localhost")
        .header("Content-Type", "application/json")
        .header("authorization", token)
        .body(Body::empty())
        .expect("Failed to construct a DELETE request");
    let res = client
        .request(req)
        .await
        .expect("Failed to make a DELETE request");
    match res.status() {
        StatusCode::OK => Ok(()),
        _ => {
            let status = res.status();
            let body = res.into_body().collect().await.unwrap().to_bytes();
            let message: String = read_error_message(body);
            Err((status, message))
        }
    }
}

pub fn read_error_message(body: Bytes) -> String {
    let res: Map<String, Value> = serde_json::from_slice(body.as_ref()).unwrap();
    match res
        .get("msg")
        .expect("Failed to read 'msg' field in an error message")
    {
        Value::String(error_msg) => error_msg.to_owned(),
        _ => panic!("Failed to convert error message from response into a string"),
    }
}
