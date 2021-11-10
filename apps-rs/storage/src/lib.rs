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

//! Storage Manager.
//!
//! Smart contract allowing the account owner to load, store and delete
//! arbitrary data into the account.
//!
//! Furthermore the contract contains support methods to transfer an asset that
//! is compliant with the TAI assets interface.
//!
//! ### Rules
//!
//! 1. Only the account owner is allowed to invoke the methods.

use trinci_sdk::{AppContext, PackedValue, WasmError, WasmResult};

mod types;
use types::*;

trinci_sdk::app_export!(load_data, store_data, remove_data, balance, transfer);

/// Remove data into the account data `key` field
pub fn remove_data(ctx: AppContext, args: RemoveDataArgs) -> WasmResult<()> {
    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }
    trinci_sdk::remove_data(args.key);
    Ok(())
}

/// Store arbitrary data into the account data `key` field
pub fn store_data(ctx: AppContext, args: StoreDataArgs) -> WasmResult<()> {
    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }
    trinci_sdk::store_data(args.key, args.data);
    Ok(())
}

/// Load arbitrary data from the account data `key` field
pub fn load_data(ctx: AppContext, args: LoadDataArgs) -> WasmResult<PackedValue> {
    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }

    Ok(PackedValue(trinci_sdk::load_data(args.key)))
}

/// Call the host function hf_transfer to transfer an *amount* of *asset* from the *caller account* to the *dest *account*
pub fn transfer(ctx: AppContext, args: TransferArgs) -> WasmResult<()> {
    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }

    trinci_sdk::log!(
        "Transfering {} {}\nfrom: {}\nto: {}",
        args.units,
        args.asset,
        ctx.owner,
        args.to
    );

    trinci_sdk::asset_transfer(ctx.caller, args.to, args.asset, args.units)
}

/// Call the host function hf_balance to get the caller account balance
fn balance(ctx: AppContext, args: BalanceArgs) -> WasmResult<u64> {
    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }

    trinci_sdk::asset_balance(args.asset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trinci_sdk::{not_wasm, tai::Asset};

    const CALLER_ID: &str = "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";
    const ASSET_ID: &str = "QmSCRCPFznxEX6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq";
    const DATA_KEY: &str = "data";

    #[test]
    fn store_data_test() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        let args = StoreDataArgs {
            key: DATA_KEY,
            data: &[1, 2, 3],
        };

        not_wasm::call_wrap(store_data, ctx, args).unwrap();

        let data = not_wasm::get_account_data(CALLER_ID, DATA_KEY);
        assert_eq!(data, &[1, 2, 3]);
    }

    #[test]
    fn unauthorized_store_data() {
        let ctx = not_wasm::create_app_context(CALLER_ID, "DummyUser");

        let args = StoreDataArgs {
            key: DATA_KEY,
            data: &[1, 2, 3],
        };

        let err = not_wasm::call_wrap(store_data, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn load_data_test() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        not_wasm::set_account_data(CALLER_ID, DATA_KEY, &[1, 2, 3]);
        let args = LoadDataArgs { key: DATA_KEY };

        let data = not_wasm::call_wrap(load_data, ctx, args).unwrap();

        assert_eq!(*data, &[1, 2, 3]);
    }

    #[test]
    fn load_data_not_existing_key() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        let args = LoadDataArgs { key: "config" };

        let data = not_wasm::call_wrap(load_data, ctx, args).unwrap();

        assert!(data.is_empty());
    }

    #[test]
    fn unauthorized_load_data() {
        let ctx = not_wasm::create_app_context(CALLER_ID, "DummyUser");
        not_wasm::set_account_data(CALLER_ID, DATA_KEY, &[1, 2, 3]);
        let args = LoadDataArgs { key: DATA_KEY };

        let err = not_wasm::call_wrap(load_data, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn remove_data_test() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        not_wasm::set_account_data(CALLER_ID, DATA_KEY, &[1, 2, 3]);
        let args = RemoveDataArgs { key: DATA_KEY };

        not_wasm::call_wrap(remove_data, ctx, args).unwrap();

        let data = not_wasm::get_account_data(CALLER_ID, DATA_KEY);
        assert!(data.is_empty());
    }

    #[test]
    fn unauthorized_remove_data() {
        let ctx = not_wasm::create_app_context(CALLER_ID, "DummyUser");
        not_wasm::set_account_data(CALLER_ID, DATA_KEY, &[1, 2, 3]);
        let args = RemoveDataArgs { key: DATA_KEY };

        let err = not_wasm::call_wrap(remove_data, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn balance_valid_asset() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(3));
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        let args = BalanceArgs { asset: ASSET_ID };

        let units = not_wasm::call_wrap(balance, ctx, args).unwrap();

        assert_eq!(units, 3);
    }

    #[test]
    fn balance_asset_not_found() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(3));
        not_wasm::set_contract_method("DummyAsset", "balance", not_wasm::asset_balance);
        let args = BalanceArgs {
            asset: "DummyAsset",
        };

        let units = not_wasm::call_wrap(balance, ctx, args).unwrap();

        assert_eq!(units, 0);
    }

    #[test]
    fn balance_not_authorized() {
        let ctx = not_wasm::create_app_context(CALLER_ID, "DummyUser");
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(3));
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        let args = BalanceArgs { asset: ASSET_ID };

        let err = not_wasm::call_wrap(balance, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn transfer_asset_success() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(9));
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);
        let args = TransferArgs {
            to: "abcdef",
            asset: ASSET_ID,
            units: 3,
        };

        not_wasm::call_wrap(transfer, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.units, 6);
        let asset: Asset = not_wasm::get_account_asset_gen("abcdef", ASSET_ID);
        assert_eq!(asset.units, 3);
    }

    #[test]
    fn transfer_asset_not_found() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(9));
        not_wasm::set_contract_method("DummyAsset", "transfer", not_wasm::asset_transfer);
        let args = TransferArgs {
            to: "abcdef",
            asset: "DummyAsset",
            units: 3,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "error during transfer");
    }

    #[test]
    fn transfer_asset_not_authorized() {
        let ctx = not_wasm::create_app_context(CALLER_ID, "DummyUser");
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(9));
        let args = TransferArgs {
            to: "abcdef",
            asset: "DummyAsset",
            units: 3,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }
}
