#[cfg(test)]
mod reminder_testing {
    use crate::models::reminder::Priority;
    use crate::testing::helpers::reminder_helpers::{create_category, create_reminder};
    use crate::testing::helpers::user_helpers::{auth_user, create_user, get_user_me};
    use crate::testing::TestHelper;
    use hyper::StatusCode;

    #[tokio::test]
    async fn reminder_crud() {
        let helper = TestHelper::init().await;

        let username = "foo";
        let password = "foo";

        // Create a user
        create_user(&helper.client, username, password, &helper.address)
            .await
            .unwrap();

        // Get a token for the user
        let token = auth_user(&helper.client, username, password, &helper.address)
            .await
            .unwrap();

        // Get our user so we have their id
        let user = get_user_me(&helper.client, token.as_str(), &helper.address)
            .await
            .unwrap();

        // Create a category
        let category_slug = "test_category";
        let created_category = create_category(
            &helper.client,
            token.as_str(),
            category_slug,
            category_slug,
            &helper.address,
        )
        .await
        .expect("Failed to create a new category");
        assert_eq!(created_category.slug.as_str(), category_slug);
        assert_eq!(created_category.name.as_str(), category_slug);
        assert_eq!(created_category.user_id, user.id);

        // Try to create a category that already exists
        match create_category(
            &helper.client,
            token.as_str(),
            category_slug,
            category_slug,
            &helper.address,
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

        // Try to create a reminder that points to a category that does not exist
        // TODO: Implement this check

        // Create a reminder
        let reminder_name = "test_reminder";
        let reminder_categories = vec![created_category.id];
        let created_reminder = create_reminder(
            &helper.client,
            token.as_ref(),
            reminder_name,
            "test_reminder",
            reminder_categories.clone(),
            Priority::Medium,
            &helper.address,
        )
        .await
        .expect("Failed to create a new reminder");
        assert_eq!(created_reminder.name.as_str(), reminder_name);
        assert_eq!(created_reminder.description.as_str(), "test_reminder");
        assert_eq!(created_reminder.categories, reminder_categories);
        assert_eq!(created_reminder.priority, Priority::Medium);
        assert_eq!(created_reminder.user_id, user.id);
    }
}
