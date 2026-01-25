use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    error::ErrorKind,
    Collection,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    db::{str_to_object_id, MongoModel, PatDatabase},
    error_handler::DbError,
    models::reminder::{
        validation::{CreateCategorySchema, CreateReminderSchema, UpdateReminderSchema},
        Category, Reminder,
    },
};

// Categories
pub async fn insert_category(db_handle: &PatDatabase, data: &CreateCategorySchema, user_id: String) -> Result<Category, DbError> {
    let collection: Collection<Document> = db_handle.get_type_agnostic_collection(Category::collection_name());
    let doc = doc! {
        "slug": data.slug.clone(),
        "name": data.name.clone(),
        "user_id": user_id
    };
    match collection.insert_one(doc).await {
        Ok(_res) => (),
        Err(e) => {
            return match *e.kind {
                ErrorKind::Write(_) => Err(DbError::AlreadyExists(Category::model_name(), data.slug.clone())),
                _ => Err(e.into()),
            }
        }
    }
    get_category_by_slug(db_handle, &data.slug).await
}

pub async fn get_categories_for_user(db_handle: &PatDatabase, user_id: String) -> Result<Vec<Category>, DbError> {
    let collection: Collection<Category> = db_handle.get_collection();
    let doc = doc! { "user_id": user_id };
    match collection.find(doc).await {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
    }
}

pub async fn get_category_by_slug(db_handle: &PatDatabase, slug: &str) -> Result<Category, DbError> {
    let doc = doc! { "slug": slug };
    db_handle.find_one(doc).await
}

pub async fn delete_category_by_id(db_handle: &PatDatabase, category_id: String, user_id: String) -> Result<u64, DbError> {
    // Verify that the category being deleted is not in use by a reminder
    let reminder_collection: Collection<Reminder> = db_handle.get_collection();
    let doc = doc! { "categories": category_id.clone() };

    // Could also use .count() here and check if the result is greater than 1, but getting a cursor
    // and only checking one record might be more efficient
    match reminder_collection.find(doc).await {
        Ok(mut cursor) => {
            if cursor.next().await.is_some() {
                return Err(DbError::RelationshipViolation(Category::model_name(), category_id.clone()));
            }
        }
        Err(e) => return Err(e.into()),
    }

    let category_collection: Collection<Category> = db_handle.get_collection();

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
pub async fn insert_reminder(db_handle: &PatDatabase, data: &CreateReminderSchema, user_id: String) -> Result<Reminder, DbError> {
    let serialized_priority = data.priority as i64;
    let date_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    // TODO: Verify that all category IDs exist

    let reminder_collection: Collection<Document> = db_handle.get_type_agnostic_collection(Reminder::collection_name());
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
            _ => return Err(DbError::UnhandledException("Failed to insert a new reminder".to_owned())),
        },
        Err(e) => return Err(e.into()),
    };

    get_reminder_by_id(db_handle, new_doc_id.as_str()).await
}

pub async fn get_reminder_by_id(db_handle: &PatDatabase, id: &str) -> Result<Reminder, DbError> {
    let bson_id: ObjectId = str_to_object_id(id)?;
    let doc = doc! { "_id": Bson::ObjectId(bson_id) };
    db_handle.find_one(doc).await
}

pub async fn get_reminders_for_user(
    db_handle: &PatDatabase,
    user_id: String,
    maybe_categories: Option<Vec<String>>,
) -> Result<Vec<Reminder>, DbError> {
    let reminder_collection: Collection<Reminder> = db_handle.get_collection();
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

pub async fn db_update_reminder(db_handle: &PatDatabase, reminder_id: String, updates: UpdateReminderSchema) -> Result<Reminder, DbError> {
    let reminder_collection: Collection<Reminder> = db_handle.get_collection();
    let mut doc = Document::new();

    // TODO: This is going to be very cumbersome for update schemas with many fields. I should
    //       make a trait or macro where I can just do schema_to_doc!()
    if let Some(name) = updates.name {
        doc.insert("name", name);
    };

    if let Some(description) = updates.description {
        doc.insert("description", description);
    };

    if let Some(priority) = updates.priority {
        let serialized_priority: i64 = priority.into();
        doc.insert("priority", serialized_priority);
    };

    if let Some(categories) = updates.categories {
        doc.insert("categories", categories);
    };

    let bson_id: ObjectId = match reminder_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let filter_doc = doc! { "_id": Bson::ObjectId(bson_id) };

    if doc.is_empty() {
        return Err(DbError::EmptyDbExpression(Reminder::model_name(), "updating".to_owned()));
    }

    let update_doc = doc! { "$set": doc};
    match reminder_collection.update_one(filter_doc, update_doc).await {
        Ok(_update_res) => (),
        Err(e) => return Err(e.into()),
    };

    get_reminder_by_id(db_handle, reminder_id.as_str()).await
}

pub async fn db_delete_reminder(db_handle: &PatDatabase, reminder_id: String, user_id: String) -> Result<u64, DbError> {
    let reminder_collection: Collection<Reminder> = db_handle.get_collection();
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
