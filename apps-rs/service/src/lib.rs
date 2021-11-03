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

//! Service Contract
//!
//! Smart contract that is associated only with the Service Account
//! It allows to register Assets, Contracts, Oracles and Aliases,
//! it allows to retrieve Assets, Contracts, Oracles and Aliases data and
//! it allows to remove Aliases and retrieve Account Id from aliases
//!
//!
//! ### Rules
//!
//! 1. At the moment anyone can register assets, contracts, oracles
//! 2. Only the account id owner associated could remove an alias
//! 3. Anyone can retrieve an Asset, Contract, Oracle data
//! 4. Anyone can lookup for an Account Id starting from an Alias
//!

use sha256::digest_bytes;
use std::collections::BTreeMap;
use trinci_sdk::{rmp_serialize_named, AppContext, PackedValue, WasmError, WasmResult};

mod types;
use types::*;

trinci_sdk::app_export!(
    asset_registration,
    contract_registration,
    oracle_registration,
    get_asset_information,
    get_oracle_information,
    get_contract_information,
    alias_registration,
    alias_deletion,
    alias_lookup
);

const CONTRACTS_KEY: &str = "contracts";
const ASSETS_KEY: &str = "assets";
const ORACLES_KEY: &str = "oracles";
const ALIASES_KEY: &str = "aliases";

#[inline]
fn calculate_contract_sha256_multihash(contract_data: &[u8]) -> String {
    let mut multihash = "1220".to_string();
    multihash.push_str(&digest_bytes(contract_data));
    multihash
}

/// Registration of a new contract.
fn contract_registration(ctx: AppContext, args: ContractRegistrationArgs) -> WasmResult<String> {
    // Load the assets list from the service account data
    let buf = trinci_sdk::load_data(CONTRACTS_KEY);
    let mut contract_list: BTreeMap<String, ContractRegistrationData> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    let contract_data = ContractRegistrationData {
        name: args.name,
        version: args.version,
        creator: ctx.caller,
        description: args.description,
        url: args.url,
    };

    let contract_hash = calculate_contract_sha256_multihash(args.bin);

    // Check if a contract with the same name and same version already exists

    if contract_list
        .values()
        .any(|data| data.name == args.name && data.version == args.version)
    {
        return Err(WasmError::new(
            "contract with the same name and version already registered",
        ));
    }

    // Add the new contract and check if the asset already exist
    if contract_list
        .insert(contract_hash.clone(), contract_data)
        .is_some()
    {
        return Err(WasmError::new("contract already registered"));
    };

    // Store the asset list
    trinci_sdk::store_account_data_mp!(CONTRACTS_KEY, &contract_list)?;

    // Store contract binary.
    // This is the field that the core will use for contract lookup.
    trinci_sdk::store_data(contract_hash.as_str(), args.bin);

    Ok(contract_hash)
}

/// Registration of a new asset.
///
/// The account id associated to the new alias will be the caller id
fn asset_registration(ctx: AppContext, args: AssetRegistrationArgs) -> WasmResult<()> {
    // Load the assets list from the service account data
    let buf = trinci_sdk::load_data(ASSETS_KEY);
    let mut asset_list: BTreeMap<String, AssetRegistrationData> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    let asset_data = AssetRegistrationData {
        name: args.name,
        creator: ctx.caller,
        url: args.url,
        contract: args.contract,
    };

    // Check if an asset with the same name has already been registered
    if asset_list.values().any(|data| data.name == args.name) {
        return Err(WasmError::new("asset name already registered"));
    }

    // Add the new asset and check if the asset already exist
    if asset_list.insert(args.id.to_string(), asset_data).is_some() {
        return Err(WasmError::new("asset already registered"));
    };

    // Store the asset list
    trinci_sdk::store_account_data_mp!(ASSETS_KEY, &asset_list)?;

    Ok(())
}

