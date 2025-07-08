#[cfg(test)]
mod chat_testing {
    use crate::testing::{
        helpers::user_helpers::{auth_user, create_user, get_user_me},
        TestHelper,
        FAKE_MONGO_ID,
    };
    use hyper::StatusCode;

    use crate::models::chat::{
        message::CreateMessageSchema,
        chat_channel::{ChannelType, CreateChannelSchema}
    };
    use crate::testing::helpers::chat_helpers::{create_chat_channel, list_channels, receive_chat_message, subscribe_to_channel, unsubscribe_from_channel, send_chat_message, get_channel_by_id};

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
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::NOT_FOUND,
                "Getting a channel by a non-existent ID should 404"
            ),
        };

        // Get a chat channel
        let get_channel = get_channel_by_id(client, addr, token.as_str(), first_channel._id.as_str()).await.expect("Failed to get a chat channel by ID");
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
        // TODO: If I add any more chat tests I should probably have a helper that creates a chat
        //       channel or two with some users and tokens, this is a lot of lines of setup
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

        // Create two channels, owned by different users
        let data = CreateChannelSchema {
            name: Some("First Channel".to_owned()),
            channel_type: 0,
            slug: "first_channel".to_string(),
        };
        let first_channel = create_chat_channel(client, addr, token.as_str(), &data)
            .await
            .expect("Failed to create a chat channel");
        let data_two = CreateChannelSchema {
            name: Some("Second Channel".to_owned()),
            channel_type: 0,
            slug: "second_channel".to_string(),
        };
        let second_channel = create_chat_channel(client, addr, second_token.as_str(), &data_two)
            .await
            .expect("Failed to create a second chat channel");

        // Subscribe to the first channel with the second user
        subscribe_to_channel(client, addr, second_token.as_str(), first_channel._id.as_str())
            .await
            .expect("Failed to subscribe to another users chat channel");

        // Open a websocket connection with the first user for chatting
        let (mut first_socket, _response) =
            tokio_tungstenite::connect_async(format!("ws://{}/api/chat/ws?auth_token={}", addr, token.as_str()))
                .await
                .expect("Failed to open a ws connection with first user");

        // Open a websocket connection with the second user for chatting
        let (mut second_socket, _second_response) =
            tokio_tungstenite::connect_async(format!("ws://{}/api/chat/ws?auth_token={}", addr, second_token.as_str()))
                .await
                .expect("Failed to open a ws connection with second user");

        // ----------------------------------
        // Test receiving a broadcast

        // Create a message as the first user
        let message_data = CreateMessageSchema{ channel_id: first_channel._id.clone(), contents: "Test Message".to_owned(), reply_to: None };
        send_chat_message(&mut first_socket, message_data).await;

        // After the first user creates a message, both users should receive it as a broadcast
        // Wrap this in a timeout to prevent the test from hanging if the socket never receives data
        let chat_message = receive_chat_message(&mut first_socket).await;
        assert_eq!(chat_message.channel_id.as_str(), first_channel._id.as_str());
        assert_eq!(chat_message.author_id.as_str(), user.id.as_str());
        assert_eq!(chat_message.contents.as_str(), "Test Message");
        assert_eq!(chat_message.reply_to, None);
        assert_eq!(chat_message.reactions.len(), 0);
        assert_eq!(chat_message.pinned, false);
        assert_eq!(chat_message.atomic_id, 1);

        let chat_message_user_two = receive_chat_message(&mut second_socket).await;
        // Can prob just impl PartialEq for ChatMessage and then assert that chat_message == chat_message_user_two
        assert_eq!(chat_message_user_two.channel_id.as_str(), first_channel._id.as_str());
        assert_eq!(chat_message_user_two.author_id.as_str(), user.id.as_str());
        assert_eq!(chat_message_user_two.contents.as_str(), "Test Message");
        assert_eq!(chat_message_user_two.reply_to, None);
        assert_eq!(chat_message_user_two.reactions.len(), 0);
        assert_eq!(chat_message_user_two.pinned, false);
        assert_eq!(chat_message_user_two.atomic_id, 1);

        // ----------------------------------
        // Test not receiving a broadcast when a message is sent in a channel a user is not a part of

        // Send a chat message as the second user to a channel only they are subscribed in
        let user_two_unique_message = CreateMessageSchema{ channel_id: second_channel._id.clone(), contents: "Secret Message".to_owned(), reply_to: None };
        send_chat_message(&mut second_socket, user_two_unique_message).await;

        // Receive a message as the second user to clear out their socket
        receive_chat_message(&mut second_socket).await;

        // Send a message as the first user in a channel both users are in, just to generate a
        // broadcast for themselves
        let message_data = CreateMessageSchema{ channel_id: first_channel._id.clone(), contents: "Another message from me".to_owned(), reply_to: None };
        send_chat_message(&mut first_socket, message_data).await;

        // Receive a message as the second user again to clear their socket
        receive_chat_message(&mut second_socket).await;

        // Receive a message as the first user, verify that this is the message they just sent
        // and is not the message the second user sent in their channel to themselves. The
        // websocket is a FIFO queue so this means they did not get the first broadcast
        let user_one_message = receive_chat_message(&mut first_socket).await;
        assert_eq!(user_one_message.channel_id.as_str(), first_channel._id.as_str());

        // Assert that the second message on the first channel has an atomic ID of 2
        assert_eq!(user_one_message.atomic_id, 2);

        // Get the first channel, assert that its most recent message ID has been updated
        let updated_first_channel = get_channel_by_id(client, addr, token.as_str(), first_channel._id.as_str()).await.expect("Failed to get a chat channel by ID");
        assert_eq!(updated_first_channel.most_recent_message_id, 2);


        /*
          TESTING
            - user-1 send a message to a channel that does not exist
            - user-1 send a message to a channel i am not subscribed to
            - request messages
                - request more messages than exists
                - request messages from a channel that doesn't exist
                - request messages from a channel user is not subscribed to
        */
    }
}
