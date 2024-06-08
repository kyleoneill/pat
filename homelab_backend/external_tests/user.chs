// TODO: Rename this file, make an actual folder structure to store it in, go over and re-do it slightly to make it
//       a more official test
// TODO: Uncomment commented assertions after ChimeraScript issues are resolved (string special char support, etc)

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
    ASSERT EQUALS (again.body) "Username '(username)' is already taken";

    // Auth as the user
    var token = POST /api/user/auth username=(username) password=(password);
    ASSERT STATUS (token) 201;

    // Get a user
    var user = GET /api/user/me authorization:(token.body);
    ASSERT STATUS (user) 200;
    ASSERT EQUALS (user.body.username) (username);

    // TODO: Update user

    case user_permissions() {
         case other_user_permissions() {
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
             ASSERT STATUS (deleteOther) 403 "Failed to assert that one user cannot delete another user";
         }
         case admin_user_permissions() {
            // Auth as an admin
            var adminRes = POST /api/user/auth username="admin" password="test";
            ASSERT STATUS (adminRes) 201;
         }
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

    // Make a request with valid auth
    var goodAuth = GET /api/user/me authorization:(res.body);
    ASSERT STATUS (goodAuth) 200;

    // Make a request with invalid auth
    var badAuth = GET /api/user/me authorization:"InvalidTokenString";
    ASSERT STATUS (badAuth) 404;

    // Auth as the user with an invalid password
    var badPasswordRes = POST /api/user/auth username=(username) password="wrongPassword";
    ASSERT STATUS (badPasswordRes) 404;

    // Auth as a user that does not exist
    var badRes = POST /api/user/auth username="idontexist" password=(password);
    ASSERT STATUS (badRes) 404;
    ASSERT EQUALS (badRes.body) "No such user with username 'idontexist', or invalid password";
}
