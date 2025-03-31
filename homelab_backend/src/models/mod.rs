use mongodb::bson::Bson;
use serde::{Deserialize, Deserializer};

pub mod games;
pub mod log;
pub mod reminder;
pub mod user;

fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let bson = Bson::deserialize(deserializer)?;
    match bson {
        Bson::ObjectId(value) => Ok(value.to_hex()),
        Bson::String(value) => Ok(value),
        _ => Err(serde::de::Error::custom(
            "Failed to deserialize an ObjectId",
        )),
    }
}

fn name_to_slug(name: &str) -> String {
    // TODO: This should probably return an error if there are zero valid chars
    //       e.g., "name"  is "" or "!@#$"
    let mut out_str = String::new();
    for c in name.chars() {
        let lowered = c.to_ascii_lowercase();
        if lowered.is_ascii() && lowered.is_alphabetic() {
            out_str.push(lowered);
        } else {
            out_str.push('-')
        }
    }
    out_str
}
