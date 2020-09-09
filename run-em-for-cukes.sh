#!/bin/sh

# compile things
(cd backend && cargo build)
docker run --rm -p 8000:8000 -d amazon/dynamodb-local@sha256:3bf539a420178b89f9dc696f5883cf889f11e381ffb25a7e18f01ba685f4f752 &
docker run -p 9000:9000 -d -e "MINIO_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE" -e "MINIO_SECRET_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY" minio/minio@sha256:33c2f3f08ef1c48a3c7d485f3511cc0a3945258eb4e077c4540ec700d7dbd4a3 server /data &
sleep 15
docker ps -a
(cd backend && RUST_LOG="backend" AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY cargo run) &
