#!/bin/bash
#
# Broadcast a cargo command to all the projects

CURRENT_DIR=$(pwd)
BUILD_DIR="${CURRENT_DIR}/build"

export CARGO_HOME="${BUILD_DIR}/cargo"
export CARGO_TARGET_DIR="${BUILD_DIR}/target"

args=$(echo "$@" | sed 's/ -- /@/g')

PREV_IFS=$IFS
IFS="@"
read -ra items <<< "$args"
main="${items[0]}"
sub="${items[1]}"
IFS=$PREV_IFS

dirs=$(find ${CURRENT_DIR} -maxdepth 1 -type d)
for dir in $dirs; do
    manifest="${dir}/Cargo.toml"
    if [ -f $dir/Cargo.toml ]; then
        echo PROCESSING $manifest
        cargo $main --manifest-path $manifest -- $sub
    fi
done
