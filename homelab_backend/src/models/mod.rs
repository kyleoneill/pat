use mongodb::bson::Bson;
use serde::{Deserialize, Deserializer};

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
