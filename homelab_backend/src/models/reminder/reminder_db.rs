use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    Collection,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    db::{MongoModel, PatDatabase},
    error_handler::DbError,
    models::reminder::{
        validation::{CreateCategorySchema, CreateReminderSchema, UpdateReminderSchema},
        Category, Reminder,
    },
};

impl MongoModel for Reminder {
    fn collection_name() -> &'static str {
        "reminders"
    }
    fn model_name() -> &'static str {
        "Reminder"
    }
    fn mongo_id(&self) -> Result<ObjectId, DbError> {
        match self.id.parse::<ObjectId>() {
            Ok(res) => Ok(res),
            Err(_) => Err(DbError::BadId),
        }
    }
}

impl MongoModel for Category {
    fn collection_name() -> &'static str {
        "categories"
    }
    fn model_name() -> &'static str {
        "Reminder Category"
    }
    fn mongo_id(&self) -> Result<ObjectId, DbError> {
        match self.id.parse::<ObjectId>() {
            Ok(res) => Ok(res),
            Err(_) => Err(DbError::BadId),
        }
    }
}

// Categories
pub async fn insert_category(db_handle: &PatDatabase, data: &CreateCategorySchema, user_id: String) -> Result<Category, DbError> {
    let doc = doc! {
        "slug": data.slug.clone(),
        "name": data.name.clone(),
        "user_id": user_id
    };
    db_handle.insert_and_retrieve_one(doc).await
}

pub async fn get_categories_for_user(db_handle: &PatDatabase, user_id: String) -> Result<Vec<Category>, DbError> {
    let doc = doc! { "user_id": user_id };
    db_handle.find(doc).await
}

// pub async fn get_category_by_slug(db_handle: &PatDatabase, slug: &str) -> Result<Category, DbError> {
//     let doc = doc! { "slug": slug };
//     db_handle.find_one(doc).await
// }

pub async fn delete_category_by_id(db_handle: &PatDatabase, category_id: String, user_id: String) -> Result<(), DbError> {
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

    let bson_id: ObjectId = match category_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! { "_id": Bson::ObjectId(bson_id), "user_id": user_id };
    db_handle.delete_one::<Category>(doc).await
}

// Reminders
pub async fn insert_reminder(db_handle: &PatDatabase, data: &CreateReminderSchema, user_id: String) -> Result<Reminder, DbError> {
    let date_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    // TODO: Verify that all category IDs exist

    let doc = doc! {
        "name": data.name.clone(),
        "description": data.description.clone(),
        "priority": data.priority as i64,
        "user_id": user_id,
        "date_time": date_time,
        "categories": data.categories.clone()
    };

    db_handle.insert_and_retrieve_one(doc).await
}

pub async fn get_reminders_for_user(
    db_handle: &PatDatabase,
    user_id: String,
    maybe_categories: Option<Vec<String>>,
) -> Result<Vec<Reminder>, DbError> {
    let mut doc = doc! {"user_id": user_id};

    if maybe_categories.is_some() {
        let categories = maybe_categories.unwrap();
        doc.insert("categories", doc! {"$in": categories});
    }

    db_handle.find(doc).await
}

pub async fn db_update_reminder(db_handle: &PatDatabase, reminder_id: String, updates: UpdateReminderSchema) -> Result<Reminder, DbError> {
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
    db_handle.find_and_update_one(filter_doc, update_doc).await
}

pub async fn db_delete_reminder(db_handle: &PatDatabase, reminder_id: String, user_id: String) -> Result<(), DbError> {
    let reminder_bson_id: ObjectId = match reminder_id.parse() {
        Ok(bson_id) => bson_id,
        Err(_) => return Err(DbError::BadId),
    };
    let doc = doc! {"_id": Bson::ObjectId(reminder_bson_id), "user_id": Bson::String(user_id)};

    db_handle.delete_one::<Reminder>(doc).await
}
