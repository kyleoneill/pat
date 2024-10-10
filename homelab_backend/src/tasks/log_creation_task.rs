use mongodb::bson::{doc, Document};
use mongodb::{Collection, Database};

pub struct LogCreationTask {
    method: String,
    uri: String,
    user_id: String,
    date_time: i64,
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
        let doc = doc! {
            "method": self.method.clone(),
            "uri": self.uri.clone(),
            "user_id": self.user_id.clone(),
            "date_time": self.date_time
        };
        let _res = collection.insert_one(doc).await;
    }
}
