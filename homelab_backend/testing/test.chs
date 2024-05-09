[test]
case user_crud() {
    var username = LITERAL "test";
    var password = LITERAL "test";

    // Create and delete the user, if it already exists
    // TODO: This is a hack until ChimeraScript supports IF statements
    POST /api/user username=(username) password=(password);
    var token = POST /api/user/auth username=(username) password=(password);
    DELETE /api/user/me authorization:(token.body);

    case create_user_bad_args() {
        // Try to create a user with no args
        var noArgs = POST /api/user;
        ASSERT STATUS (noArgs) 422;

        // Try to create a user with bad args
        var badArgs = POST /api/user foo=5 bar="foobar";
        ASSERT STATUS (badArgs) 422;

        // Try to create a user without a password
        var noPassword = POST /api/user username=(username);
        ASSERT STATUS (noPassword) 422;
    }

    // Create a user
    var user = POST /api/user username=(username) password=(password);
    ASSERT STATUS (user) 201;
    ASSERT EQUALS (user.body.username) (username);

    // Try to create the user again, assert we get an error
    var again = POST /api/user username=(username) password=(password);
    ASSERT STATUS (again) 400;
    // ASSERT EQUALS (again.body) "Username '(username)' is already taken";

    // Auth as the user
    var token = POST /api/user/auth username=(username) password=(password);
    ASSERT STATUS (token) 201;

    // TODO: Get user
    // TODO: Need to create a `GET /user/me` endpoint

    // TODO: Update user

    case invalid_perms() {
        // Create a second user, don't assert in case they already exist
        POST /api/user username="dummy" password="dummy";

        // Auth as a second user
        var authUserTwo = POST /api/user/auth username="dummy" password="dummy";
        ASSERT STATUS (authUserTwo) 201;

        // Get the first user as the second
        var otherUser = GET /api/user?username=(username) authorization:(authUserTwo.body);
        ASSERT STATUS (otherUser) 200;

        // TODO Update other, fail

        // Try to delete another user
        var deleteOther = DELETE /api/user/(otherUser.body.id) authorization:(authUserTwo.body);
        ASSERT STATUS (deleteOther) 403;
    }

    // Delete the user
    var deleteRes = DELETE /api/user/me authorization:(token.body);
    ASSERT STATUS (deleteRes) 200;

    // TODO: Delete another user as admin
}

[test]
case auth_user() {
    var username = LITERAL "test";
    var password = LITERAL "test";

    // Verify that our user exists
    POST /api/user username=(username) password=(password);

    // Auth as the user
    var res = POST /api/user/auth username=(username) password=(password);
    ASSERT STATUS (res) 201;

    // Auth as the user with an invalid password
    var badPasswordRes = POST /api/user/auth username=(username) password="wrongPassword";
    ASSERT STATUS (badPasswordRes) 404;

    // Auth as a user that does not exist
    var badRes = POST /api/user/auth username="idontexist" password=(password);
    ASSERT STATUS (badRes) 404;
    // ASSERT EQUALS (badRes.body) "No such user with username 'idontexist'";
}

[test]
case admin_user() {
    // Auth as an admin
    var adminRes = POST /api/user/auth username="admin" password="test";
    ASSERT STATUS (adminRes) 201;
}

// NEED TO TEST TOKENS
// Try to use a token meant for user-1 as a user-2
// Completely invalid random nonsense as a token
// Valid token works
