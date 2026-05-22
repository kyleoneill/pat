#[cfg(test)]
mod log_testing {
    use crate::testing::{
        helpers::{
            log_helpers::get_logs_for_user,
            user_helpers::{auth_user, create_user, get_user_me},
        },
        TestHelper,
    };

    #[tokio::test]
    async fn log_generation() {
        let helper = TestHelper::init().await;

        let user_one_username = "foo";
        let user_one_password = "bar";

        let user_two_username = "two";
        let user_two_password = "two";

        // Create a user
        create_user(&helper, user_one_username, user_one_password).await.unwrap();

        // Generate logs with our user
        let token = auth_user(&helper, user_one_username, user_one_password).await.unwrap();
        let user = get_user_me(&helper, token.as_str()).await.unwrap();
        get_user_me(&helper, token.as_str()).await.unwrap();

        // Run the log generation task manually rather than waiting for it
        {
            helper
                .task_manager
                .lock()
                .expect("Failed to get task manager mutex lock")
                .run_logs_task()
                .await;
        }

        // Get logs
        let logs = get_logs_for_user(&helper, token.as_str()).await.unwrap();

        // Verify that the logs look correct
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].uri.as_str(), "/api/users/me");
        assert_eq!(logs[0].method.as_str(), "GET");
        assert_eq!(logs[0].user_id, user.id);

        // Create a second user
        create_user(&helper, user_two_username, user_two_password).await.unwrap();

        // Generate logs with the second user
        let token_two = auth_user(&helper, user_two_username, user_two_password).await.unwrap();
        let user_two = get_user_me(&helper, token_two.as_str()).await.unwrap();

        // Run the log generation task manually rather than waiting for it
        {
            helper
                .task_manager
                .lock()
                .expect("Failed to get task manager mutex lock")
                .run_logs_task()
                .await;
        }

        // Get logs for the second user
        let logs = get_logs_for_user(&helper, token_two.as_str()).await.unwrap();

        // Verify that the logs look correct
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, user_two.id);
    }
}
