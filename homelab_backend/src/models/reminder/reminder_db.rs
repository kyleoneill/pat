use super::{Category, CategorySchema, Reminder, ReminderSchema};
use crate::error_handler::DbError;
use crate::models::reminder::Priority;
use serde::Deserialize;
use sqlx::error::ErrorKind;
use sqlx::{Error, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

// Categories
pub async fn insert_category(
    pool: &SqlitePool,
    data: &CategorySchema,
    user_id: i64,
) -> Result<Category, DbError> {
    let slug = data.slug.clone();
    match sqlx::query!(
        "INSERT INTO categories (slug, name, user_id) VALUES (?, ?, ?)",
        data.slug,
        data.name,
        user_id
    )
    .execute(pool)
    .await
    {
        Ok(_) => get_category_by_slug(pool, slug.as_str()).await,
        Err(e) => {
            // TODO: If I end up using nosql, I need to make sure that the unique constraint for 'slug' is
            //       preserved
            match e {
                Error::Database(db_err) => match db_err.kind() {
                    ErrorKind::UniqueViolation => {
                        Err(DbError::AlreadyExists("slug".to_owned(), slug))
                    }
                    _ => Err(DbError::UnhandledException),
                },
                _ => Err(DbError::UnhandledException),
            }
        }
    }
}

pub async fn get_category_by_slug(pool: &SqlitePool, slug: &str) -> Result<Category, DbError> {
    match sqlx::query_as!(
        Category,
        r#"SELECT id as "id!", slug, name, user_id FROM categories WHERE slug = ?"#,
        slug
    )
    .fetch_one(pool)
    .await
    {
        Ok(category) => Ok(category),
        Err(e) => match e {
            Error::RowNotFound => Err(DbError::NotFound("category".to_owned(), slug.to_owned())),
            _ => Err(DbError::UnhandledException),
        },
    }
}

// Reminders
#[derive(Deserialize)]
struct IntermediaryReminder {
    id: i64,
    name: String,
    description: String,
    priority: Priority,
    user_id: i64,
    date_time: i64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ReminderCategories {
    reminder_id: i64,
    category_id: i64,
}

pub async fn insert_reminder(
    pool: &SqlitePool,
    data: &ReminderSchema,
    user_id: i64,
) -> Result<Reminder, DbError> {
    let serialized_priority = data.priority as i64;
    let date_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    match sqlx::query!(
        "INSERT INTO reminders (name, description, priority, user_id, date_time) VALUES (?, ?, ?, ?, ?)",
        data.name,
        data.description,
        serialized_priority,
        user_id,
        date_time
    ).execute(pool).await {
        Ok(_) => {
            match sqlx::query_as!(IntermediaryReminder, r#"SELECT id as "id!", name, description, priority, user_id, date_time FROM reminders ORDER BY id DESC"#)
                .fetch_one(pool)
                .await
            {
                Ok(intermediary_reminder) => {
                    for category_id in &data.categories {
                        // TODO: Verify that category IDs exist?
                        sqlx::query!(
                            "INSERT INTO reminderCategories (reminder_id, category_id) VALUES (?, ?)",
                            intermediary_reminder.id,
                            category_id
                        ).execute(pool).await.expect("Unhandled exception inserting reminderCategories");
                    }
                    get_reminder(pool,intermediary_reminder.id).await
                },
                Err(_) => Err(DbError::UnhandledException)
            }
        },
        Err(e) => match e {
            Error::Database(db_err) => {
                match db_err.kind() {
                    _ => Err(DbError::UnhandledException)
                }
            },
            _ => Err(DbError::UnhandledException)
        }
    }
}

pub async fn get_reminder(pool: &SqlitePool, id: i64) -> Result<Reminder, DbError> {
    // TODO: I should be able to do this in a single query where the tables are joined, this is not efficient
    match sqlx::query_as!(IntermediaryReminder, r#"SELECT id as "id!", name, description, priority, user_id, date_time FROM reminders WHERE id = ?"#, id)
        .fetch_one(pool)
        .await {
        Ok(intermediary_reminder) => {
            match sqlx::query_as!(ReminderCategories, r#"SELECT reminder_id as "reminder_id!", category_id as "category_id!" FROM reminderCategories WHERE reminder_id = ?"#, intermediary_reminder.id)
                .fetch_all(pool).await {
                Ok(reminder_categories) => {
                    let categories: Vec<i64> = reminder_categories.iter().map(|reminder_category| reminder_category.category_id).collect();
                    Ok(Reminder {
                        id: intermediary_reminder.id,
                        name: intermediary_reminder.name,
                        description: intermediary_reminder.description,
                        categories,
                        priority: intermediary_reminder.priority,
                        user_id: intermediary_reminder.user_id,
                        date_time: intermediary_reminder.date_time
                    })
                },
                Err(_e) => Err(DbError::UnhandledException)
            }
        },
        Err(e) => match e {
            Error::Database(db_err) => {
                match db_err.kind() {
                    _ => Err(DbError::UnhandledException)
                }
            },
            _ => Err(DbError::UnhandledException)
        }
    }
}
