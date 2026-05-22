#[cfg(test)]
mod user_testing {
    use crate::models::user::validation::UpdateUserSchema;
    use crate::testing::helpers::user_helpers::{auth_user, create_user, delete_user_me, get_user_me, update_user};
    use crate::testing::TestHelper;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn user_crud() {
        let helper = TestHelper::init().await;

        let username = "foo";
        let password = "bar";

        // Create a new user
        create_user(&helper, username, password).await.expect("Failed to create a new user");

        // Try to create a user where the username is already taken
        match create_user(&helper, username, password).await {
            Ok(_) => panic!("Creating a user where the username is already taken should error"),
            Err((status_code, err_msg)) => match status_code {
                StatusCode::BAD_REQUEST => {
                    assert_eq!("Username 'foo' is already taken", err_msg);
                }
                _ => panic!("Creating a user where the username is already taken should 400"),
            },
        }

        // Auth with the new user
        let token = match auth_user(&helper, username, password).await {
            Ok(t) => t,
            Err((_status_code, msg)) => panic!("{}", msg),
        };

        // get user
        let user_me = get_user_me(&helper, token.as_str()).await.unwrap();
        assert_eq!(user_me.username.as_str(), username);

        // Update
        {
            let user_two_username = "user_two";
            let new_username = "user_two_new";
            let user_two_password = "user_two";
            let new_password = "user_two_new";

            // Create a second user to test username collision
            let user_two_token = create_user(&helper, user_two_username, user_two_password).await.unwrap();

            // Try to update the username to one already in use
            let bad_update_data = UpdateUserSchema {
                username: Some(username.to_owned()),
                password: None,
            };
            match update_user(&helper, user_two_token.as_str(), bad_update_data).await {
                Ok(_) => panic!("Updating a username to one already in use should fail"),
                Err((status_code, _msg)) => assert_eq!(
                    status_code,
                    StatusCode::BAD_REQUEST,
                    "Updating a username to one already in use should fail"
                ),
            }

            // Update the username and password
            let update_data = UpdateUserSchema {
                username: Some(new_username.to_owned()),
                password: Some(new_password.to_owned()),
            };
            let updated_user = update_user(&helper, user_two_token.as_str(), update_data)
                .await
                .expect("Failed to update a users username and password");
            assert_eq!(
                updated_user.username.as_str(),
                new_username,
                "Failed to assert that the username has been changed after an update"
            );

            // TODO: https://github.com/kyleoneill/pat/issues/48
            // // Try to make a request with user two's existing token, which should have been kicked
            // match get_user_me(&helper, user_two_token.as_str()).await {
            //     Ok(_) => panic!("User token should have been kicked after their password was changed"),
            //     Err((status_code, _msg)) => assert_eq!(
            //         status_code,
            //         StatusCode::UNAUTHORIZED,
            //         "User token should have been kicked after their password was changed"
            //     )
            // }

            // Auth with the new password, verify the token words
            let new_token = auth_user(&helper, new_username, new_password)
                .await
                .expect("Failed to auth with a new password");
            get_user_me(&helper, new_token.as_str())
                .await
                .expect("Failed to use an auth token generated after password has been changed");
        }

        // delete user
        match delete_user_me(&helper, token.as_str()).await {
            Ok(_) => (),
            Err(_e) => panic!("Failed to delete a user with the /me endpoint"),
        }

        // Try to get the user, verify that they were deleted and that their token does not work
        // TODO: Should be getting the user as admin so we know the request is valid and there is
        //       a better confirmation that the user is deleted
        match get_user_me(&helper, token.as_str()).await {
            Ok(_) => panic!("Deleting a user and then trying to get their account should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::UNAUTHORIZED,
                "Getting a user that does not exist should result in a 401 status code"
            ),
        };
    }
}
