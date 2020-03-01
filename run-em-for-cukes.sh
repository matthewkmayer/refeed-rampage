#!/bin/sh

# compile things
(cd backend && cargo build)
docker run --rm -p 8000:8000 amazon/dynamodb-local &
sleep 3
docker ps -a
(cd backend && RUST_LOG="meals" AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY cargo run) &
