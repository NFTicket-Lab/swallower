#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
./target/release/node-swallower benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_swallower \
    --extrinsic '*' \
    --steps 100 \
    --repeat 2 \
    --raw \
    --output ./
