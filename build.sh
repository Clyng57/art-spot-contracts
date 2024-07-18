#!/bin/sh
set -e
cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/spot_token.wasm compiled/
cp target/wasm32-unknown-unknown/release/registrar.wasm compiled/