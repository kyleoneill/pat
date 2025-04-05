use super::{ConnectionGame, ConnectionGameSchema};
use crate::db::resource_kinds::ResourceKind;
use crate::error_handler::DbError;
use crate::models::name_to_slug;
use futures::TryStreamExt;
use mongodb::error::ErrorKind;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn insert_connections_game(
    pool: &Database,
    data: &ConnectionGameSchema,
    user_id: String,
) -> Result<ConnectionGame, DbError> {
    let collection: Collection<Document> = pool.collection("game_connections");
    let slug = name_to_slug(data.puzzle_name.as_str());
    let date_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

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
                ErrorKind::Write(_) => {
                    Err(DbError::AlreadyExists(ResourceKind::ConnectionsGame, slug))
                }
                _ => Err(e.into()),
            }
        }
    }
    get_connection_game_by_slug(pool, slug.as_str()).await
}

pub async fn get_connection_game_by_slug(
    pool: &Database,
    slug: &str,
) -> Result<ConnectionGame, DbError> {
    let collection: Collection<ConnectionGame> = pool.collection("game_connections");
    let doc = doc! { "slug": slug };
    match collection.find_one(doc).await {
        Ok(maybe_record) => match maybe_record {
            Some(category) => Ok(category),
            None => Err(DbError::NotFound(
                ResourceKind::ConnectionsGame,
                slug.to_owned(),
            )),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn get_all_connections_games(
    pool: &Database,
    user_id: &str,
    this_users_games: bool,
) -> Result<Vec<ConnectionGame>, DbError> {
    // TODO: This should take a cursor and be paginated
    let collection: Collection<ConnectionGame> = pool.collection("game_connections");
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
