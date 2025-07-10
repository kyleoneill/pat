#[cfg(test)]
mod chat_testing {
    use crate::testing::{
        helpers::user_helpers::{auth_user, create_user, get_user_me},
        TestHelper, FAKE_MONGO_ID,
    };
    use hyper::StatusCode;
    use std::net::SocketAddr;

    use crate::models::chat::{
        chat_channel::{ChannelType, CreateChannelSchema, ReturnChannel},
        message::CreateMessageSchema,
        packet::{RequestMessagesSchema, WebSocketRequest},
    };
    use crate::models::user::ReturnUser;
    use crate::testing::helpers::chat_helpers::{
        create_chat_channel, get_channel_by_id, list_channels, receive_chat_message, receive_chat_state, send_arbitrary_data, send_websocket_request,
        subscribe_to_channel, unsubscribe_from_channel,
    };

    use axum::body::Body;
    use hyper_util::client::legacy::{connect::HttpConnector, Client};

    struct ChatHelper {
        users: Vec<ReturnUser>,
        tokens: Vec<String>,
        channels: Vec<ReturnChannel>,
    }

    impl ChatHelper {
        pub async fn setup_chat(client: &Client<HttpConnector, Body>, addr: &SocketAddr, user_and_channel_count: usize) -> Self {
            let mut users: Vec<ReturnUser> = Vec::new();
            let mut tokens: Vec<String> = Vec::new();
            let mut channels: Vec<ReturnChannel> = Vec::new();
            for n in 0..user_and_channel_count {
                let username = format!("user-{}", n);
                let password = format!("user-{}", n);

                // TODO: Client and addr are being passed all over the place, and here it's causing a needless import. Should
                //       bundle this into an actual testing struct
                // Create a user, get the user and a token for the user
                let user = create_user(client, username.as_str(), password.as_str(), addr).await.unwrap();
                let token = auth_user(client, username.as_str(), password.as_str(), addr).await.unwrap();

                // Create a channel for the user
                let data = CreateChannelSchema {
                    name: Some(format!("channel-{}", n)),
                    channel_type: 0,
                    slug: format!("channel-{}", n),
                };
                let channel = create_chat_channel(client, addr, token.as_str(), &data)
                    .await
                    .expect("Failed to create a chat channel");

                users.push(user);
                tokens.push(token);
                channels.push(channel);
            }
            Self { users, tokens, channels }
        }
    }

