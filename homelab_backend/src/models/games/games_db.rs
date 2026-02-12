use super::{validation::CreateConnectionGameSchema, ConnectionGame};
use crate::{
    db::{MongoModel, PatDatabase},
    error_handler::DbError,
    models::name_to_slug,
};
use mongodb::bson::{doc, oid::ObjectId};
use std::time::{SystemTime, UNIX_EPOCH};

impl MongoModel for ConnectionGame {
    fn collection_name() -> &'static str {
        "game_connections"
    }
    fn model_name() -> &'static str {
        "Connections Game"
    }
    fn mongo_id(&self) -> Result<ObjectId, DbError> {
        match self.id.parse::<ObjectId>() {
            Ok(res) => Ok(res),
            Err(_) => Err(DbError::BadId),
        }
    }
}

pub async fn insert_connections_game(db_handle: &PatDatabase, data: &CreateConnectionGameSchema, user_id: String) -> Result<ConnectionGame, DbError> {
    let slug = name_to_slug(data.puzzle_name.as_str());
    let date_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    let doc = doc! {
        "connection_categories": [
            data.connection_categories[0].to_doc(),
            data.connection_categories[1].to_doc(),
            data.connection_categories[2].to_doc(),
            data.connection_categories[3].to_doc(),
        ],
        "puzzle_name": data.puzzle_name.clone(),
        "slug": slug.clone(),
        "author_id": user_id,
        "creation_datetime": date_time,
    };
    db_handle.insert_and_retrieve_one(doc).await
}

pub async fn get_connection_game_by_slug(db_handle: &PatDatabase, slug: &str) -> Result<ConnectionGame, DbError> {
    let doc = doc! { "slug": slug };
    db_handle.find_one(doc).await
}

pub async fn get_all_connections_games(db_handle: &PatDatabase, user_id: &str, this_users_games: bool) -> Result<Vec<ConnectionGame>, DbError> {
    let doc = match this_users_games {
        true => doc! { "author_id": user_id },
        false => doc! { "author_id": { "$ne": user_id } },
    };

    db_handle.find(doc).await
}
