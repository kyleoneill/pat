#[cfg(test)]
mod user_testing {
    use super::*;
    use crate::generate_app;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use hyper_util::client::legacy::connect::HttpConnector;
    use hyper_util::client::legacy::Client;
    use serde::Serialize;
    use serde_json::json;
    use sqlx::sqlite::SqlitePool;
    use std::net::SocketAddr;
    use std::sync::OnceLock;
    use tokio::net::TcpListener;

    static SERVER: OnceLock<Client<HttpConnector, Body>> = OnceLock::new();
    static ADDR: OnceLock<SocketAddr> = OnceLock::new();

    async fn setup_database() {
        let database_url = dotenv!("DATABASE_URL").to_owned();
        let pool = SqlitePool::connect(database_url.as_str())
            .await
            .expect("Failed to connect to database");
        let _ = sqlx::query!(
            "\
            DELETE FROM users;\
            UPDATE `main`.`sqlite_sequence` SET `seq` = '0' WHERE  `name` = 'users';\
            INSERT INTO users (username, password, auth_level, salt) VALUES ('admin', 'D600AD1AAEA6261F2B5923FE076AE08B42688CDF6051FEF2D8CC4ED303D19E22', 'Admin', 'zTGNpsiiXQ5f');\
            "
        ).execute(&pool).await;
    }

    async fn initialize() {
        setup_database().await;
        match SERVER.get() {
            Some(_val) => (),
            None => {
                let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
                let addr = listener.local_addr().unwrap();
                // TODO: Might want to store this JoinHandle from spawn() so I can clean up resources
                //       correctly at the end of testing?
                tokio::spawn(
                    async move { axum::serve(listener, generate_app().await).await.unwrap() },
                );
                let client = hyper_util::client::legacy::Client::builder(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .build_http();
                SERVER.set(client).unwrap();
                ADDR.set(addr).unwrap();
            }
        }
    }

    fn json_bytes<T>(structure: T) -> Vec<u8>
    where
        T: Serialize,
    {
        let mut bytes: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut bytes, &structure).unwrap();
        bytes
    }

    #[tokio::test]
    async fn user_crud() {
        initialize().await;
        let client = SERVER.get().unwrap();

        let username = "foo";
        let password = "bar";

        // Create a new user
        let create_user_req = create_user_req(username, password);
        let response = client.request(create_user_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "Failed to create a new user"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        // TODO: Should almost definitely not be asserting on the ID here, should deserialize
        //       this into a response struct and assert on WhateverStruct.username
        assert_eq!(&body[..], b"{\"id\":2,\"username\":\"foo\"}");

        // Try to create a user where the username is already taken
        let duplicate_user_req = user_testing::create_user_req(username, password);
        let response = client.request(duplicate_user_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Creating a user with a duplicate username should 400"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"\"Username 'foo' is already taken\"");

        // Auth with the new user
        let auth_req = auth_user_req(username, password);
        let response = client.request(auth_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "Failed to auth as user"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let foo = body.as_ref();
        let slice = &foo[1..foo.len() - 1];
        let token = std::str::from_utf8(slice).unwrap();

        // get user
        let get_me_req = get_user_me_req(token);
        let response = client.request(get_me_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Failed to get a user with the /me endpoint"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        // TODO: See todo above, should deserialize and assert on the username. Also should maybe
        //       not be returning the id here?
        assert_eq!(&body[..], b"{\"id\":2,\"username\":\"foo\"}");

        // delete user
        let delete_me_req = delete_me_req(token);
        let response = client.request(delete_me_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Failed to delete a user with the /me endpoint"
        );

        // Try to get the user, verify that they were deleted and that their token does not work
        // TODO: Should be getting the user as admin so we know the request is valid and there is
        //       a better confirmation that the user is deleted
        let get_me_req = get_user_me_req(token);
        let response = client.request(get_me_req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Deleting a user and then trying to get their account should result in a 404"
        );
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            &body[..],
            b"No user found for the given authorization token"
        );
    }

    fn create_user_req(username: &str, password: &str) -> Request<Body> {
        let addr = ADDR.get().unwrap();
        Request::builder()
            .uri(format!("http://{addr}/api/user"))
            .method("POST")
            .header("Host", "localhost")
            .header("Content-Type", "application/json")
            .body(Body::from(json_bytes(
                json!({"username": username, "password": password}),
            )))
            .unwrap()
    }

    fn auth_user_req(username: &str, password: &str) -> Request<Body> {
        let addr = ADDR.get().unwrap();
        Request::builder()
            .uri(format!("http://{addr}/api/user/auth"))
            .method("POST")
            .header("Host", "localhost")
            .header("Content-Type", "application/json")
            .body(Body::from(json_bytes(
                json!({"username": username, "password": password}),
            )))
            .unwrap()
    }

    fn get_user_me_req(token: &str) -> Request<Body> {
        let addr = ADDR.get().unwrap();
        Request::builder()
            .uri(format!("http://{addr}/api/user/me"))
            .method("GET")
            .header("Host", "localhost")
            .header("Content-Type", "application/json")
            .header("authorization", token)
            .body(Body::from(Body::empty()))
            .unwrap()
    }

    fn delete_me_req(token: &str) -> Request<Body> {
        let addr = ADDR.get().unwrap();
        Request::builder()
            .uri(format!("http://{addr}/api/user/me"))
            .method("DELETE")
            .header("Host", "localhost")
            .header("Content-Type", "application/json")
            .header("authorization", token)
            .body(Body::from(Body::empty()))
            .unwrap()
    }
}
