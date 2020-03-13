#!/bin/sh

(cd frontend && cargo build && npx wasm-pack build  --target web --out-name package --dev)
sleep 15
# mock backend:
docker run -it -v $PWD/tck/duty.yaml:/duty.yaml -v $PWD/tck/responses:/responses --name duty -d gomicro/duty
docker run -it -v $PWD/tck/routes.yaml:/routes.yaml -p 3030:4567 --link duty:duty -d gomicro/avenues

(cd frontend && cargo build && npx wasm-pack build  --target web --out-name package --dev && docker build . -t rrampage:local && docker run --name rrampage -p 8080:9090 rrampage:local) &

# wait for services to come up: curl in a loop?
echo "\n\nwaiting is the hardest part\n\n"
sleep 5
