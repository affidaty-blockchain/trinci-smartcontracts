ChangeLog
=========

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com).

Type of changes

* Added: for new features.
* Changed: for changes in existing functionality.
* Deprecated: for soon-to-be removed features.
* Removed: for now removed features.
* Fixed: for any bug fixes.
* Security: in case of vulnerabilities.

This project adheres to [Semantic Versioning](http://semver.org).

Given a version number MAJOR.MINOR.PATCH
* MAJOR incremented for incompatible API changes
* MINOR incremented for new functionalities
* PATCH incremented for bug fixes

Additional labels for pre-release metadata:
* alpha.x: internal development stage.
* beta.x: shipped version under testing.
* rc.x: stable release candidate.


0.2.0 - 10-11-2021
------------------

Added
* added notify method to test emit_data host function

Changed
* updated trinci sdk
* rust version 2021
* renamed alloc in rust sdk


0.1.2 - 13-10-2021
------------------

Changed
* Mint and Burn can be performed by a smart contract call
  if the transaction signer is authorized


0.1.2 - 26-10-2021
------------------

Added
* methods for deterministic tests
  * random_sequence,
  * return_hashmap,
  * get_time
* added mint method for integration test purpouse


0.1.1 - 13-09-2021
------------------

Changed
* test contract for wasm machine testing
