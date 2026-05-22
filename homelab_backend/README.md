# PAT Backend

## Database
Pat uses [MongoDB](https://www.mongodb.com/) running locally at the default port of 27017.

## Debugging
Axum handler errors are sometimes overly generic and not helpful.
Adding the following decorator to an endpoint method give it more useful
error messages:
```rust
#[axum::debug_handler]
async fn my_endpoint_function() -> Response {  }
```

## Testing
Testing currently must be done on a single thread until I better figure out how to handle global state
```shell
cargo test -- --test-threads 1
```

## Development
Before any PR is made, make sure the following:
```shell
cargo fmt
cargo clippy
```

## Setup
The backend requires a `.env` in order to run, which should look like:
```
# This is the mongodb connection string
CONNECTION_STRING="mongodb://localhost:27017"
APP_SECRET="app_secret_string"
JWT_SECRET="jwt_secret"
JWT_EXPIRES_IN="jwt_expires_in"
JWT_MAX_AGE=604800
ADMIN_PASSWORD_HASH="admin_password_hash"
ADMIN_SALT="admin_salt"
```
The admin password hash and salt are used to automatically create an admin account when the app starts, to ensure
one exists for debugging. This will only occur if the application is running as debug and not release.
