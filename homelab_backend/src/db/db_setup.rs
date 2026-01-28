use mongodb::{bson::doc, options::IndexOptions, Client, Collection, Database, IndexModel};

use crate::{
    db::PatDatabase,
    models::{
        chat::{chat_channel::ChatChannel, message::ChatMessage},
        games::ConnectionGame,
        reminder::Category,
        user::{user_db::db_create_user, AuthLevel, User},
    },
};

pub async fn initialize_database_handle(connection_string: String, database_name: &str) -> Database {
    let client = Client::with_uri_str(connection_string).await.expect("Failed to connect to database");
    let database = client.database(database_name);

    let db_handle = PatDatabase::new(database.clone());

    check_indexes(&db_handle).await;

    if cfg!(debug_assertions) {
        // If in debug mode, check for admin, create if not here
        let user_collection: Collection<User> = db_handle.get_collection();
        let user_filter = doc! {"username": "admin"};
        let maybe_admin = user_collection
            .find_one(user_filter)
            .await
            .expect("DB error during initialization when checking if the admin account exists");
        if maybe_admin.is_none() {
            let admin_password: String = dotenv!("ADMIN_PASSWORD_HASH").to_owned();
            let salt: String = dotenv!("ADMIN_SALT").to_owned();

            db_create_user(&db_handle, "admin".to_owned(), admin_password, AuthLevel::Admin, salt)
                .await
                .expect("DB error during initialization when creating an admin account");
        }
    }
    database
}

pub async fn check_indexes(db_handle: &PatDatabase) {
    // TODO: This is also not too sustainable, is there a better way to create indexes?
    // User
    create_user_indexes(db_handle).await;

    // Game
    create_category_indexes(db_handle).await;
    create_connections_game_indexes(db_handle).await;

    // Chat
    create_chat_channels_indexes(db_handle).await;
    create_chat_message_indexes(db_handle).await;
}

pub async fn create_user_indexes(db_handle: &PatDatabase) {
    let user_collection: Collection<User> = db_handle.get_collection();

    // username index, unique on username
    let user_index_options = IndexOptions::builder().unique(true).name(Some("username".to_owned())).build();
    let username_index = IndexModel::builder().keys(doc! {"username": 1}).options(user_index_options).build();
    user_collection
        .create_index(username_index)
        .await
        .expect("Failed to create a username index on the users collection");
}

pub async fn create_category_indexes(db_handle: &PatDatabase) {
    let category_collection: Collection<Category> = db_handle.get_collection();

    // slug index, unique on slug
    let category_index_options = IndexOptions::builder().unique(true).name(Some("slug".to_owned())).build();
    let category_index = IndexModel::builder().keys(doc! {"slug": 1}).options(category_index_options).build();
    category_collection
        .create_index(category_index)
        .await
        .expect("Failed to create a slug index on the categories collection");
}

pub async fn create_connections_game_indexes(db_handle: &PatDatabase) {
    let game_connections_collection: Collection<ConnectionGame> = db_handle.get_collection();

    // Slug index, unique on slug and author_id
    let category_index_options = IndexOptions::builder().unique(true).name(Some("slug_and_author".to_owned())).build();
    let category_index = IndexModel::builder()
        .keys(doc! {"slug": 1, "author_id": 1})
        .options(category_index_options)
        .build();
    game_connections_collection
        .create_index(category_index)
        .await
        .expect("Failed to create a slug_and_author index on the game_connections collection");
}

pub async fn create_chat_channels_indexes(db_handle: &PatDatabase) {
    let chat_channels_collection: Collection<ChatChannel> = db_handle.get_collection();

    // Slug index, unique on slug and owner_id
    let category_index_options = IndexOptions::builder().unique(true).name(Some("slug_and_owner_id".to_owned())).build();
    let category_index = IndexModel::builder()
        .keys(doc! {"slug": 1, "owner_id": 1})
        .options(category_index_options)
        .build();
    chat_channels_collection
        .create_index(category_index)
        .await
        .expect("Failed to create a slug_and_owner_id index on the chat_channels collection");
}

pub async fn create_chat_message_indexes(db_handle: &PatDatabase) {
    let chat_messages_collection: Collection<ChatMessage> = db_handle.get_collection();

    let category_index_options = IndexOptions::builder()
        .unique(true)
        .name(Some("channel_and_atomic_ids".to_owned()))
        .build();
    // atomic_id ordered by -1 which makes it descending. This will put the largest number (the most
    // recent message) first
    let category_index = IndexModel::builder()
        .keys(doc! {"channel_id": 1, "atomic_id": -1})
        .options(category_index_options)
        .build();
    chat_messages_collection
        .create_index(category_index)
        .await
        .expect("Failed to create a channel_and_atomic_ids index on the chat_messages collection");
}
