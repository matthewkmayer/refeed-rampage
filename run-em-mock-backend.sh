#!/bin/sh

# compile things
docker run --rm -p 8000:8000 amazon/dynamodb-local &
(cd frontend && cargo build && npx wasm-pack build  --target web --out-name package --dev)
sleep 15
# mock backend:
docker run -it -v $PWD/tck/duty.yaml:/duty.yaml -v $PWD/tck/responses:/responses --name duty -d gomicro/duty
docker run -it -v $PWD/tck/routes.yaml:/routes.yaml -p 3030:4567 --link duty:duty -d gomicro/avenues

# (cd backend && RUST_LOG="backend" AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY cargo run) &
(cd frontend && cargo build && npx wasm-pack build  --target web --out-name package --dev && docker build . -t rrampage:local && docker run --name rrampage -p 8080:9090 rrampage:local) &

# wait for services to come up: curl in a loop?
echo "\n\nwaiting is the hardest part\n\n"
sleep 5
