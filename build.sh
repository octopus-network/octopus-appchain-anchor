#!/bin/bash
cargo doc -p appchain-anchor --no-deps
RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release

mkdir -p res
cp target/wasm32-unknown-unknown/release/*.wasm ./res/

if [ "$1" == "test" ]; then
    if [ "$2" == "" ]; then
        RUST_BACKTRACE=1 cargo test --tests -- --nocapture
    else
        RUST_BACKTRACE=1 cargo test $2 -- --nocapture
    fi
fi
