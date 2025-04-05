use hyper::body::Bytes;
use serde_json::{value::Value, Map};
use std::fmt;

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
