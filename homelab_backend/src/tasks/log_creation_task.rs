use crate::db::PatDatabase;
use crate::models::log::Log;
use mongodb::bson::doc;
use std::fmt::{Display, Formatter};

pub struct LogCreationTask {
    method: String,
    uri: String,
    user_id: String,
    date_time: i64,
}

impl Display for LogCreationTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} {} from {}", self.date_time, self.method, self.uri, self.user_id)
    }
}

impl LogCreationTask {
    pub fn new(method: String, uri: String, user_id: String, date_time: i64) -> Self {
        Self {
            method,
            uri,
            user_id,
            date_time,
        }
    }
    pub async fn write_log(&self, db_handle: &PatDatabase) {
        let mut uri = self.uri.clone();
        if uri.starts_with("/api/chat/ws?auth_token=") {
            // This is a bit hacky and should be made to support a general-purpose solution to the problem
            // Websocket authentication includes the token as a query param in the uri, so redact it
            uri = "/api/chat/ws?auth_token=<redacted>".to_string()
        }
        let doc = doc! {
            "method": self.method.clone(),
            "uri": uri,
            "user_id": self.user_id.clone(),
            "date_time": self.date_time
        };
        let _res = db_handle.insert_one::<Log>(doc).await;
    }
}
