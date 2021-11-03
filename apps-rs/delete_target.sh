#!/bin/bash
#
# Delete target directory from contracts

CURRENT_DIR=$(pwd)

dirs=$(find ${CURRENT_DIR} -maxdepth 1 -type d)

for dir in $dirs; do
    rm -rf "${dir}/target"	
done