/// Registration of a new oracle
///
/// The account id associated to the new alias will be the caller id
fn oracle_registration(ctx: AppContext, args: OracleRegistrationArgs) -> WasmResult<()> {
    // Load the oracles list from the service account data
    let buf = trinci_sdk::load_data(ORACLES_KEY);
    let mut oracle_list: BTreeMap<String, OracleRegistrationData> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    let oracle_data = OracleRegistrationData {
        name: args.name,
        creator: ctx.caller,
        description: args.description,
        url: args.url,
        contract: args.contract,
    };

    // Add the new oracle and check if the oracle already exists
    if oracle_list
        .insert(args.id.to_string(), oracle_data)
        .is_some()
    {
        return Err(WasmError::new("oracle already registered"));
    };

    // Store the oracle list
    trinci_sdk::store_account_data_mp!(ORACLES_KEY, &oracle_list)?;

    Ok(())
}

/// Registration of a new alias
///
/// The account id associated to the new alias will be the caller id
fn alias_registration(ctx: AppContext, args: AliasRegistrationArgs) -> WasmResult<()> {
    // Load the alias list from the service account data
    let buf = trinci_sdk::load_data(ALIASES_KEY);
    let mut alias_list: BTreeMap<String, String> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    // Add the new alias and check if the alias already exists
    if alias_list
        .insert(args.alias.to_string(), ctx.caller.to_string())
        .is_some()
    {
        return Err(WasmError::new("alias already registered"));
    };

    // Store the alias list
    trinci_sdk::store_account_data_mp!(ALIASES_KEY, &alias_list)?;

    Ok(())
}

/// Deletion of an alias
fn alias_deletion(ctx: AppContext, args: AliasDeletionArgs) -> WasmResult<()> {
    // Load the alias list from the service account data
    let buf = trinci_sdk::load_data(ALIASES_KEY);
    let mut alias_list: BTreeMap<String, String> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    // Delete the alias if the alias exists
    match alias_list.remove(args.alias) {
        Some(account_id) => {
            if account_id != ctx.caller {
                return Err(WasmError::new("not authorized"));
            }
        }
        None => return Err(WasmError::new("alias not registered")),
    };

    // Store the alias list
    trinci_sdk::store_account_data_mp!(ALIASES_KEY, &alias_list)?;

    Ok(())
}

/// Retrieves account id from alias
fn alias_lookup(_ctx: AppContext, args: AliasLookupArgs) -> WasmResult<String> {
    // Load the alias list from the service account data
    let buf = trinci_sdk::load_data(ALIASES_KEY);
    let alias_list: BTreeMap<String, String> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    if let Some(account_id) = alias_list.get(args.alias) {
        Ok(account_id.to_owned())
    } else {
        Err(WasmError::new("alias not registered"))
    }
}

/// Retrieve information about an asset.
fn get_asset_information(_ctx: AppContext, args: GetAssetArgs) -> WasmResult<PackedValue> {
    // Load the assets list from the service account data
    let buf = trinci_sdk::load_data(ASSETS_KEY);
    let asset_list: BTreeMap<String, AssetRegistrationData> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    let asset = match asset_list.get(args.asset_id) {
        Some(asset) => asset,
        None => return Err(WasmError::new("the asset is not registered")),
    };

    let buf = rmp_serialize_named(asset)?;

    Ok(PackedValue(buf))
}

/// Retrieve information about an oracle.
fn get_oracle_information(_ctx: AppContext, args: GetOracleArgs) -> WasmResult<PackedValue> {
    // Load the oracles list from the service account data
    let buf = trinci_sdk::load_data(ORACLES_KEY);
    let oracle_list: BTreeMap<String, OracleRegistrationData> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    let oracle = match oracle_list.get(args.oracle_id) {
        Some(oracle) => oracle,
        None => return Err(WasmError::new("the oracle is not registered")),
    };

    let buf = rmp_serialize_named(oracle)?;

    Ok(PackedValue(buf))
}

