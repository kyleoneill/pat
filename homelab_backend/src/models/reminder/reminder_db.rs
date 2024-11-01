use super::{Category, CategorySchema, Reminder, ReminderSchema, ReminderUpdateSchema};
use crate::error_handler::DbError;
use futures::{StreamExt, TryStreamExt};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Bson;
use mongodb::error::ErrorKind;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Deserialize)]
struct ReminderCategories {
    reminder_id: i64,
    category_id: i64,
}

// Categories
pub async fn insert_category(
    pool: &Database,
    data: &CategorySchema,
    user_id: String,
) -> Result<Category, DbError> {
    let collection: Collection<Document> = pool.collection("categories");
    let doc = doc! {
        "slug": data.slug.clone(),
        "name": data.name.clone(),
        "user_id": user_id
    };
    match collection.insert_one(doc).await {
        Ok(_res) => (),
        Err(e) => {
            return match *e.kind {
                ErrorKind::Write(_) => Err(DbError::AlreadyExists(
                    "category".to_owned(),
                    data.slug.clone(),
                )),
                _ => Err(e.into()),
            }
        }
    }
    get_category_by_slug(pool, &data.slug).await
}

pub async fn get_categories_for_user(
    pool: &Database,
    user_id: String,
) -> Result<Vec<Category>, DbError> {
    let collection: Collection<Category> = pool.collection("categories");
    let doc = doc! { "user_id": user_id };
    match collection.find(doc).await {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn get_category_by_slug(pool: &Database, slug: &str) -> Result<Category, DbError> {
    let collection: Collection<Category> = pool.collection("categories");
    let doc = doc! { "slug": slug };
    match collection.find_one(doc).await {
        Ok(maybe_record) => match maybe_record {
            Some(category) => Ok(category),
            None => Err(DbError::NotFound("category".to_owned(), slug.to_owned())),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn delete_category_by_id(
    pool: &Database,
    category_id: String,
    user_id: String,
) -> Result<u64, DbError> {
    // Verify that the category being deleted is not in use by a reminder
    let reminder_collection: Collection<Reminder> = pool.collection("reminders");
    let doc = doc! { "categories": category_id.clone() };

    // Could also use .count() here and check if the result is greater than 1, but getting a cursor
    // and only checking one record might be more efficient
    match reminder_collection.find(doc).await {
        Ok(mut cursor) => {
            if cursor.next().await.is_some() {
                return Err(DbError::RelationshipViolation(
                    "category".to_owned(),
                    category_id.clone(),
                ));
            }
        }
        Err(e) => return Err(e.into()),
    }

    let category_collection: Collection<Category> = pool.collection("categories");

    let bson_id: ObjectId = match category_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! { "_id": Bson::ObjectId(bson_id), "user_id": user_id };
    match category_collection.delete_one(doc).await {
        Ok(delete_result) => Ok(delete_result.deleted_count),
        Err(e) => Err(e.into()),
    }
}

// Reminders
pub async fn insert_reminder(
    pool: &Database,
    data: &ReminderSchema,
    user_id: String,
) -> Result<Reminder, DbError> {
    let serialized_priority = data.priority as i64;
    let date_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // TODO: Verify that all category IDs exist

    let reminder_collection: Collection<Document> = pool.collection("reminders");
    let doc = doc! {
        "name": data.name.clone(),
        "description": data.description.clone(),
        "priority": serialized_priority,
        "user_id": user_id,
        "date_time": date_time,
        "categories": data.categories.clone()
    };

    let new_doc_id = match reminder_collection.insert_one(doc).await {
        Ok(res) => match res.inserted_id {
            Bson::ObjectId(id) => id.to_string(),
            _ => return Err(DbError::UnhandledException),
        },
        Err(e) => return Err(e.into()),
    };

    get_reminder_by_id(pool, new_doc_id).await
}

pub async fn get_reminder_by_id(pool: &Database, id: String) -> Result<Reminder, DbError> {
    let reminder_collection: Collection<Reminder> = pool.collection("reminders");
    let bson_id: ObjectId = match id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };

    match reminder_collection.find_one(doc).await {
        Ok(maybe_doc) => match maybe_doc {
            Some(doc) => Ok(doc),
            None => Err(DbError::NotFound("reminder".to_owned(), id.to_string())),
        },
        Err(e) => {
            println!("{:?}", e);
            Err(e.into())
        }
    }
}

pub async fn get_reminders_for_user(
    pool: &Database,
    user_id: String,
    maybe_categories: Option<Vec<String>>,
) -> Result<Vec<Reminder>, DbError> {
    let reminder_collection: Collection<Reminder> = pool.collection("reminders");
    let mut doc = doc! {"user_id": user_id};

    if maybe_categories.is_some() {
        let categories = maybe_categories.unwrap();
        doc.insert("categories", doc! {"$in": categories});
    }

    match reminder_collection.find(doc).await {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn db_update_reminder(
    pool: &Database,
    reminder_id: String,
    updates: ReminderUpdateSchema,
) -> Result<Reminder, DbError> {
    let reminder_collection: Collection<Reminder> = pool.collection("reminders");
    let mut doc = Document::new();

    if let Some(name) = &updates.name {
        doc.insert("name", name.to_owned());
    };

    if let Some(description) = &updates.description {
        doc.insert("description", description.to_owned());
    };

    if let Some(priority) = &updates.priority {
        let serialized_priority: i64 = priority.to_owned().into();
        doc.insert("priority", serialized_priority);
    };

    if let Some(categories) = &updates.categories {
        doc.insert("categories", categories.to_owned());
    };

    let bson_id: ObjectId = match reminder_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let filter_doc = doc! { "_id": Bson::ObjectId(bson_id) };

    if doc.is_empty() {
        return Err(DbError::EmptyDbExpression(
            "updating".to_owned(),
            "reminder".to_owned(),
        ));
    }

    let update_doc = doc! { "$set": doc};
    match reminder_collection.update_one(filter_doc, update_doc).await {
        Ok(_update_res) => (),
        Err(e) => return Err(e.into()),
    };

    get_reminder_by_id(pool, reminder_id).await
}

pub async fn db_delete_reminder(
    pool: &Database,
    reminder_id: String,
    user_id: String,
) -> Result<u64, DbError> {
    let reminder_collection: Collection<Reminder> = pool.collection("reminders");
    let reminder_bson_id: ObjectId = match reminder_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! {"_id": Bson::ObjectId(reminder_bson_id), "user_id": Bson::String(user_id)};

    match reminder_collection.delete_one(doc).await {
        Ok(res) => Ok(res.deleted_count),
        Err(e) => Err(e.into()),
    }
}
