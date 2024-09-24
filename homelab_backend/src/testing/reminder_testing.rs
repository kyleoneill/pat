#[cfg(test)]
mod reminder_testing {
    use crate::models::reminder::{Priority, ReminderUpdateSchema};
    use crate::testing::helpers::reminder_helpers::{
        create_category, create_reminder, delete_category_by_id, delete_reminder_helper,
        get_categories, list_reminders, update_reminder_helper,
    };
    use crate::testing::helpers::user_helpers::{auth_user, create_user, get_user_me};
    use crate::testing::TestHelper;
    use hyper::StatusCode;

    #[tokio::test]
    async fn reminder_crud() {
        let helper = TestHelper::init().await;
        let client = &helper.client;
        let addr = &helper.address;

        let username = "foo";
        let password = "foo";

        // Create a user
        create_user(client, username, password, addr).await.unwrap();

        // Get a token for the user
        let token = auth_user(client, username, password, addr).await.unwrap();

        // Get our user so we have their id
        let user = get_user_me(client, token.as_str(), addr).await.unwrap();

        // Create two categories
        let created_category = create_category(
            client,
            token.as_str(),
            "test_category",
            "test_category",
            addr,
        )
        .await
        .expect("Failed to create a new category");

        let second_category = create_category(
            client,
            token.as_str(),
            "second_category",
            "second_category",
            addr,
        )
        .await
        .expect("Failed to create a second new category");

        // Try to create a reminder that points to a category that does not exist
        // TODO: Implement this check

        // Create a reminder which has two IDs
        let reminder_name = "test_reminder";
        let reminder_categories = vec![created_category.id, second_category.id];
        let created_reminder = create_reminder(
            client,
            token.as_str(),
            reminder_name,
            "test_reminder",
            reminder_categories.clone(),
            Priority::Medium,
            addr,
        )
        .await
        .expect("Failed to create a new reminder");
        assert_eq!(created_reminder.name.as_str(), reminder_name);
        assert_eq!(created_reminder.description.as_str(), "test_reminder");
        assert_eq!(created_reminder.categories, reminder_categories.clone());
        assert_eq!(created_reminder.priority, Priority::Medium);
        assert_eq!(created_reminder.user_id, user.id);

        // Try to delete a category used by a reminder
        match delete_category_by_id(client, token.as_str(), addr, created_category.id).await {
            Ok(_) => panic!("Should not be able to delete a category linked to a reminder"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::BAD_REQUEST,
                "Trying to delete a category being used by a reminder should fail with a 400"
            ),
        };

        // Create a second reminder
        let second_reminder = create_reminder(
            client,
            token.as_str(),
            "second_reminder",
            "second test reminder",
            reminder_categories.clone(),
            Priority::Medium,
            addr,
        )
        .await
        .expect("Failed to create a second reminder");

        // List all reminders
        let reminders = list_reminders(client, addr, token.as_str())
            .await
            .expect("Failed to get a list of reminders");
        assert_eq!(reminders.len(), 2);
        assert_eq!(reminders[0], created_reminder);
        assert_eq!(reminders[1], second_reminder);

        // Try to update a reminder with no data, which should fail
        let bad_update_data = ReminderUpdateSchema {
            name: None,
            description: None,
            categories: None,
            priority: None,
        };
        match update_reminder_helper(
            client,
            addr,
            token.as_str(),
            reminders[0].id,
            bad_update_data,
        )
        .await
        {
            Ok(_) => panic!("Updating a category with no update data should fail."),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::BAD_REQUEST,
                "Trying to update a category with no data should 400"
            ),
        };

        // Update a reminder
        let update_data = ReminderUpdateSchema {
            name: None,
            description: Some("This is a new description".to_owned()),
            categories: None,
            priority: None,
        };
        let update_reminder_res =
            update_reminder_helper(client, addr, token.as_str(), reminders[0].id, update_data)
                .await
                .expect("Failed to update a reminder");
        assert_eq!(update_reminder_res.name.as_str(), reminder_name);
        assert_eq!(
            update_reminder_res.description.as_str(),
            "This is a new description"
        );

        // Update a reminder while changing both the name and categories fields
        let multiple_updates = ReminderUpdateSchema {
            name: Some("new name".to_owned()),
            description: None,
            categories: Some(vec![created_category.id]),
            priority: None,
        };
        let second_update = update_reminder_helper(
            client,
            addr,
            token.as_str(),
            reminders[0].id,
            multiple_updates,
        )
        .await
        .expect("Failed to update a reminder");
        assert_eq!(second_update.categories, vec![created_category.id]);
        assert_eq!(second_update.name.as_str(), "new name");

        // Delete a reminder
        delete_reminder_helper(client, addr, token.as_str(), reminders[0].id)
            .await
            .expect("Failed to delete a reminder");

        // List reminders, there should only be one now
        let reminders_again = list_reminders(client, addr, token.as_str())
            .await
            .expect("Failed to get a list of reminders");
        assert_eq!(reminders_again.len(), 1);
    }

    #[tokio::test]
    async fn category_crud() {
        let helper = TestHelper::init().await;
        let client = &helper.client;
        let addr = &helper.address;

        let username = "foo";
        let password = "foo";

        // Create a user
        create_user(client, username, password, addr).await.unwrap();

        // Get a token for the user
        let token = auth_user(client, username, password, addr).await.unwrap();

        // Get our user so we have their id
        let user = get_user_me(client, token.as_str(), addr).await.unwrap();

        // Create a category
        let category_slug = "test_category";
        let created_category =
            create_category(client, token.as_str(), category_slug, category_slug, addr)
                .await
                .expect("Failed to create a new category");
        assert_eq!(created_category.slug.as_str(), category_slug);
        assert_eq!(created_category.name.as_str(), category_slug);
        assert_eq!(created_category.user_id, user.id);

        // Try to create a category that already exists
        match create_category(client, token.as_str(), category_slug, category_slug, addr).await {
            Ok(_) => panic!("Creating a category with a duplicate slug should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::BAD_REQUEST,
                "Creating a category with a duplicate slug should 400"
            ),
        };

        // Create a second category
        let second_category_slug = "second_category";
        create_category(
            client,
            token.as_str(),
            second_category_slug,
            second_category_slug,
            addr,
        )
        .await
        .expect("Failed to create a new category");

        // Get all categories
        let categories = get_categories(&helper.client, token.as_str(), &helper.address)
            .await
            .expect("Failed to get categories for a user");
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].user_id, user.id);
        assert_eq!(categories[1].user_id, user.id);

        // Delete a category
        delete_category_by_id(client, token.as_str(), addr, categories[1].id)
            .await
            .expect("Failed to delete category by id");

        // Try to delete a category that was already deleted
        match delete_category_by_id(client, token.as_str(), addr, categories[1].id).await {
            Ok(_) => panic!("Deleting a category that does not exist should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::NOT_FOUND,
                "Trying to delete a category that no longer exists should 404"
            ),
        };
    }
}
