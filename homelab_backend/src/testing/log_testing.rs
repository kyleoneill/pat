#[cfg(test)]
mod log_testing {
    use crate::testing::helpers::log_helpers::get_logs_for_user;
    use crate::testing::helpers::user_helpers::{auth_user, create_user, get_user_me};
    use crate::testing::TestHelper;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn log_generation() {
        let helper = TestHelper::init().await;

        let user_one_username = "foo";
        let user_one_password = "bar";

        let user_two_username = "two";
        let user_two_password = "two";

        // Create a user
        create_user(&helper.client, user_one_username, user_one_password, &helper.address)
            .await
            .unwrap();

        // Generate logs with our user
        let token = auth_user(&helper.client, user_one_username, user_one_password, &helper.address)
            .await
            .unwrap();
        let user = get_user_me(&helper.client, token.as_str(), &helper.address).await.unwrap();
        get_user_me(&helper.client, token.as_str(), &helper.address).await.unwrap();

        // Wait for the log task to run
        // TODO: _NEED_ to find a way to run this task right here and not wait, tests should not
        //       contain real waits
        sleep(Duration::from_secs(7)).await;

        // Get logs
        let logs = get_logs_for_user(&helper.client, token.as_str(), &helper.address).await.unwrap();

        // Verify that the logs look correct
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].uri.as_str(), "/api/users/me");
        assert_eq!(logs[0].method.as_str(), "GET");
        assert_eq!(logs[0].user_id, user.id);

        // Create a second user
        create_user(&helper.client, user_two_username, user_two_password, &helper.address)
            .await
            .unwrap();

        // Generate logs with the second user
        let token_two = auth_user(&helper.client, user_two_username, user_two_password, &helper.address)
            .await
            .unwrap();
        let user_two = get_user_me(&helper.client, token_two.as_str(), &helper.address).await.unwrap();

        // Wait for the log task to run
        // TODO: _NEED_ to find a way to run this task right here and not wait, tests should not
        //       contain real waits
        sleep(Duration::from_secs(7)).await;

        // Get logs for the second user
        let logs = get_logs_for_user(&helper.client, token_two.as_str(), &helper.address).await.unwrap();

        // Verify that the logs look correct
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, user_two.id);
    }
}
