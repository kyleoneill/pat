use crate::{
    db::{str_to_object_id, MongoModel, PatDatabase},
    error_handler::DbError,
    models::user::{validation::UpdateUserSchema, AuthLevel, User},
};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    Collection,
};

pub async fn db_create_user(db_handle: &PatDatabase, username: String, hash: String, auth_level: AuthLevel, salt: String) -> Result<User, DbError> {
    let collection: Collection<Document> = db_handle.get_type_agnostic_collection(User::collection_name());
    let copied_username = username.clone();
    let doc = doc! {
        "username": username,
        "password": hash,
        "auth_level": auth_level,
        "salt": salt
    };
    match collection.insert_one(doc).await {
        Ok(_res) => (),
        Err(e) => return Err(e.into()),
    };
    db_get_user_by_username(db_handle, copied_username.as_str()).await
}

pub async fn db_update_user(db_handle: &PatDatabase, user: User, update_data: UpdateUserSchema) -> Result<User, DbError> {
    let user_collection: Collection<User> = db_handle.get_collection();
    let mut data = Document::new();

    if let Some(username) = update_data.username {
        data.insert("username", username);
    };

    if let Some(password) = update_data.password {
        data.insert("password", password);
    };

    let bson_id: ObjectId = match user.id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let filter_doc = doc! { "_id": Bson::ObjectId(bson_id) };
    let update_doc = doc! { "$set": data };
    match user_collection.update_one(filter_doc, update_doc).await {
        Ok(_update_res) => (),
        Err(e) => return Err(e.into()),
    }

    db_get_user_by_id(db_handle, user.id.as_str()).await
}

pub async fn db_get_user_by_username(db_handle: &PatDatabase, username: &str) -> Result<User, DbError> {
    let doc = doc! { "username": username };
    db_handle.find_one(doc).await
}

pub async fn db_delete_user(db_handle: &PatDatabase, user_id: String) -> Result<(), DbError> {
    let user_collection: Collection<User> = db_handle.get_collection();
    let bson_id: ObjectId = match user_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };
    match user_collection.delete_one(doc).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn db_get_user_by_id(db_handle: &PatDatabase, id: &str) -> Result<User, DbError> {
    let user_id = str_to_object_id(id)?;
    let doc = doc! { "_id": Bson::ObjectId(user_id) };
    db_handle.find_one(doc).await
}
