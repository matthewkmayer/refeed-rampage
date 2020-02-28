#!/bin/sh

# compile things
(cd backend && cargo build)

(cd backend && RUST_LOG="meals" cargo run) &
