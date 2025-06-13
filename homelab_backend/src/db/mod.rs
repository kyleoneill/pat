pub mod resource_kinds;

use crate::models::chat::chat_channel::ChatChannel;
use crate::models::games::ConnectionGame;
use crate::models::reminder::Category;
use crate::models::user::user_db::db_create_user;
use crate::models::user::{AuthLevel, User};
use mongodb::error::ErrorKind;
use mongodb::options::IndexOptions;
use mongodb::{bson::doc, Client, Collection, Database, IndexModel};

pub async fn initialize_database_handle(
    connection_string: String,
    database_name: &str,
) -> Database {
    let client = Client::with_uri_str(connection_string)
        .await
        .expect("Failed to connect to database");
    let database = client.database(database_name);

    check_indexes(&database).await;

    // Check for admin, create if not here
    let user_collection: Collection<User> = database.collection("users");
    let user_filter = doc! {"username": "admin"};
    let maybe_admin = user_collection
        .find_one(user_filter)
        .await
        .expect("DB error during initialization when checking if the admin account exists");
    if maybe_admin.is_none() {
        let admin_password: String = dotenv!("ADMIN_PASSWORD_HASH").to_owned();
        let salt: String = dotenv!("ADMIN_SALT").to_owned();
        db_create_user(
            &database,
            "admin".to_owned(),
            admin_password,
            AuthLevel::Admin,
            salt,
        )
        .await
        .expect("DB error during initialization when creating an admin account");
    }
    database
}

pub async fn check_indexes(database: &Database) {
    // TODO: This is also not too sustainable, is there a better way to create indexes?
    check_user_indexes(database).await;
    check_category_indexes(database).await;
    check_connections_game_indexes(database).await;
    check_chat_channel_indexes(database).await;
}

pub async fn check_user_indexes(database: &Database) {
    let user_collection: Collection<User> = database.collection("users");

    match user_collection.list_index_names().await {
        Ok(user_indexes) => {
            if !user_indexes.contains(&"username".to_owned()) {
                create_user_indexes(user_collection).await;
            }
        },
        Err(e) => {
            match *e.kind {
                ErrorKind::Command(command_error) => {
                    match command_error.code {
                        26 => create_user_indexes(user_collection).await,
                        _ => panic!("Failed to list indexes for user collection with unknown error during db initialization")
                    }
                },
                _ => panic!("Failed to list indexes for user collection with unknown error during db initialization")
            }
        }
    }
}

pub async fn create_user_indexes(user_collection: Collection<User>) {
    // Create an index for the username field, which should be unique
    let user_index_options = IndexOptions::builder()
        .unique(true)
        .name(Some("username".to_owned()))
        .build();
    let username_index = IndexModel::builder()
        .keys(doc! {"username": 1})
        .options(user_index_options)
        .build();
    user_collection
        .create_index(username_index)
        .await
        .expect("Failed to create a username index on the users collection");
}

pub async fn check_category_indexes(database: &Database) {
    let category_collection: Collection<Category> = database.collection("categories");

    match category_collection.list_index_names().await {
        Ok(user_indexes) => {
            if !user_indexes.contains(&"slug".to_owned()) {
                create_category_indexes(category_collection).await;
            }
        },
        Err(e) => {
            match *e.kind {
                ErrorKind::Command(command_error) => {
                    match command_error.code {
                        26 => create_category_indexes(category_collection).await,
                        _ => panic!("Failed to list indexes for category collection with unknown error during db initialization")
                    }
                },
                _ => panic!("Failed to list indexes for category collection with unknown error during db initialization")
            }
        }
    }
}

pub async fn create_category_indexes(category_collection: Collection<Category>) {
    // Create an index for the slug field, which should be unique
    let category_index_options = IndexOptions::builder()
        .unique(true)
        .name(Some("slug".to_owned()))
        .build();
    let category_index = IndexModel::builder()
        .keys(doc! {"slug": 1})
        .options(category_index_options)
        .build();
    category_collection
        .create_index(category_index)
        .await
        .expect("Failed to create a slug index on the categories collection");
}

pub async fn check_connections_game_indexes(database: &Database) {
    let game_connections_collection: Collection<ConnectionGame> =
        database.collection("game_connections");

    match game_connections_collection.list_index_names().await {
        Ok(indexes) => {
            if !indexes.contains(&"slug".to_owned()) {
                create_connections_game_indexes(game_connections_collection).await;
            }
        },
        Err(e) => {
            match *e.kind {
                ErrorKind::Command(command_error) => {
                    match command_error.code {
                        26 => create_connections_game_indexes(game_connections_collection).await,
                        _ => panic!("Failed to list indexes for game_connections collection with unknown error during db initialization")
                    }
                },
                _ => panic!("Failed to list indexes for game_connections collection with unknown error during db initialization")
            }
        }
    }
}

pub async fn create_connections_game_indexes(
    game_connections_collection: Collection<ConnectionGame>,
) {
    // Create an index for the slug field, which should be unique
    // TODO: Currently user A making a puzzle with slug "foo" prevents user B from making a puzzle
    //       with the same name. This should probably be a user_id/slug composite index.
    //       Will need to delete the index I have locally when I change this
    let category_index_options = IndexOptions::builder()
        .unique(true)
        .name(Some("slug".to_owned()))
        .build();
    let category_index = IndexModel::builder()
        .keys(doc! {"slug": 1})
        .options(category_index_options)
        .build();
    game_connections_collection
        .create_index(category_index)
        .await
        .expect("Failed to create a slug index on the game_connections collection");
}

pub async fn check_chat_channel_indexes(database: &Database) {
    let chat_channels_collection: Collection<ChatChannel> = database.collection("chat_channels");

    match chat_channels_collection.list_index_names().await {
        Ok(indexes) => {
            if !indexes.contains(&"slug_and_owner_id".to_owned()) {
                create_chat_channels_indexes(chat_channels_collection).await;
            }
        },
        Err(e) => {
            match *e.kind {
                ErrorKind::Command(command_error) => {
                    match command_error.code {
                        26 => create_chat_channels_indexes(chat_channels_collection).await,
                        _ => panic!("Failed to list indexes for chat_channels collection with unknown error during db initialization")
                    }
                },
                _ => panic!("Failed to list indexes for chat_channels collection with unknown error during db initialization")
            }
        }
    }
}

pub async fn create_chat_channels_indexes(chat_channels_collection: Collection<ChatChannel>) {
    // Create a composite index for the slug and owner_id fields, which should be unique
    let category_index_options = IndexOptions::builder()
        .unique(true)
        .name(Some("slug_and_owner_id".to_owned()))
        .build();
    let category_index = IndexModel::builder()
        .keys(doc! {"slug": 1, "owner_id": 1})
        .options(category_index_options)
        .build();
    chat_channels_collection
        .create_index(category_index)
        .await
        .expect("Failed to create a slug_and_owner_id index on the chat_channels collection");
}
