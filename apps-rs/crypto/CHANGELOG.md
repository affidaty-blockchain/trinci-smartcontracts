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


0.2.1 - 07-10-2021
------------------

Added
* merkle_tree_verify method with merkle tree multiproof verify


0.2.0 - 10-09-2021
------------------

Changed
* updated to PublicKey new implementation


0.1.0 - 01-09-2021
------------------

Added
* `verify` method - checks if the data signature is valid
  Calls the hf_verify
* `hash` method - calculate the hash of the data passed
  Supports SHA256, SHA384, SHA512