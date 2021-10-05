// This file is part of TRINCI.
//
// Copyright (C) 2021 Affidaty Spa.
//
// TRINCI is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// TRINCI is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with TRINCI. If not, see <https://www.gnu.org/licenses/>.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub use trinci_sdk::tai::{
    Asset, AssetLockArgs as LockArgs, AssetTransferArgs as TransferArgs, LockPrivilege, LockType,
};

/// Initialization arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct InitArgs<'a> {
    /// Asset name.
    pub name: &'a str,
    /// Accounts allowed to `mint` and `burn`.
    pub authorized: Vec<&'a str>,
    /// Asset description.
    pub description: &'a str,
    /// Asset public url.
    pub url: &'a str,
    /// Max mintable units.
    pub max_units: u64,
}

/// Mint method arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct MintArgs<'a> {
    /// Destination account.
    pub to: &'a str,
    /// Number of units.
    pub units: u64,
}

/// Burn method arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct BurnArgs<'a> {
    /// Source account.
    pub from: &'a str,
    /// Number of units.
    pub units: u64,
}

/// Asset configuration.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct AssetConfig<'a> {
    /// Asset name.
    pub name: &'a str,
    /// Account that has invoked the `init` method.
    pub creator: &'a str,
    /// Accounts allowed to `mint` and `burn`.
    pub authorized: Vec<&'a str>,
    /// Asset description.
    pub description: &'a str,
    /// Asset public url.
    pub url: &'a str,
    /// Max mintable units.
    pub max_units: u64,
    /// Minted units (dynamic).
    pub minted: u64,
    /// Burned units (dynamic).
    pub burned: u64,
}

/// Struct to delegate not-owner account to perform payment
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Delegation<'a> {
    /// The delegate account (will be the transfer caller)
    pub delegate: &'a str,
    /// Amount of asset to allow the transfer
    pub units: u64,
    /// Destination account for the transfer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<&'a str>,
}

/// Add Delegation method arguments
pub type AddDelegationArgs<'a> = Delegation<'a>;

/// Arguments for the asset delegate `transfer` to store in memory.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DelegateTransferArgs<'a> {
    pub from: &'a str,
    pub units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<&'a str>,
}

/// Delegations data field
pub type DelegationsData<'a> = HashMap<&'a str, Vec<DelegateTransferArgs<'a>>>;

#[cfg(test)]
pub(crate) mod tests {

    use super::*;
    use trinci_sdk::{rmp_deserialize, rmp_serialize};

    const DESTINATION_ID: &str = "QDestEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfqxx";

    const BURN_ARGS_HEX: &str = "92a36162632a";
    const MINT_ARGS_HEX: &str = "92a378797a2a";
    const TRANSFER_ARGS_HEX: &str = "93a3616263a378797a2a";
    const INIT_ARGS_HEX: &str = "95a346434b93a94163636f756e745f31a94163636f756e745f32a94163636f756e745f33b053696d706c65207465737420636f696eae687474703a2f2f666f6f2e626172cd03e8";
    const ASSET_CONFIG_HEX: &str = "98a346434ba94d794163636f756e7493a94163636f756e745f31a94163636f756e745f32a94163636f756e745f33b053696d706c65207465737420636f696eae687474703a2f2f666f6f2e626172cd03e80000";
    const ADD_DELEGATION_ARGS_HEX: &str = "93d92e514465737445514c64663568374b59626a4650754853526b325350676458724a57466835573639364850667178782ad92e514465737445514c64663568374b59626a4650754853526b325350676458724a5746683557363936485066717878";
    const ADD_DELEGATION_NONE_ARGS_HEX: &str = "92d92e514465737445514c64663568374b59626a4650754853526b325350676458724a57466835573639364850667178782a";

    pub fn create_init_args() -> InitArgs<'static> {
        InitArgs {
            name: "FCK",
            description: "Simple test coin",
            url: "http://foo.bar",
            max_units: 1000,
            authorized: vec!["Account_1", "Account_2", "Account_3"],
        }
    }

    pub fn create_asset_config() -> AssetConfig<'static> {
        AssetConfig {
            name: "FCK",
            description: "Simple test coin",
            url: "http://foo.bar",
            max_units: 1000,
            creator: "MyAccount",
            authorized: vec!["Account_1", "Account_2", "Account_3"],
            minted: 0,
            burned: 0,
        }
    }
    fn create_transfer_args() -> TransferArgs<'static> {
        TransferArgs {
            from: "abc",
            to: "xyz",
            units: 42,
        }
    }
    fn create_mint_args() -> MintArgs<'static> {
        MintArgs {
            to: "xyz",
            units: 42,
        }
    }
    fn create_burn_args() -> BurnArgs<'static> {
        BurnArgs {
            from: "abc",
            units: 42,
        }
    }

    #[test]
    fn mint_args_serialize() {
        let data = create_mint_args();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), MINT_ARGS_HEX);
    }

    #[test]
    fn mint_args_deserialize() {
        let expected = create_mint_args();
        let buf = hex::decode(MINT_ARGS_HEX).unwrap();

        let data: MintArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn burn_args_serialize() {
        let data = create_burn_args();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), BURN_ARGS_HEX);
    }

    #[test]
    fn burn_args_deserialize() {
        let expected = create_burn_args();
        let buf = hex::decode(BURN_ARGS_HEX).unwrap();

        let data: BurnArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn transfer_args_serialize() {
        let data = create_transfer_args();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), TRANSFER_ARGS_HEX);
    }

    #[test]
    fn transfer_args_deserialize() {
        let expected = create_transfer_args();

        let buf = hex::decode(TRANSFER_ARGS_HEX).unwrap();

        let data: TransferArgs = rmp_deserialize(&buf).unwrap();
        assert_eq!(data, expected);
    }

    #[test]
    fn init_args_serialize() {
        let data = create_init_args();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), INIT_ARGS_HEX);
    }

    #[test]
    fn init_args_deserialize() {
        let expected = create_init_args();

        let buf = hex::decode(INIT_ARGS_HEX).unwrap();

        let data: InitArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn asset_config_serialize() {
        let data = create_asset_config();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ASSET_CONFIG_HEX);
    }

    #[test]
    fn asset_config_deserialize() {
        let expected = create_asset_config();

        let buf = hex::decode(ASSET_CONFIG_HEX).unwrap();

        let data: AssetConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn add_delegation_args_serialize() {
        let data = AddDelegationArgs {
            delegate: DESTINATION_ID,
            to: Some(DESTINATION_ID),
            units: 42,
        };

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ADD_DELEGATION_ARGS_HEX);
    }

    #[test]
    fn add_delegation_args_deserialize() {
        let expected = AddDelegationArgs {
            delegate: DESTINATION_ID,
            to: Some(DESTINATION_ID),
            units: 42,
        };

        let buf = hex::decode(ADD_DELEGATION_ARGS_HEX).unwrap();

        let data: AddDelegationArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn add_delegation_args_with_none_serialize() {
        let data = AddDelegationArgs {
            delegate: DESTINATION_ID,
            to: None,
            units: 42,
        };

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ADD_DELEGATION_NONE_ARGS_HEX);
    }

    #[test]
    fn add_delegation_args_with_none_deserialize() {
        let expected = AddDelegationArgs {
            delegate: DESTINATION_ID,
            to: None,
            units: 42,
        };

        let buf = hex::decode(ADD_DELEGATION_NONE_ARGS_HEX).unwrap();

        let data: AddDelegationArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }
}
