use crate::{
    db::{MongoModel, PatDatabase, str_to_object_id},
    error_handler::DbError,
    models::user::{AuthLevel, User, validation::UpdateUserSchema},
};
use mongodb::bson::{Bson, Document, doc, oid::ObjectId};

impl MongoModel for User {
    fn collection_name() -> &'static str {
        "users"
    }
    fn model_name() -> &'static str {
        "User"
    }
    fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

pub async fn db_create_user(db_handle: &PatDatabase, username: String, hash: String, auth_level: AuthLevel, salt: String) -> Result<User, DbError> {
    let doc = doc! {
        "username": username,
        "password": hash,
        "auth_level": auth_level,
        "salt": salt
    };
    db_handle.insert_and_retrieve_one(doc).await
}

pub async fn db_update_user(db_handle: &PatDatabase, user: User, update_data: UpdateUserSchema) -> Result<User, DbError> {
    let mut data = Document::new();

    if let Some(username) = update_data.username {
        data.insert("username", username);
    };

    if let Some(password) = update_data.password {
        data.insert("password", password);
    };

    let bson_id: ObjectId = user.mongo_id()?;
    let filter_doc = doc! { "_id": Bson::ObjectId(bson_id) };
    let update_doc = doc! { "$set": data };
    db_handle.find_and_update_one(filter_doc, update_doc).await
}

pub async fn db_get_user_by_username(db_handle: &PatDatabase, username: &str) -> Result<User, DbError> {
    let doc = doc! { "username": username };
    db_handle.find_one(doc).await
}

pub async fn db_delete_user(db_handle: &PatDatabase, user_id: String) -> Result<(), DbError> {
    let bson_id: ObjectId = match user_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };
    db_handle.delete_one::<User>(doc).await
}

pub async fn db_get_user_by_id(db_handle: &PatDatabase, id: &str) -> Result<User, DbError> {
    let user_id = str_to_object_id(id)?;
    let doc = doc! { "_id": Bson::ObjectId(user_id) };
    db_handle.find_one(doc).await
}
