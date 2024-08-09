use super::{Category, CategorySchema, Reminder, ReminderSchema};
use crate::error_handler::DbError;
use crate::models::reminder::Priority;
use serde::Deserialize;
use sqlx::error::ErrorKind;
use sqlx::{Error, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

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

pub async fn get_categories_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<Category>, DbError> {
    match sqlx::query_as!(
        Category,
        r#"SELECT id as "id!", slug, name, user_id FROM categories WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(categories) => Ok(categories),
        Err(e) => match e {
            _ => Err(DbError::UnhandledException)
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

pub async fn delete_category_by_id(pool:  &SqlitePool, category_id: i64, user_id: i64) -> Result<u64, DbError> {
    // Verify that the category being deleted is not in use by a reminder
    match sqlx::query!("SELECT COUNT(category_id) as count FROM reminderCategories WHERE category_id = ?", category_id)
        .fetch_one(pool)
        .await
    {
        Ok(res) => {
            if res.count > 0 {
                return Err(DbError::RelationshipViolation("category".to_owned(), category_id.to_string()))
            }
        },
        Err(_e) => return Err(DbError::UnhandledException)
    };

    match sqlx::query!("DELETE FROM categories WHERE id = ? AND user_id = ?", category_id, user_id)
        .execute(pool)
        .await
    {
        Ok(res) => Ok(res.rows_affected()),
        Err(e) => match e {
            Error::RowNotFound => Err(DbError::NotFound("category".to_owned(), category_id.to_string())),
            _ => Err(DbError::UnhandledException)
        }
    }
}

// Reminders
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
                    get_reminder_by_id(pool,intermediary_reminder.id).await
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

pub async fn get_reminder_by_id(pool: &SqlitePool, id: i64) -> Result<Reminder, DbError> {
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

pub async fn get_reminders_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<Reminder>, DbError> {
    match sqlx::query!(
        r#"
        SELECT
            r.id,
            r.name,
            r.description,
            r.priority,
            r.user_id,
            r.date_time,
            COALESCE(rc.categories, '') AS categories
        FROM
            reminders r
        LEFT JOIN
            (
                SELECT
                    reminder_id, GROUP_CONCAT(category_id) AS categories
                FROM
                    reminderCategories
                GROUP BY
                    reminder_id
            ) rc
        ON
            r.id = rc.reminder_id
        WHERE
            r.user_id = ?
        "#,
        user_id
    )
        .fetch_all(pool)
        .await
    {
        Ok(res) => {
            let mut reminders: Vec<Reminder> = Vec::new();
            for row in res {
                let categories: Vec<i64> = row.categories
                    .split(",")
                    .filter_map(|s| s.parse().ok())
                    .collect();
                reminders.push(Reminder {
                    id: row.id,
                    name: row.name,
                    description: row.description,
                    categories,
                    priority: row.priority.into(),
                    user_id: row.user_id,
                    date_time: row.date_time
                });
            }
            Ok(reminders)
        },
        Err(_) => Err(DbError::UnhandledException)
    }
}
