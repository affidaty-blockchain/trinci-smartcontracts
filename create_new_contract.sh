#!/bin/bash
#
# Create a new empty wasm project with the name requested
#
# Requires the installation of cargo-generate:
# https://github.com/cargo-generate/cargo-generate
# $ cargo install cargo-generate
#
# Note: the git repository for the template is temporary
# Note: this template works only in t2-contracts/rust 
#       (in order to work in other directory needs to be fixed trinci_sdk path)


echo -n "Project name: "
read -r name
while [ -z "$name" ]
do
    echo -n "Please insert the project name: "
    read -r name
done
name=${name// /-}


cargo generate --git git@github.com:affidaty-blockchain/trinci-contract-template.git --name $name
