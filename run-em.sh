#!/bin/sh

# compile things
(cd backend && cargo build)
(cd frontend && cargo build && npx wasm-pack build  --target web --out-name package --dev)

(cd backend && RUST_LOG="meals" cargo run) &
(cd frontend && cargo build && npx wasm-pack build  --target web --out-name package --dev && docker build . -t rrampage:local && docker run --name rrampage -p 8080:9090 rrampage:local) &
# wait for services to come up: curl in a loop?
echo "\n\nwaiting is the hardest part\n\n"
sleep 5

if r=npx gauge run gauge-tests/specs; then
    # Success.
    echo "woo r is $r"
else
    echo "nooooo r is $r"
fi

# kill remaining cargo processes
killall cargo-make
killall backend
docker kill rrampage
docker rm rrampage

exit $r