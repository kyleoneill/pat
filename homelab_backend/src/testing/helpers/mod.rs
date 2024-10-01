use std::fmt;

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
