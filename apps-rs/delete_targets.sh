#!/bin/bash
#
# Broadcast a cargo command to all the projects

CURRENT_DIR=$(pwd)

args=$(echo "$@" | sed 's/ -- /@/g')

PREV_IFS=$IFS
IFS="@"
read -ra items <<< "$args"
main="${items[0]}"
sub="${items[1]}"
IFS=$PREV_IFS

dirs=$(find ${CURRENT_DIR} -maxdepth 1 -type d)
for dir in $dirs; do
    rm -rf ${dir}/target/
done
