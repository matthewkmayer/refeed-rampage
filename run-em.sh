#!/bin/sh

# compile things
(cd backend && cargo build)
(cd frontend && cargo build && npx wasm-pack build  --target web --out-name package --dev)
docker run --rm -p 8000:8000 -d amazon/dynamodb-local@sha256:3bf539a420178b89f9dc696f5883cf889f11e381ffb25a7e18f01ba685f4f752 &
docker run -p 9000:9000 -d -e "MINIO_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE" -e "MINIO_SECRET_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY" minio/minio@sha256:33c2f3f08ef1c48a3c7d485f3511cc0a3945258eb4e077c4540ec700d7dbd4a3 server /data &
sleep 15
(cd backend && RUST_LOG="backend" AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY cargo run) &
(cd frontend && cargo build && npx wasm-pack build  --target web --out-name package --dev && docker build . -t rrampage:local && docker run --name rrampage -p 8080:9090 rrampage:local) &

# wait for services to come up: curl in a loop?
echo "\n\nwaiting is the hardest part\n\n"
sleep 5
