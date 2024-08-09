#[cfg(test)]
mod reminder_testing {
    use crate::models::reminder::Priority;
    use crate::testing::helpers::reminder_helpers::{create_category, create_reminder, get_categories, delete_category_by_id};
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
        create_user(client, username, password, addr)
            .await
            .unwrap();

        // Get a token for the user
        let token = auth_user(client, username, password, addr)
            .await
            .unwrap();

        // Get our user so we have their id
        let user = get_user_me(client, token.as_str(), addr)
            .await
            .unwrap();

        // Create a category
        let category_slug = "test_category";
        let created_category = create_category(
            client,
            token.as_str(),
            category_slug,
            category_slug,
            addr,
        )
        .await
        .expect("Failed to create a new category");

        // Try to create a reminder that points to a category that does not exist
        // TODO: Implement this check

        // Create a reminder
        let reminder_name = "test_reminder";
        let reminder_categories = vec![created_category.id];
        let created_reminder = create_reminder(
            client,
            token.as_ref(),
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
        assert_eq!(created_reminder.categories, reminder_categories);
        assert_eq!(created_reminder.priority, Priority::Medium);
        assert_eq!(created_reminder.user_id, user.id);
    }

    #[tokio::test]
    async fn category_crud() {
        let helper = TestHelper::init().await;
        let client = &helper.client;
        let addr = &helper.address;

        let username = "foo";
        let password = "foo";

        // Create a user
        create_user(client, username, password, addr)
            .await
            .unwrap();

        // Get a token for the user
        let token = auth_user(client, username, password, addr)
            .await
            .unwrap();

        // Get our user so we have their id
        let user = get_user_me(client, token.as_str(), addr)
            .await
            .unwrap();

        // Create a category
        let category_slug = "test_category";
        let created_category = create_category(
            client,
            token.as_str(),
            category_slug,
            category_slug,
            addr,
        )
            .await
            .expect("Failed to create a new category");
        assert_eq!(created_category.slug.as_str(), category_slug);
        assert_eq!(created_category.name.as_str(), category_slug);
        assert_eq!(created_category.user_id, user.id);

        // Try to create a category that already exists
        match create_category(
            client,
            token.as_str(),
            category_slug,
            category_slug,
            addr,
        )
            .await
        {
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
        delete_category_by_id(client, token.as_str(), addr, categories[1].id).await.expect("Failed to delete category by id");

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
