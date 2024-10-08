#[cfg(test)]
mod user_testing {
    use super::*;
    use crate::models::user::ReturnUser;
    use crate::testing::helpers::user_helpers::{auth_user, get_user_me};
    use crate::testing::{json_bytes, TestHelper};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::json;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn user_crud() {
        let helper = TestHelper::init().await;

        let username = "foo";
        let password = "bar";

        // Create a new user
        let create_user_req = create_user_req(username, password, &helper.address);
        let response = helper.client.request(create_user_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "Failed to create a new user"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        // TODO: Should almost definitely not be asserting on the ID here, should deserialize
        //       this into a response struct and assert on WhateverStruct.username
        let user = ReturnUser::from_bytes(&body);
        assert_eq!(user.username.as_str(), "foo");

        // Try to create a user where the username is already taken
        let duplicate_user_req = user_testing::create_user_req(username, password, &helper.address);
        let response = helper.client.request(duplicate_user_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Creating a user with a duplicate username should 400"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"\"Username 'foo' is already taken\"");

        // Auth with the new user
        let token = match auth_user(&helper.client, username, password, &helper.address).await {
            Ok(t) => t,
            Err((_status_code, msg)) => panic!("{}", msg),
        };

        // get user
        let user_me = get_user_me(&helper.client, token.as_str(), &helper.address)
            .await
            .unwrap();
        assert_eq!(user_me.username.as_str(), "foo");

        // delete user
        let delete_me_req = delete_me_req(token.as_str(), &helper.address);
        let response = helper.client.request(delete_me_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Failed to delete a user with the /me endpoint"
        );

        // Try to get the user, verify that they were deleted and that their token does not work
        // TODO: Should be getting the user as admin so we know the request is valid and there is
        //       a better confirmation that the user is deleted
        match get_user_me(&helper.client, token.as_str(), &helper.address).await {
            Ok(_) => panic!("Deleting a user and then trying to get their account should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::UNAUTHORIZED,
                "Getting a user that does not exist should result in a 401 status code"
            ),
        };
    }

    fn create_user_req(username: &str, password: &str, addr: &SocketAddr) -> Request<Body> {
        Request::builder()
            .uri(format!("http://{addr}/api/users"))
            .method("POST")
            .header("Host", "localhost")
            .header("Content-Type", "application/json")
            .body(Body::from(json_bytes(
                json!({"username": username, "password": password}),
            )))
            .unwrap()
    }

    fn delete_me_req(token: &str, addr: &SocketAddr) -> Request<Body> {
        Request::builder()
            .uri(format!("http://{addr}/api/users/me"))
            .method("DELETE")
            .header("Host", "localhost")
            .header("Content-Type", "application/json")
            .header("authorization", token)
            .body(Body::from(Body::empty()))
            .unwrap()
    }
}