    #[tokio::test]
    async fn chat_channels_crud() {
        let helper = TestHelper::init().await;
        let client = &helper.client;
        let addr = &helper.address;

        let username = "foo";
        let password = "foo";

        let username_two = "second";
        let password_two = "second";

        // Create users
        create_user(client, username, password, addr).await.unwrap();
        create_user(client, username_two, password_two, addr).await.unwrap();

        // Get tokens for the users
        let token = auth_user(client, username, password, addr).await.unwrap();
        let second_token = auth_user(client, username_two, password_two, addr).await.unwrap();

        // Get our user so we have their id
        let user = get_user_me(client, token.as_str(), addr).await.unwrap();
        let user_two = get_user_me(client, second_token.as_str(), addr).await.unwrap();

        // Create a channel without a name
        let data = CreateChannelSchema {
            name: None,
            channel_type: 0,
            slug: "test_channel".to_string(),
        };
        let first_channel = create_chat_channel(client, addr, token.as_str(), &data)
            .await
            .expect("Failed to create a chat channel");
        assert_eq!(first_channel.name, None);
        assert_eq!(first_channel.slug.as_str(), "test_channel");
        assert_eq!(first_channel.channel_type, ChannelType::DirectMessage);
        assert_eq!(first_channel.pinned_messages.len(), 0);
        assert_eq!(first_channel.subscribers, vec![user.clone().into()]);
        assert_eq!(first_channel.owner_id, user.id.as_str());
        assert_eq!(first_channel.most_recent_message_id, 0);

        // Create a named group-chat channel
        let data_two = CreateChannelSchema {
            name: Some("My Channel".to_string()),
            channel_type: 1,
            slug: "second_channel".to_string(),
        };
        let second_channel = create_chat_channel(client, addr, token.as_str(), &data_two)
            .await
            .expect("Failed to create a chat channel with a name");
        assert_eq!(second_channel.name, Some("My Channel".to_string()));
        assert_eq!(second_channel.channel_type, ChannelType::Group);

        // Try to create a channel with a duplicate slug for a user, which should fail
        match create_chat_channel(client, addr, token.as_str(), &data).await {
            Ok(_) => panic!("Creating a chat channel with a duplicate slug for a user should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::BAD_REQUEST,
                "Creating a chat channel with a duplicate slug for a user should 400"
            ),
        };

        // Create a channel with a duplicate slug, but for a new user which should be fine
        let third_channel = create_chat_channel(client, addr, second_token.as_str(), &data)
            .await
            .expect("Failed to create a chat channel");
        assert_eq!(third_channel.slug.as_str(), "test_channel");
        assert_eq!(third_channel.subscribers, vec![user_two.clone().into()]);
        assert_eq!(third_channel.owner_id, user_two.id.as_str());

        // Try to subscribe to a channel that doesn't exist
        match subscribe_to_channel(client, addr, token.as_str(), token.as_str()).await {
            Ok(_) => panic!("Subscribing to a non-existent channel should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::NOT_FOUND,
                "Subscribing to a chat channel that doesn't exist should 404"
            ),
        };

        // Try to get a channel that does not exist
        match get_channel_by_id(client, addr, token.as_str(), FAKE_MONGO_ID).await {
            Ok(_) => panic!("Getting a channel by a non-existent ID should fail"),
            Err((status_code, _msg)) => assert_eq!(status_code, StatusCode::NOT_FOUND, "Getting a channel by a non-existent ID should 404"),
        };

        // Get a chat channel
        let get_channel = get_channel_by_id(client, addr, token.as_str(), first_channel._id.as_str())
            .await
            .expect("Failed to get a chat channel by ID");
        assert_eq!(get_channel._id.as_str(), first_channel._id.as_str());

        // Subscribe to a channel
        let subscribed_channel = subscribe_to_channel(client, addr, token.as_str(), third_channel._id.as_str())
            .await
            .expect("Failed to subscribe to another users chat channel");
        assert!(subscribed_channel.subscribers.contains(&user.clone().into()));

        // Try to subscribe to a channel the user is already subscribed to
        match subscribe_to_channel(client, addr, token.as_str(), first_channel._id.as_str()).await {
            Ok(_) => panic!("Subscribing to a channel already subscribed to should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::NOT_FOUND,
                "Subscribing to a channel already subscribed to should 404"
            ),
        };

        // Try to unsubscribe from a channel that doesn't exist
        match unsubscribe_from_channel(client, addr, token.as_str(), token.as_str()).await {
            Ok(_) => panic!("Unsubscribing from a non-existent channel should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::NOT_FOUND,
                "Unsubscribing from a chat channel that doesn't exist should 404"
            ),
        };

        // Unsubscribe from a channel
        let unsubscribed_channel = unsubscribe_from_channel(client, addr, token.as_str(), third_channel._id.as_str())
            .await
            .expect("Failed to unsubscribe from another users chat channel");
        assert!(!unsubscribed_channel.subscribers.contains(&user.clone().into()));

        // Try to unsubscribe from a channel the user is not in
        match unsubscribe_from_channel(client, addr, token.as_str(), third_channel._id.as_str()).await {
            Ok(_) => panic!("Unsubscribing from a channel a user is not in should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::NOT_FOUND,
                "Unsubscribing from a channel a user is not in should 404"
            ),
        };

        // Try to unsubscribe from an owned channel, which should fail
        match unsubscribe_from_channel(client, addr, token.as_str(), first_channel._id.as_str()).await {
            Ok(_) => panic!("Unsubscribing from an owned channel should fail"),
            Err((status_code, _msg)) => assert_eq!(status_code, StatusCode::NOT_FOUND, "Unsubscribing from an owned channel should 404"),
        };

        // List all channels with no query params
        let all_channels = list_channels(client, addr, token.as_str(), "")
            .await
            .expect("Failed to list all chat channels");
        assert_eq!(all_channels.len(), 3);

        // List only my channels
        let my_channels = list_channels(client, addr, token.as_str(), "?my_channels=true")
            .await
            .expect("Failed to list channels filtered to ones owned by the requester");
        assert_eq!(my_channels.len(), 2);
        assert_eq!(my_channels[0].owner_id, user.id);
        assert_eq!(my_channels[1].owner_id, user.id);

        // List only other channels
        let other_channels = list_channels(client, addr, token.as_str(), "?my_channels=false")
            .await
            .expect("Failed to list channels owned by users other than the requester");
        assert_eq!(other_channels.len(), 1);
        assert_eq!(other_channels[0].owner_id, user_two.id);

        // List subscribed channels, include "my_channels" here as the user is currently only subscribed to their own channels
        let subscribed = list_channels(client, addr, token.as_str(), "?subscribed=true&my_channels=true")
            .await
            .expect("Failed to list channels the requester subscribes to");
        assert_eq!(subscribed.len(), 2);
        assert!(subscribed[0].subscribers.contains(&user.clone().into()));
        assert!(subscribed[1].subscribers.contains(&user.clone().into()));

        // List unsubscribed channels
        let unsubscribed = list_channels(client, addr, token.as_str(), "?subscribed=false")
            .await
            .expect("Failed to list channels the requester is not subscribed to");
        assert_eq!(unsubscribed.len(), 1);
        assert!(!unsubscribed[0].subscribers.contains(&user.clone().into()));

        // TODO: Delete channel
        // TODO: Try to delete a channel not owned by me
    }

