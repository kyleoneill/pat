#[cfg(test)]
mod log_testing {
    use std::time::Duration;
    use tokio::time::sleep;

    use crate::testing::helpers::log_helpers::get_logs_for_user;
    use crate::testing::helpers::user_helpers::{auth_user, create_user, get_user_me};
    use crate::testing::{initialize, SERVER};

    #[tokio::test(start_paused = true)]
    async fn log_generation() {
        initialize().await;
        let client = SERVER.get().unwrap();

        let user_one_username = "foo";
        let user_one_password = "bar";

        // Create a user
        match create_user(client, user_one_username, user_one_password).await {
            Ok(_) => (),
            Err(msg) => panic!("{}", msg)
        };

        // Auth with our user
        let token = match auth_user(client, user_one_username, user_one_password).await {
            Ok(t) => t,
            Err((_, msg)) => panic!("{}", msg)
        };

        // Generate logs with our user
        let _res = match get_user_me(client, token.as_str()).await{
            Ok(res) => res,
            Err((_, msg)) => panic!("{}", msg)
        };

        // Wait for the log task to run
        sleep(Duration::from_secs(10)).await;

        // Get logs
        let _logs_bytes = match get_logs_for_user(client, token.as_str()).await {
            Ok(bytes) => bytes,
            Err((_, msg)) => panic!("{}", msg)
        };

        // Verify that the logs look correct

        // Create a second user

        // Generate logs with the second user

        // Get logs for the second user, verify that they only see their logs
    }
}
