# PAT Backend

## Database
To create a database and run migrations, use the following commands:
```shell
sqlx db create
sqlx migrate run
```

## Debugging
Axum handler errors are sometimes overly generic and not helpful.
Adding the following decorator to an endpoint method give it more useful
error messages:
```rust
#[axum::debug_handler]
async fn my_endpoint_function() -> Response {  }
```