    #[tokio::test]
    async fn chat_messages_basic() {
        let helper = TestHelper::init().await;
        let client = &helper.client;
        let addr = &helper.address;

        let chat_helper = ChatHelper::setup_chat(client, addr, 2).await;
        let token = chat_helper.tokens[0].as_str();
        let second_token = chat_helper.tokens[1].as_str();
        let user_one_id = chat_helper.users[0].id.as_str();
        let user_two_id = chat_helper.users[1].id.as_str();
        let channel_one_id = chat_helper.channels[0]._id.as_str();
        let channel_two_id = chat_helper.channels[1]._id.as_str();

        // Subscribe to the first channel with the second user
        subscribe_to_channel(client, addr, second_token, channel_one_id)
            .await
            .expect("Failed to subscribe to another users chat channel");

        // Open a websocket connection with the first user for chatting
        let (mut first_socket, _response) = tokio_tungstenite::connect_async(format!("ws://{}/api/chat/ws?auth_token={}", addr, token))
            .await
            .expect("Failed to open a ws connection with first user");

        // Open a websocket connection with the second user for chatting
        let (mut second_socket, _second_response) =
            tokio_tungstenite::connect_async(format!("ws://{}/api/chat/ws?auth_token={}", addr, second_token))
                .await
                .expect("Failed to open a ws connection with second user");

        // ----------------------------------
        // Test receiving a broadcast

        // Create a message as the first user
        let message_data: WebSocketRequest = CreateMessageSchema {
            channel_id: channel_one_id.to_string(),
            contents: "Test Message".to_owned(),
            reply_to: None,
        }
        .into();
        send_websocket_request(&mut first_socket, &message_data).await;

        // After the first user creates a message, both users should receive it as a broadcast
        let chat_message = receive_chat_message(&mut first_socket)
            .await
            .expect("Failed to receive a chat message after sending one");
        assert_eq!(chat_message.channel_id.as_str(), channel_one_id);
        assert_eq!(chat_message.author_id.as_str(), user_one_id);
        assert_eq!(chat_message.contents.as_str(), "Test Message");
        assert_eq!(chat_message.reply_to, None);
        assert_eq!(chat_message.reactions.len(), 0);
        assert_eq!(chat_message.pinned, false);
        assert_eq!(chat_message.atomic_id, 1);

        let chat_message_user_two = receive_chat_message(&mut second_socket)
            .await
            .expect("Failed to receive a chat message after one was sent in a subscribed channel");
        assert_eq!(chat_message_user_two, chat_message);

        // ----------------------------------
        // Test not receiving a broadcast when a message is sent in a channel a user is not a part of

        // Send a chat message as the second user to a channel only they are subscribed in
        let user_two_unique_message: WebSocketRequest = CreateMessageSchema {
            channel_id: channel_two_id.to_string(),
            contents: "Secret Message".to_owned(),
            reply_to: None,
        }
        .into();
        send_websocket_request(&mut second_socket, &user_two_unique_message).await;

        // Receive a message as the second user to clear out their socket
        let user_two_only_msg = receive_chat_message(&mut second_socket)
            .await
            .expect("Failed to receive a chat message when one was expected");
        assert_eq!(user_two_only_msg.author_id.as_str(), user_two_id);

        // Send a message as the first user in a channel both users are in, just to generate a
        // broadcast for themselves
        let message_data: WebSocketRequest = CreateMessageSchema {
            channel_id: channel_one_id.to_string(),
            contents: "Another message from me".to_owned(),
            reply_to: None,
        }
        .into();
        send_websocket_request(&mut first_socket, &message_data).await;

        // Receive a message as the second user again to clear their socket
        receive_chat_message(&mut second_socket)
            .await
            .expect("Failed to receive a chat message when one was expected");

        // Receive a message as the first user, verify that this is the message they just sent
        // and is not the message the second user sent in their channel to themselves. The
        // websocket is a FIFO queue so this means they did not get the first broadcast
        let user_one_message = receive_chat_message(&mut first_socket)
            .await
            .expect("Failed to receive a chat message when one was expected");
        assert_eq!(user_one_message.channel_id.as_str(), channel_one_id);

        // Assert that the second message on the first channel has an atomic ID of 2
        assert_eq!(user_one_message.atomic_id, 2);

        // Get the first channel, assert that its most recent message ID has been updated
        let updated_first_channel = get_channel_by_id(client, addr, token, channel_one_id)
            .await
            .expect("Failed to get a chat channel by ID");
        assert_eq!(updated_first_channel.most_recent_message_id, 2);

        // ----------------------------------

        // Try to send a message to a channel that does not exist
        let message_data_bad_channel: WebSocketRequest = CreateMessageSchema {
            channel_id: FAKE_MONGO_ID.to_string(),
            contents: "Bad Message".to_owned(),
            reply_to: None,
        }
        .into();
        send_websocket_request(&mut first_socket, &message_data_bad_channel).await;
        match receive_chat_message(&mut first_socket).await {
            Ok(_) => panic!("Should receive a WebSocketError after sending a chat message to a channel that doesn't exist"),
            Err(e) => assert_eq!(e.status_code, 404),
        }

        // Try to send a message to a channel the user is not a subscriber of
        let unauthorized_message: WebSocketRequest = CreateMessageSchema {
            channel_id: channel_two_id.to_string(),
            contents: "Bad Message".to_owned(),
            reply_to: None,
        }
        .into();
        send_websocket_request(&mut first_socket, &unauthorized_message).await;
        match receive_chat_message(&mut first_socket).await {
            Ok(_) => panic!("Should receive a WebSocketError after sending a chat message to a channel that the user is not subscribed to"),
            Err(e) => assert_eq!(e.status_code, 400),
        }

        // Send garbage data, assert that we get a 400 back
        send_arbitrary_data(&mut first_socket, "This cannot be deserialized".to_owned()).await;
        match receive_chat_message(&mut first_socket).await {
            Ok(_) => panic!("Should receive a WebSocketError after sending arbitrary text that cannot be deserialized"),
            Err(e) => assert_eq!(e.status_code, 400),
        }
    }

