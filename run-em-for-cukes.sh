#!/bin/sh

# compile things
(cd backend && cargo build)
docker-compose up -d
sleep 15
docker ps -a
(cd backend && RUST_LOG="backend" AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY cargo run) &
