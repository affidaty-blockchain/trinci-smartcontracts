TRINCI Smart Contracts
======================

This folder contains the material to allow third party
development of smart contract applications using Rust.



## `apps-rs` directory 
 - The `apps-rs` directory contains the **Rust** contracts source code

### Create a new contract
```bash
$ ./create_new_contract.sh
```

Then insert the new contract name.

A project with the name provided will be created.

- Note requires `cargo generate`


### Launch the same cargo command for all the contracts:
```bash
$ ./cargo_broadcast.sh <COMMAND>
```

Example used to test all the contracts: 
```bash
$ ./cargo_broadcast.sh test
```

### Compile all the contracts with the rust installed on your computer
```
$ ./build_wasm.sh
```
 - requires:
   - `rust` with target `wasm32-unknown-unknown`
   - enabling unstable options flag

### Compile all the contracts with a docker image
```
$ ./build_wasm_docker.sh
```
 - Note: requires `docker`


## `registry` directory 
 - The `registry` directory contains the .wasm contracts


## `integration` directory 
 - The `integration` directory is a rust module that provide an enviroment for testing
 - Requires (because of the trinci-core dependency):
   - `rust` nightly toolchain
   - `clang`
   - `libclang-dev`
   - `protobuf-compiler`
 
