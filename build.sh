#!/bin/bash
cargo fmt --all
cargo doc -p appchain-anchor --no-deps
RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release
if [ ! -d "out" ]; then
    mkdir -p "out"
fi
if [ ! -d "res" ]; then
    mkdir -p "res"
fi
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
cp target/wasm32-unknown-unknown/release/appchain_anchor.wasm ./out/main.wasm