    #[tokio::test]
    async fn chat_state() {
        let helper = TestHelper::init().await;
        let client = &helper.client;
        let addr = &helper.address;

        let chat_helper = ChatHelper::setup_chat(client, addr, 2).await;
        let token = chat_helper.tokens[0].as_str();
        let second_token = chat_helper.tokens[1].as_str();
        let channel_one_id = chat_helper.channels[0]._id.as_str();
        let channel_two_id = chat_helper.channels[1]._id.as_str();

        // Subscribe to the first channel with the second user
        subscribe_to_channel(client, addr, second_token, channel_one_id)
            .await
            .expect("Failed to subscribe to another users chat channel");

        // Open a websocket connection with the first user for chatting
        let (mut first_socket, _response) = tokio_tungstenite::connect_async(format!("ws://{}/api/chat/ws?auth_token={}", addr, token))
            .await
            .expect("Failed to open a ws connection with first user");

        // Open a websocket connection with the second user for chatting
        let (mut second_socket, _second_response) =
            tokio_tungstenite::connect_async(format!("ws://{}/api/chat/ws?auth_token={}", addr, second_token))
                .await
                .expect("Failed to open a ws connection with second user");

        // Send some messages as the second user
        for n in 0..20 {
            let message_data: WebSocketRequest = CreateMessageSchema {
                channel_id: channel_one_id.to_string(),
                contents: format!("Chat message {}", n),
                reply_to: None,
            }
            .into();
            send_websocket_request(&mut second_socket, &message_data).await;
        }

        // Read all generated messages as user-1 to clear the socket
        for _ in 0..20 {
            receive_chat_message(&mut first_socket)
                .await
                .expect("Failed to receive a chat message when one was expected");
        }

        // Request messages from a channel that does not exist
        let request_invalid_channel: WebSocketRequest = RequestMessagesSchema {
            message_count: 25,
            atomic_message_id: 1,
            channel_id: FAKE_MONGO_ID.to_string(),
        }
        .into();
        send_websocket_request(&mut first_socket, &request_invalid_channel).await;
        match receive_chat_state(&mut first_socket).await {
            Ok(_) => panic!("Should receive a WebSocketError when requesting chat state for a channel that doesn't exist"),
            Err(e) => assert_eq!(e.status_code, 404),
        }

        // Request messages from a channel the user is not subscribed to
        let request_invalid_channel: WebSocketRequest = RequestMessagesSchema {
            message_count: 25,
            atomic_message_id: 1,
            channel_id: channel_two_id.to_string(),
        }
        .into();
        send_websocket_request(&mut first_socket, &request_invalid_channel).await;
        match receive_chat_state(&mut first_socket).await {
            Ok(_) => panic!("Should receive a WebSocketError when requesting chat state for a channel that the user is not subscribed to"),
            Err(e) => assert_eq!(e.status_code, 400),
        }

        // Try to request too many messages at once
        let request_too_many_messages: WebSocketRequest = RequestMessagesSchema {
            message_count: 100,
            atomic_message_id: 1,
            channel_id: channel_one_id.to_string(),
        }
        .into();
        send_websocket_request(&mut first_socket, &request_too_many_messages).await;
        match receive_chat_state(&mut first_socket).await {
            Ok(_) => panic!("Should receive a WebSocketError when requesting too many chat messages at once"),
            Err(e) => assert_eq!(e.status_code, 400),
        }

        // Request 5 chat messages starting at some arbitrary message, assert we get the right ones back
        let request_some_messages: WebSocketRequest = RequestMessagesSchema {
            message_count: 5,
            atomic_message_id: 15,
            channel_id: channel_one_id.to_string(),
        }
        .into();
        send_websocket_request(&mut first_socket, &request_some_messages).await;
        let some_messages = receive_chat_state(&mut first_socket).await.expect("Failed to get some chat messages");
        assert_eq!(some_messages.len(), 5);
        for n in 0usize..5usize {
            let id_should_be = (11 + n) as i64;
            assert_eq!(some_messages[n].atomic_id, id_should_be);
        }

        // Request 10 chat messages when only two can possibly be returned
        let request_some_messages: WebSocketRequest = RequestMessagesSchema {
            message_count: 10,
            atomic_message_id: 2,
            channel_id: channel_one_id.to_string(),
        }
        .into();
        send_websocket_request(&mut first_socket, &request_some_messages).await;
        let messages_we_can_get = receive_chat_state(&mut first_socket).await.expect("Failed to get some chat messages");
        assert_eq!(messages_we_can_get.len(), 2);
        assert_eq!(messages_we_can_get[0].atomic_id, 1);
        assert_eq!(messages_we_can_get[1].atomic_id, 2);

        // Request 10 chat messages starting at an id that does not exist
        let request_some_messages: WebSocketRequest = RequestMessagesSchema {
            message_count: 10,
            atomic_message_id: 1000,
            channel_id: channel_one_id.to_string(),
        }
        .into();
        send_websocket_request(&mut first_socket, &request_some_messages).await;
        let no_messages = receive_chat_state(&mut first_socket).await.expect("Failed to get some chat messages");
        assert_eq!(no_messages.len(), 0);
    }

