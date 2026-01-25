use super::{validation::CreateConnectionGameSchema, ConnectionGame};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    error::ErrorKind,
    Collection,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    db::{MongoModel, PatDatabase},
    error_handler::DbError,
    models::name_to_slug,
};

pub async fn insert_connections_game(db_handle: &PatDatabase, data: &CreateConnectionGameSchema, user_id: String) -> Result<ConnectionGame, DbError> {
    let collection: Collection<Document> = db_handle.get_type_agnostic_collection(ConnectionGame::collection_name());
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
    match collection.insert_one(doc).await {
        Ok(_res) => (),
        Err(e) => {
            return match *e.kind {
                ErrorKind::Write(_) => Err(DbError::AlreadyExists(ConnectionGame::model_name(), slug)),
                _ => Err(e.into()),
            }
        }
    }
    get_connection_game_by_slug(db_handle, slug.as_str()).await
}

pub async fn get_connection_game_by_slug(db_handle: &PatDatabase, slug: &str) -> Result<ConnectionGame, DbError> {
    let doc = doc! { "slug": slug };
    db_handle.find_one(doc).await
}

pub async fn get_all_connections_games(db_handle: &PatDatabase, user_id: &str, this_users_games: bool) -> Result<Vec<ConnectionGame>, DbError> {
    // TODO: This should take a cursor and be paginated
    let collection: Collection<ConnectionGame> = db_handle.get_collection();
    let doc = match this_users_games {
        true => doc! { "author_id": user_id },
        false => doc! { "author_id": { "$ne": user_id } },
    };
    match collection.find(doc).await {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
    }
}
