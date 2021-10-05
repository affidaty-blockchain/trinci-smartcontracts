#!/bin/bash

# 1. Build all Rust wasm artefacts and place them into the `build` dir.
# 2. Optimize wasm artefacts.

CURRENT_DIR=$(pwd)
BUILD_DIR="${CURRENT_DIR}/build"

# Release build

CARGO_CMD="build --release --target wasm32-unknown-unknown --out-dir ${BUILD_DIR} -Z unstable-options"

./cargo_broadcast.sh $CARGO_CMD

# Optimization

files=$(find ${BUILD_DIR} -maxdepth 1 -type f | grep "\.wasm")

for file in $files; do
    wasm-opt --strip $file -o $file
    sha256sum $file
	cp $file ../registry/
done