    #[tokio::test]
    async fn chat_message_order() {
        // Rapidly generates 1000 messages, make sure that they are received in the correct order
        // randomize author in the generation, make sure that the broadcasts are received correctly?
        let helper = TestHelper::init().await;
        let client = &helper.client;
        let addr = &helper.address;

        let chat_helper = ChatHelper::setup_chat(client, addr, 2).await;
        let token = chat_helper.tokens[0].as_str();
        let second_token = chat_helper.tokens[1].as_str();
        let user_two_id = chat_helper.users[1].id.as_str();
        let channel_one_id = chat_helper.channels[0]._id.as_str();

        // Subscribe to the first channel with the second user
        subscribe_to_channel(client, addr, second_token, channel_one_id)
            .await
            .expect("Failed to subscribe to another users chat channel");

        // Open a websocket connection with the first user for chatting
        let (mut first_socket, _response) = tokio_tungstenite::connect_async(format!("ws://{}/api/chat/ws?auth_token={}", addr, token))
            .await
            .expect("Failed to open a ws connection with first user");

        // Open a websocket connection with the second user for chatting
        let (mut second_socket, _second_response) =
            tokio_tungstenite::connect_async(format!("ws://{}/api/chat/ws?auth_token={}", addr, second_token))
                .await
                .expect("Failed to open a ws connection with second user");

        // Quickly make 100 messages where the author is alternated
        for n in 0..100 {
            let message_data: WebSocketRequest = CreateMessageSchema {
                channel_id: channel_one_id.to_string(),
                contents: format!("Chat message {}", n),
                reply_to: None,
            }
            .into();
            send_websocket_request(&mut second_socket, &message_data).await;
        }

        // Receive messages as one of the users, verify that they're received in order
        for n in 0..100 {
            let chat_message = receive_chat_message(&mut first_socket)
                .await
                .expect("Failed to receive a chat message when one was expected");
            assert_eq!(chat_message.atomic_id, n + 1);
            assert_eq!(chat_message.author_id.as_str(), user_two_id);
        }
    }
}
