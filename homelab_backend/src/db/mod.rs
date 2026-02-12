use crate::error_handler::DbError;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    error::Error,
    options::FindOneAndUpdateOptions,
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
    fn mongo_id(&self) -> Result<ObjectId, DbError>;
}

// Derives clone so tasks can copy a handle to the pool to be moved into an async block, cloning
// this is fine as Database is an Arc<_>
#[derive(Clone)]
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
        // Inserting documents uses a Collection<Document> rather than a Collection<T> since it
        // uses an insert schema rather than a T, as a T includes an _id field which does not
        // exist until after the data has been inserted. Is there a way to resolve this without
        // using a Collection<Document>?
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

    pub async fn find<T>(&self, filter_doc: Document) -> Result<Vec<T>, DbError>
    where
        T: MongoModel + Send + Sync + DeserializeOwned,
    {
        // TODO: Should support a sort here, then replace the collection.find() call being made
        //       in message_db.rs
        let collection: Collection<T> = self.pool.collection(T::collection_name());
        match collection.find(filter_doc).await {
            Ok(cursor) => match cursor.try_collect().await {
                Ok(records) => Ok(records),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }
    }

    pub async fn insert_one<T>(&self, insertion_data: Document) -> Result<ObjectId, DbError>
    where
        T: MongoModel + Send + Sync + DeserializeOwned,
    {
        let collection: Collection<Document> = self.pool.collection(T::collection_name());
        match collection.insert_one(insertion_data).await {
            Ok(res) => match res.inserted_id {
                Bson::ObjectId(id) => Ok(id),
                _ => Err(DbError::UnhandledException("Insertion failed to produce an ID".to_owned())),
            },
            Err(e) => Err(e.into()),
        }
    }

    pub async fn insert_and_retrieve_one<T>(&self, insertion_data: Document) -> Result<T, DbError>
    where
        T: MongoModel + Send + Sync + DeserializeOwned,
    {
        let object_id = self.insert_one::<T>(insertion_data).await?;
        let doc = doc! {"_id": object_id};
        self.find_one(doc).await
    }

    #[allow(dead_code)]
    pub async fn update_one<T>(&self, filter_doc: Document, update_doc: Document) -> Result<u64, DbError>
    where
        T: MongoModel + Send + Sync + DeserializeOwned,
    {
        let collection: Collection<T> = self.pool.collection(T::collection_name());
        match collection.update_one(filter_doc, update_doc).await {
            Ok(update_res) => {
                if update_res.matched_count == 0 {
                    return Err(DbError::NotFound(T::model_name()));
                }
                Ok(update_res.modified_count)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn find_and_update_one<T>(&self, filter_doc: Document, update_doc: Document) -> Result<T, DbError>
    where
        T: MongoModel + Send + Sync + DeserializeOwned,
    {
        let collection: Collection<T> = self.pool.collection(T::collection_name());

        // find_one_and_update acts like an atomic operation and by default "swaps" the new and old
        // documents, meaning that the default behavior is to return the document _before_ it's
        // updated. This isn't what's desired for this helper, so use an option to get
        // the document _after_ the update
        let update_options = FindOneAndUpdateOptions::builder()
            .return_document(Some(mongodb::options::ReturnDocument::After))
            .build();
        match collection
            .find_one_and_update(filter_doc, update_doc)
            .with_options(Some(update_options))
            .await
        {
            Ok(update_res) => match update_res {
                Some(res) => Ok(res),
                None => Err(DbError::NotFound(T::model_name())),
            },
            Err(e) => Err(e.into()),
        }
    }

    pub async fn delete_one<T>(&self, filter_doc: Document) -> Result<(), DbError>
    where
        T: MongoModel + Send + Sync + DeserializeOwned,
    {
        let collection: Collection<T> = self.pool.collection(T::collection_name());
        match collection.delete_one(filter_doc).await {
            Ok(delete_res) => {
                if delete_res.deleted_count == 0 {
                    return Err(DbError::NotFound(T::model_name()));
                }
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}