/// Retrieve information about a contract.
fn get_contract_information(_ctx: AppContext, args: GetContractArgs) -> WasmResult<PackedValue> {
    // Load the assets list from the service account data
    let buf = trinci_sdk::load_data(CONTRACTS_KEY);
    let contract_list: BTreeMap<String, ContractRegistrationData> =
        trinci_sdk::rmp_deserialize(&buf).unwrap_or_default();

    let contract = match contract_list.get(args.contract) {
        Some(contract) => contract,
        None => return Err(WasmError::new("the contract is not registered")),
    };

    let buf = rmp_serialize_named(contract)?;

    Ok(PackedValue(buf))
}

#[cfg(test)]
mod tests {

    use crate::types::tests::{
        create_alias_deletion_args, create_alias_lookup_args, create_alias_registration_args,
        create_asset_registration_args, create_asset_registration_data,
        create_contract_registration_data, create_get_asset_information_args,
        create_get_contract_information_args, create_get_oracle_information_args,
        create_oracle_registration_args, create_oracle_registration_data, ASSET_ID, CALLER_ID,
        CONTRACT_MULTIHASH, ORACLE_ID, SERVICE_ID, USER_ID,
    };
    use trinci_sdk::not_wasm::create_app_context;
    use trinci_sdk::{not_wasm, rmp_deserialize, rmp_serialize};

    use super::types::tests::create_contract_registration_args;
    use super::*;

    const ASSET_INFORMATION_HEX: &str = "84a46e616d65a64d79436f696ea763726561746f72d92e516d43616c6c657250467a6e78455836533331364d3479566d786478504236584e36336f626a46596b50364d4c71a375726cb7687474703a2f2f7777772e6d79636f696e2e6d6f6e6579a8636f6e7472616374c403010203";
    const CONTRACT_INFORMATION_HEX: &str = "85a46e616d65aa6d79636f6e7472616374a776657273696f6ea5302e312e30a763726561746f72d92e516d43616c6c657250467a6e78455836533331364d3479566d786478504236584e36336f626a46596b50364d4c71ab6465736372697074696f6ebc54686973206973206d7920706572736f6e616c20636f6e7472616374a375726cb9687474703a2f2f7777772e6d79636f6e74726163742e6f7267";
    const ORACLE_INFORMATION_HEX: &str = "85a46e616d65ab54696d65204f7261636c65a763726561746f72d92e516d43616c6c657250467a6e78455836533331364d3479566d786478504236584e36336f626a46596b50364d4c71ab6465736372697074696f6ed928546869732077696c6c20736179207468652074696d6520696e2074686520626c6f636b636861696ea375726cb5687474703a2f2f54696d654f7261636c652e6f7267a8636f6e7472616374c403010203";

