use mongodb::bson::{doc, Document};
use mongodb::{Collection, Database};
use std::fmt::{Display, Formatter};

pub struct LogCreationTask {
    method: String,
    uri: String,
    user_id: String,
    date_time: i64,
}

impl Display for LogCreationTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} {} from {}",
            self.date_time, self.method, self.uri, self.user_id
        )
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
    pub async fn run_task(&self, database: &Database) {
        let collection: Collection<Document> = database.collection("logs");
        let mut uri = self.uri.clone();
        if uri.starts_with("/api/chat/ws?auth_token=") {
            // This is a bit hacky and should be made to support a general-purpose solution to the problem
            uri = "/api/chat/ws?auth_token=<redacted>".to_string()
        }
        let doc = doc! {
            "method": self.method.clone(),
            "uri": uri,
            "user_id": self.user_id.clone(),
            "date_time": self.date_time
        };
        let _res = collection.insert_one(doc).await;
    }
}
