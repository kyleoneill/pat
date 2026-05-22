#[cfg(test)]
mod games_testing {
    use crate::models::games::validation::{CreateConnectionCategorySchema, CreateConnectionGameSchema};
    use crate::testing::{
        helpers::{
            games_helpers::{create_connections_game, get_game_to_play, list_connections_games, try_connections_solution},
            user_helpers::{create_user, get_user_me},
        },
        TestHelper,
    };
    use hyper::StatusCode;

    #[tokio::test]
    async fn connections_crud() {
        let helper = TestHelper::init().await;

        let username = "foo";
        let password = "foo";

        let username_two = "second";
        let password_two = "second";

        // Create users
        let token = create_user(&helper, username, password).await.unwrap();
        let second_token = create_user(&helper, username_two, password_two).await.unwrap();

        // Get our user so we have their id
        let user = get_user_me(&helper, token.as_str()).await.unwrap();
        let user_two = get_user_me(&helper, second_token.as_str()).await.unwrap();

        let connection_categories = [
            CreateConnectionCategorySchema {
                category_clues: ["foo".to_string(), "bar".to_string(), "baz".to_string(), "bash".to_string()],
                category_name: "first".to_string(),
            },
            CreateConnectionCategorySchema {
                category_clues: ["foo".to_string(), "bar".to_string(), "baz".to_string(), "bash".to_string()],
                category_name: "second".to_string(),
            },
            CreateConnectionCategorySchema {
                category_clues: ["foo".to_string(), "bar".to_string(), "baz".to_string(), "bash".to_string()],
                category_name: "third".to_string(),
            },
            CreateConnectionCategorySchema {
                category_clues: ["foo".to_string(), "bar".to_string(), "baz".to_string(), "bash".to_string()],
                category_name: "fourth".to_string(),
            },
        ];

        // Create a connections game
        let data = CreateConnectionGameSchema {
            connection_categories: connection_categories.clone(),
            puzzle_name: "Test Puzzle".to_string(),
        };
        let connection_game = create_connections_game(&helper, token.as_str(), &data)
            .await
            .expect("Failed to create a connections game");
        assert_eq!(connection_game.author_id, user.id);
        assert_eq!(connection_game.puzzle_name, "Test Puzzle");

        // Try to create a connections game with a duplicate slug
        match create_connections_game(&helper, token.as_str(), &data).await {
            Ok(_) => panic!("Creating a connections game with a duplicate slug should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::BAD_REQUEST,
                "Creating a connection game with a duplicate slug should 400"
            ),
        }

        // Create a second connections game
        let second_data = CreateConnectionGameSchema {
            connection_categories: connection_categories.clone(),
            puzzle_name: "Second Test Puzzle".to_string(),
        };
        let _second_connections_game = create_connections_game(&helper, token.as_str(), &second_data)
            .await
            .expect("Failed to create a connections game");

        // Create a connections game as a second user
        let other_user_data = CreateConnectionGameSchema {
            connection_categories: connection_categories.clone(),
            puzzle_name: "Other User Test Puzzle".to_string(),
        };
        let other_user_connections_game = create_connections_game(&helper, second_token.as_str(), &other_user_data)
            .await
            .expect("Failed to create a connections game");

        // List "my" connections games for the first user, there should be two
        let my_connections_games = list_connections_games(&helper, token.as_str(), true)
            .await
            .expect("Failed to get connections games for 'me'");
        assert_eq!(my_connections_games.len(), 2);
        assert_eq!(my_connections_games[0].author_id, user.id);
        assert_eq!(my_connections_games[1].author_id, user.id);

        // List all connections games for other users as the first user, there should be one
        let other_connections_games = list_connections_games(&helper, token.as_str(), false)
            .await
            .expect("Failed to get connections games for other users");
        assert_eq!(other_connections_games.len(), 1);
        assert_eq!(other_connections_games[0].author_id, user_two.id);

        // Try to get a game to play that doesn't exist
        match get_game_to_play(&helper, token.as_str(), "i-dont-exist").await {
            Ok(_) => panic!("Getting a connections game with a nonexistent slug should fail"),
            Err((status_code, _msg)) => assert_eq!(
                status_code,
                StatusCode::NOT_FOUND,
                "Getting a connections game with a nonexistent slug should 404"
            ),
        }

        // Get a connections game to play, this should give us 16 scrambled words
        let play_game = get_game_to_play(&helper, token.as_str(), other_user_connections_game.slug.as_str())
            .await
            .expect("Failed to get a connections game to play");
        assert_eq!(play_game.scrambled_clues.len(), 16);

        // Try to solve a row with an incorrect solution
        let bad_guess = ["wrong".to_string(), "wrong".to_string(), "wrong".to_string(), "wrong".to_string()];
        let bad_guess_response = try_connections_solution(&helper, token.as_str(), other_user_connections_game.slug.as_str(), bad_guess)
            .await
            .expect("Failed to attempt an incorrect guess for a connections game row");
        assert_eq!(bad_guess_response.row_name, None);
        assert_eq!(bad_guess_response.correct_guess, false);

        // Solve a row with a correct solution
        let good_guess = connection_categories[0].category_clues.clone();
        let good_guess_category_name = connection_categories[0].category_name.clone();
        let good_guess_response = try_connections_solution(&helper, token.as_str(), other_user_connections_game.slug.as_str(), good_guess)
            .await
            .expect("Failed to attempt a correct guess for a connections game row");
        assert_eq!(good_guess_response.row_name, Some(good_guess_category_name));
        assert_eq!(good_guess_response.correct_guess, true);
    }
}
