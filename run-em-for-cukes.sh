#!/bin/sh

# compile things
(cd backend && cargo build)
docker run --rm -p 8000:8000 amazon/dynamodb-local &
(cd backend && RUST_LOG="meals" cargo run) &
