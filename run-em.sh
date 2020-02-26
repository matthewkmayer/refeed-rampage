#!/bin/sh

# compile things
(cd backend && cargo build)
(cd frontend && cargo make build)
cargo install microserver

(cd backend && RUST_LOG="meals" cargo run) &
(cd frontend && cargo make build frontend && cargo make serve frontend) &
# wait for services to come up: curl in a loop?
echo "\n\nwaiting is the hardest part\n\n"
sleep 5
(cd gauge-tests && gauge run specs)

# kill remaining cargo processes
killall cargo-make
killall backend
killall microserver
