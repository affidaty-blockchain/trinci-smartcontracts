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


0.2.0 - 01-09-2021
------------------
Changed
* Now the fees are in thousandth (5 => 0.5% 1000 => 100%)

0.1.0 - 01-09-2021
------------------

Added
* `init` method - initializes the contract
* `get_info` method - returns the contract information
* `set_sellable` method - set the NFT sellable
* `set_price` method - set the NFT price
* `set_minimum_price` method - set the NFT minimum price
* `set_intermediary` method - set a new intermediary for the selling
* `buy` method - allows to buy the NFT
