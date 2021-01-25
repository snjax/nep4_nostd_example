#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +nightly-2020-10-05 build --target wasm32-unknown-unknown --release -Z features=host_dep
mkdir -p ../../out
cp target/wasm32-unknown-unknown/release/*.wasm res

