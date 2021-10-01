#!/bin/bash
#
# Build all Rust wasm artefacts using Docker and "davxy/rust-builder" container.
#
# Building is performed using "docker" to prevent continuous sha changes when
# building on different machines because of different compiler versions and
# like.
#
# Build artefacts are cached in the host under the "build" folder.

COMMAND="./build_wasm.sh"

if [ -n "$1" ]; then
    COMMAND="$@"
fi

USER_ID=$(id -u $USER)
GROUP_ID=$(id -g $USER)

# WARNING
# THIS IS REQUIRED UNTIL THE CONTRACTS USE FILESYSTEM PATH TO RESOLVE TRINCI-SDK
# DEPENDENCY. IN THE FUTURE, WHEN WE'RE GOING TO USE THE CRATES REGISTRY, THIS
# WILL NOT BE AN ISSUE.
PROJECT_PATH=$(pwd)
PROJECT_PATH_GUEST="/trinci-contracts"
GUEST_WD="${PROJECT_PATH_GUEST}/"

# Build

docker run \
    --rm \
    -ti \
    -v $PROJECT_PATH:$PROJECT_PATH_GUEST \
    -w $GUEST_WD \
    -u $USER_ID:$GROUP_ID \
    davxy/rust-builder $COMMAND
    

# Push artifacts to the registry.

#files=$(find build -maxdepth 1 -type f | grep "\.wasm")
#(for file in $files; do
#    cp $file ../registry/$newfile
#done