    fn set_alias_data() {
        let mut map: BTreeMap<String, String> = BTreeMap::default();
        map.insert("MyCoolAlias".to_string(), CALLER_ID.to_string());
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, ALIASES_KEY, &buf);
    }
    fn set_oracle_data() {
        let mut map: BTreeMap<String, OracleRegistrationData> = BTreeMap::default();
        map.insert(ORACLE_ID.to_string(), create_oracle_registration_data());
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, ORACLES_KEY, &buf);
    }

    #[test]
    fn contract_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let args = create_contract_registration_args();

        let expected = create_contract_registration_data();

        not_wasm::call_wrap(contract_registration, ctx, args).unwrap();

        let buf = trinci_sdk::load_data(CONTRACTS_KEY);
        let contracts: BTreeMap<String, ContractRegistrationData> = rmp_deserialize(&buf).unwrap();
        let contract_hash = hex::encode(CONTRACT_MULTIHASH);
        let contract = contracts.get(contract_hash.as_str()).unwrap().to_owned();

        assert_eq!(contract, expected);

        let contract = trinci_sdk::load_data(contract_hash.as_str());

        assert_eq!(contract, &[1u8, 2, 3]);
    }

    #[test]
    fn duplicate_contract_registration_test() {
        let ctx = create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: BTreeMap<String, ContractRegistrationData> = BTreeMap::default();
        map.insert(
            hex::encode(CONTRACT_MULTIHASH),
            create_contract_registration_data(),
        );
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, CONTRACTS_KEY, &buf);
        not_wasm::set_account_data(
            SERVICE_ID,
            hex::encode(CONTRACT_MULTIHASH).as_str(),
            &[1u8, 2, 3],
        );

        let mut args = create_contract_registration_args();
        args.version = "3.4.5";

        let err = not_wasm::call_wrap(contract_registration, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "contract already registered");
    }

    #[test]
    fn duplicate_contract_name_version_registration_test() {
        let ctx = create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: BTreeMap<String, ContractRegistrationData> = BTreeMap::default();
        map.insert(
            hex::encode(CONTRACT_MULTIHASH),
            create_contract_registration_data(),
        );
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, CONTRACTS_KEY, &buf);
        not_wasm::set_account_data(
            SERVICE_ID,
            hex::encode(CONTRACT_MULTIHASH).as_str(),
            &[1u8, 2, 3],
        );

        let mut args = create_contract_registration_args();
        args.bin = &[5, 6, 7, 8];

        let err = not_wasm::call_wrap(contract_registration, ctx, args).unwrap_err();

        assert_eq!(
            err.to_string(),
            "contract with the same name and version already registered"
        );
    }

    #[test]
    fn get_contract_information_test() {
        let ctx = create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: BTreeMap<String, ContractRegistrationData> = BTreeMap::default();
        map.insert(
            hex::encode(CONTRACT_MULTIHASH),
            create_contract_registration_data(),
        );
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, CONTRACTS_KEY, &buf);
        not_wasm::set_account_data(
            SERVICE_ID,
            hex::encode(CONTRACT_MULTIHASH).as_str(),
            &[1u8, 2, 3],
        );

        let contract_hash = hex::encode(CONTRACT_MULTIHASH);
        let args = create_get_contract_information_args(&contract_hash);

        let res = not_wasm::call_wrap(get_contract_information, ctx, args).unwrap();

        assert_eq!(hex::encode(res.0), CONTRACT_INFORMATION_HEX);
    }

    #[test]
    fn asset_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let args = create_asset_registration_args();

        let expected = create_asset_registration_data();

        not_wasm::call_wrap(asset_registration, ctx, args).unwrap();

        let buf = trinci_sdk::load_data(ASSETS_KEY);
        let assets: BTreeMap<String, AssetRegistrationData> = rmp_deserialize(&buf).unwrap();
        let asset = assets.get(ASSET_ID).unwrap().to_owned();

        assert_eq!(asset, expected);
    }

    #[test]
    fn duplicate_asset_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: BTreeMap<String, AssetRegistrationData> = BTreeMap::default();
        map.insert(ASSET_ID.to_string(), create_asset_registration_data());
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, ASSETS_KEY, &buf);

        let mut args = create_asset_registration_args();
        args.name = "AnotherName";

        let err = not_wasm::call_wrap(asset_registration, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "asset already registered");
    }

    #[test]
    fn duplicate_name_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: BTreeMap<String, AssetRegistrationData> = BTreeMap::default();
        map.insert(ASSET_ID.to_string(), create_asset_registration_data());
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, ASSETS_KEY, &buf);

        let mut args = create_asset_registration_args();
        args.id = "AnotherId";

        let err = not_wasm::call_wrap(asset_registration, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "asset name already registered");
    }

    #[test]
    fn get_asset_information_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: BTreeMap<String, AssetRegistrationData> = BTreeMap::default();
        map.insert(ASSET_ID.to_string(), create_asset_registration_data());
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, ASSETS_KEY, &buf);

        let args = create_get_asset_information_args(ASSET_ID);

        let res = not_wasm::call_wrap(get_asset_information, ctx, args).unwrap();

        assert_eq!(hex::encode(res.0), ASSET_INFORMATION_HEX);
    }

    #[test]
    fn alias_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let args = create_alias_registration_args("MyCoolCoin");

        not_wasm::call_wrap(alias_registration, ctx, args).unwrap();

        let buf = trinci_sdk::load_data(ALIASES_KEY);
        let aliases: BTreeMap<String, String> = rmp_deserialize(&buf).unwrap();
        let account_id = aliases.get("MyCoolCoin").unwrap().to_owned();

        assert_eq!(account_id, CALLER_ID.to_string());
    }

    #[test]
    fn duplicate_alias_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: BTreeMap<String, String> = BTreeMap::default();
        map.insert("MyCoolCoin".to_string(), CALLER_ID.to_string());
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, ALIASES_KEY, &buf);

        let args = create_alias_registration_args("MyCoolCoin");

        let err = not_wasm::call_wrap(alias_registration, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "alias already registered");
    }

    #[test]
    fn deleting_alias_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        set_alias_data();

        let args = create_alias_deletion_args("MyCoolAlias");

        let _ = not_wasm::call_wrap(alias_deletion, ctx, args).unwrap();

        let buf = trinci_sdk::load_data(ALIASES_KEY);
        let aliases: BTreeMap<String, String> = rmp_deserialize(&buf).unwrap();
        let account_id = aliases.get("MyCoolAlias");

        assert!(account_id.is_none())
    }

    #[test]
    fn deleting_inexistent_alias_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        set_alias_data();

        let args = create_alias_deletion_args("MyCoolestAlias");

        let err = not_wasm::call_wrap(alias_deletion, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "alias not registered");

        let buf = trinci_sdk::load_data(ALIASES_KEY);
        let aliases: BTreeMap<String, String> = rmp_deserialize(&buf).unwrap();
        let account_id = aliases.get("MyCoolAlias").unwrap();

        assert_eq!(account_id, CALLER_ID);
    }

    #[test]
    fn unauthorized_deleting_alias_test() {
        let mut ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);
        ctx.caller = USER_ID;

        set_alias_data();

        let args = create_alias_deletion_args("MyCoolAlias");

        let err = not_wasm::call_wrap(alias_deletion, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");

        let buf = trinci_sdk::load_data(ALIASES_KEY);
        let aliases: BTreeMap<String, String> = rmp_deserialize(&buf).unwrap();
        let account_id = aliases.get("MyCoolAlias").unwrap();

        assert_eq!(account_id, CALLER_ID);
    }

    #[test]
    fn alias_lookup_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: BTreeMap<String, String> = BTreeMap::default();
        map.insert("MyCoolAsset".to_string(), ASSET_ID.to_string());
        let buf = rmp_serialize(&map).unwrap();

        not_wasm::set_account_data(SERVICE_ID, ALIASES_KEY, &buf);

        let args = create_alias_lookup_args("MyCoolAsset");

        let res = not_wasm::call_wrap(alias_lookup, ctx, args).unwrap();

        assert_eq!(res, ASSET_ID.to_string())
    }

    #[test]
    fn oracle_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let args = create_oracle_registration_args();

        let expected = create_oracle_registration_data();

        not_wasm::call_wrap(oracle_registration, ctx, args).unwrap();

        let buf = trinci_sdk::load_data(ORACLES_KEY);
        let oracles: BTreeMap<String, OracleRegistrationData> = rmp_deserialize(&buf).unwrap();
        let oracle = oracles.get(ORACLE_ID);
        let oracle = oracle.unwrap().to_owned();

        assert_eq!(oracle, expected);
    }

    #[test]
    fn duplicate_oracle_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        set_oracle_data();

        let args = create_oracle_registration_args();

        let err = not_wasm::call_wrap(oracle_registration, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "oracle already registered");
    }

    #[test]
    fn get_oracle_information_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        set_oracle_data();

        let args = create_get_oracle_information_args(ORACLE_ID);

        let res = not_wasm::call_wrap(get_oracle_information, ctx, args).unwrap();

        assert_eq!(hex::encode(res.0), ORACLE_INFORMATION_HEX);
    }

    #[test]
    fn calculate_multihash_test() {
        let data = &[1u8, 2, 3];

        let data_multihash = calculate_contract_sha256_multihash(data);

        let expected = hex::encode(CONTRACT_MULTIHASH);

        assert_eq!(data_multihash, expected);
    }
}
