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

//! Bootstrap Contract
//!
//! Smart contract used only once to register the Service Account.
//! The transaction that use this contract will be registered in the Genesis block, alone.
//!
//! This contract is not meant to be stored within the service account itself, instead is supposed
//! to be used once by the wasm machine bootstrap loader the first time the blockchain is started.
//!
//! ### Rules
//!
//! 0. There must be no other contracts registered on the Service Account!
//! 1. // TODO  Add an harcoded pk as CA and verifies the binary signature
//!

use sha256::digest_bytes;
use std::collections::HashMap;
use trinci_sdk::{AppContext, WasmError, WasmResult};

mod types;
use types::*;

trinci_sdk::app_export!(contract_registration);

const CONTRACTS_KEY: &str = "contracts";

#[inline]
fn calculate_contract_sha256_multihash(contract_data: &[u8]) -> String {
    let mut multihash = "1220".to_string();
    multihash.push_str(&digest_bytes(contract_data));
    multihash
}

/// Registration of the service contract.
fn contract_registration(ctx: AppContext, args: ContractRegistrationArgs) -> WasmResult<String> {
    // Load the assets list from the service account data
    let buf = trinci_sdk::load_data(CONTRACTS_KEY);
    if !buf.is_empty() {
        return Err(WasmError::new("blockchain already initialized"));
    }

    let mut contract_list = HashMap::new();

    let contract_data = ContractRegistrationData {
        name: args.name,
        version: args.version,
        creator: ctx.caller,
        description: args.description,
        url: args.url,
    };

    let contract_hash = calculate_contract_sha256_multihash(args.bin);

    // Add the new contract and check if the asset already exist
    contract_list.insert(contract_hash.clone(), contract_data);

    // Store the asset list
    trinci_sdk::store_account_data_mp!(CONTRACTS_KEY, &contract_list)?;

    // Store contract binary.
    // This is the field that the core will use for contract lookup.
    trinci_sdk::store_data(contract_hash.as_str(), args.bin);

    Ok(contract_hash)
}

#[cfg(test)]
mod tests {

    use crate::types::tests::{
        create_contract_registration_data, CALLER_ID, CONTRACT_MULTIHASH, SERVICE_ID,
    };
    use trinci_sdk::not_wasm::create_app_context;
    use trinci_sdk::{not_wasm, rmp_deserialize, rmp_serialize};

    use super::types::tests::create_contract_registration_args;
    use super::*;

    #[test]
    fn bootstrap_contract_registration_test() {
        let ctx = not_wasm::create_app_context(SERVICE_ID, CALLER_ID);

        let args = create_contract_registration_args();

        let expected = create_contract_registration_data();

        not_wasm::call_wrap(contract_registration, ctx, args).unwrap();

        let buf = trinci_sdk::load_data(CONTRACTS_KEY);
        let contracts: HashMap<String, ContractRegistrationData> = rmp_deserialize(&buf).unwrap();
        let contract_hash = hex::encode(CONTRACT_MULTIHASH);
        let contract = contracts.get(contract_hash.as_str()).unwrap().to_owned();

        assert_eq!(contract, expected);

        let contract = trinci_sdk::load_data(contract_hash.as_str());

        assert_eq!(contract, &[1u8, 2, 3]);
    }

    #[test]
    fn bootstrap_duplicate_contract_registration_test() {
        let ctx = create_app_context(SERVICE_ID, CALLER_ID);

        let mut map: HashMap<String, ContractRegistrationData> = HashMap::default();
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

        let args = create_contract_registration_args();

        let err = not_wasm::call_wrap(contract_registration, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "blockchain already initialized");
    }

    #[test]
    fn calculate_multihash_test() {
        let data = &[1u8, 2, 3];

        let data_multihash = calculate_contract_sha256_multihash(data);

        let expected = hex::encode(CONTRACT_MULTIHASH);

        assert_eq!(data_multihash, expected);
    }
}
