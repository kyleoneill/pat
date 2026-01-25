use crate::error_handler::DbError;
use mongodb::{
    bson::{oid::ObjectId, Document},
    error::Error,
    Collection, Database,
};
use serde::de::DeserializeOwned;

pub mod db_setup;

pub fn str_to_object_id(object_str: &str) -> Result<ObjectId, Error> {
    match ObjectId::parse_str(object_str) {
        Ok(object_id) => Ok(object_id),
        Err(_) => Err(Error::custom("Failed to construct ObjectId")),
    }
}

pub trait MongoModel {
    fn collection_name() -> &'static str;
    fn model_name() -> &'static str;
}

pub struct PatDatabase {
    pool: Database,
}

impl PatDatabase {
    pub fn new(database: Database) -> Self {
        Self { pool: database }
    }

    pub fn pool_ref(&self) -> &Database {
        &self.pool
    }

    pub fn get_type_agnostic_collection(&self, collection_name: &str) -> Collection<Document> {
        self.pool.collection(collection_name)
    }

    pub fn get_collection<T>(&self) -> Collection<T>
    where
        T: MongoModel + Send + Sync,
    {
        self.pool.collection(T::collection_name())
    }

    pub async fn find_one<T>(&self, filter_doc: Document) -> Result<T, DbError>
    where
        T: MongoModel + Send + Sync + DeserializeOwned,
    {
        let collection: Collection<T> = self.pool.collection(T::collection_name());
        match collection.find_one(filter_doc).await {
            Ok(maybe_record) => match maybe_record {
                Some(record) => Ok(record),
                None => Err(DbError::NotFound(T::model_name())),
            },
            Err(e) => Err(e.into()),
        }
    }
}
