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

## First Time MongoDB Setup
Standalone `mongod` does not run by default with support for transactions, which this application uses.
Before running Pat, stop the standalone mongodb instance and restart it with the `replSet` argument:
```shell
mongod --port 27017 --dbpath /srv/mongodb/db0 --replSet rs0 --bind_ip localhost
```

If using mongo through systemd, set the following in the `/etc/mongod.conf` file:
```
replication:
  replSetName: "rs0"
```

Connect to the instance with the mongo shell and initiate the new replica set:
```js
rs.initiate()
```

## MongoDB Debugging
### MongoDB Error - ServerSelection
If db setup fails with a `ServerSelection` error where the MongoDB driver fails to find the replica set primary, try
the following:
- Append `?directConnectionn=true` to the connection string
  - e.g., `CONNECTION_STRING="mongodb://192.168.1.254:27017/?directConnection=true"`
  - This forces the driver to directly connect to the specified instance without checking the clusters replication
    topology. This issue was run into when setting up on Arch Linux
- If using `localhost` switch to `127.0.0.1` or `0.0.0.0`
  - Some drivers resolve `localhost` differently (ipv4 vs ipv6), being explicit gets rid of this issue
- Verify that there is a primary member on the machine running mongo
  - Use `mongosh` to enter a mongo shell on the machine running mongo
  - `rs.status()` to get replica set status, ensure that a member has `stateStr: 'PRIMARY'`
- Tell Mongo which replica set to use via the connection string
  - `CONNECTION_STRING="mongodb://192.168.1.254:27017/?replicaSet=rs0"`
