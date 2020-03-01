## Backend with warp

https://github.com/seanmonstar/warp

Run with `cargo run` to start the web server at http://localhost:3030 .

To see logs: `RUST_LOG="meals=debug" cargo run` .

## Tests

API level:

`cargo test --test cucumber`

Local Dynamo: `docker run --rm -p 8000:8000 amazon/dynamodb-local`
