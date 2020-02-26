#!/bin/sh

# compile things
(cd backend && cargo build)
(cd frontend && cargo make build)
cargo install microserver

(cd backend && RUST_LOG="meals" cargo run) &
(cd frontend && cargo make build frontend && cargo make serve frontend) &
# wait for services to come up: curl in a loop?
echo "waiting is the hardest part"
sleep 5
(cd gauge-tests && gauge run specs)

# kill remaining cargo processes