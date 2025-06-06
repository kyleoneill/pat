#[cfg(test)]
mod chat_testing {
    use crate::testing::helpers::user_helpers::{auth_user, create_user, get_user_me};
    use crate::testing::TestHelper;
    use hyper::StatusCode;

    use crate::models::chat::chat_channel::{ChannelType, ChatChannel, CreateChannelSchema};

    use crate::testing::helpers::chat_helpers::create_chat_channel;

    /*
      TESTING
        - create channel
        - create second channel
        - ws connect user-1
        - user-1 subscribe to channel 1
        - ws connect user-2, subscribe to channel 1
        - try to subscribe to a channel that does not exist
        - user-1 send a message to a channel that does not exist
        - user-1 send a message to a channel i am not subscribed to
        - user-1 send a message to a channel i am in
        - user-2 read a message from channel i am subscribed to
        - unsubscribe from channel I am in
        - unsubscribe from a channel that does not exist
        - delete a channel I do not have perms for
        - delete a channel I do have perms for
    */

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
        create_user(client, username_two, password_two, addr)
            .await
            .unwrap();

        // Get tokens for the users
        let token = auth_user(client, username, password, addr).await.unwrap();
        let second_token = auth_user(client, username_two, password_two, addr)
            .await
            .unwrap();

        // Get our user so we have their id
        let user = get_user_me(client, token.as_str(), addr).await.unwrap();
        let user_two = get_user_me(client, second_token.as_str(), addr)
            .await
            .unwrap();

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
        assert_eq!(first_channel.subscribers, vec![user.id.as_str()]);
        assert_eq!(first_channel.owner_id, user.id.as_str());

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
        assert_eq!(third_channel.subscribers, vec![user_two.id.as_str()]);
        assert_eq!(third_channel.owner_id, user_two.id.as_str());
    }
}
