use super::{AuthLevel, User};
use crate::db::resource_kinds::ResourceKind;
use crate::error_handler::DbError;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Bson;
use mongodb::bson::{doc, Document};
use mongodb::{Collection, Database};

pub async fn db_create_user(
    pool: &Database,
    username: String,
    hash: String,
    auth_level: AuthLevel,
    salt: String,
) -> Result<(), DbError> {
    let collection: Collection<Document> = pool.collection("users");
    let doc = doc! {
        "username": username,
        "password": hash,
        "auth_level": auth_level,
        "salt": salt
    };
    match collection.insert_one(doc).await {
        Ok(_res) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn db_get_user_by_username(pool: &Database, username: &str) -> Result<User, DbError> {
    let collection: Collection<User> = pool.collection("users");
    let doc = doc! { "username": username };
    match collection.find_one(doc).await {
        Ok(maybe_record) => match maybe_record {
            Some(user) => Ok(user),
            None => Err(DbError::NotFound(ResourceKind::User, username.to_owned())),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn db_delete_user(pool: &Database, user_id: String) -> Result<(), DbError> {
    let collection: Collection<User> = pool.collection("users");
    let bson_id: ObjectId = match user_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };
    match collection.delete_one(doc).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn db_get_user_by_id(pool: &Database, id: String) -> Result<User, DbError> {
    let collection: Collection<User> = pool.collection("users");
    let bson_id: ObjectId = match id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };
    match collection.find_one(doc).await {
        Ok(maybe_record) => match maybe_record {
            Some(user) => Ok(user),
            None => Err(DbError::NotFound(ResourceKind::User, id.to_string())),
        },
        Err(e) => Err(e.into()),
    }
}
