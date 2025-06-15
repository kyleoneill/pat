#[cfg(test)]
mod user_testing {
    use crate::testing::helpers::user_helpers::{auth_user, create_user, delete_user_me, get_user_me};
    use crate::testing::TestHelper;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn user_crud() {
        let helper = TestHelper::init().await;
        let client = &helper.client;
        let addr = &helper.address;

        let username = "foo";
        let password = "bar";

        // Create a new user
        match create_user(client, username, password, addr).await {
            Ok(new_user) => {
                assert_eq!(new_user.username.as_str(), username)
            }
            Err(_) => panic!("Failed to create a new user"),
        }

        // Try to create a user where the username is already taken
        match create_user(client, username, password, addr).await {
            Ok(_) => panic!("Creating a user where the username is already taken should error"),
            Err((status_code, err_msg)) => match status_code {
                StatusCode::BAD_REQUEST => {
                    assert_eq!("Username 'foo' is already taken", err_msg);
                }
                _ => panic!("Creating a user where the username is already taken should 400"),
            },
        }

        // Auth with the new user
        let token = match auth_user(client, username, password, addr).await {
            Ok(t) => t,
            Err((_status_code, msg)) => panic!("{}", msg),
        };

        // get user
        let user_me = get_user_me(client, token.as_str(), addr).await.unwrap();
        assert_eq!(user_me.username.as_str(), "foo");

        // delete user
        match delete_user_me(client, token.as_str(), addr).await {
            Ok(_) => (),
            Err(_e) => panic!("Failed to delete a user with the /me endpoint"),
        }

        // Try to get the user, verify that they were deleted and that their token does not work
        // TODO: Should be getting the user as admin so we know the request is valid and there is
        //       a better confirmation that the user is deleted
        match get_user_me(client, token.as_str(), addr).await {
            Ok(_) => panic!("Deleting a user and then trying to get their account should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::UNAUTHORIZED,
                "Getting a user that does not exist should result in a 401 status code"
            ),
        };
    }
}
