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

//! Escrow
//!
//! ### Rules
//!
//! 1. Initialization is performed by the escrow account owner.
//! 2. Initialization checks if the account owns at least the quantity required by the escrow init arguments.
//! 3. Once initialized the asset under escrow is locked by the contract and
//!    operations on it can be performed only by passing through the contract
//!    methods.
//! 4. Balance returns the total amount under escrow (not the total amount on
//!    the account, that is temporary hidden).
//! 5. Balance can be invoked only by customer, merchants or guarantor.
//! 6. Update method, to resolve the escrow, can be invoked only by the guarantor.
//!
//! ### Warning
//!
//! To work correctly the contract shall be used with an asset respecting the
//! **TAI** interface.  In particular it is assumed that the asset `lock` type
//! `Contract` is implemented to not be bypassed via direct asset contract call.
//!

use std::cmp;

use trinci_sdk::{tai::LockType, AppContext, PackedValue, WasmError, WasmResult};

mod types;
use types::*;

trinci_sdk::app_export!(init, balance, get_info, update);

const CONFIG_KEY: &str = "config";
const STATUS_KEY: &str = "status";

#[inline]
fn get_status() -> WasmResult<EscrowStatus> {
    let buf = trinci_sdk::load_data(STATUS_KEY);
    trinci_sdk::rmp_deserialize(&buf).map_err(|_err| WasmError::new("contract not initialized"))
}

#[inline]
fn set_status(status: EscrowStatus) -> WasmResult<()> {
    let buf = trinci_sdk::rmp_serialize(&status)?;
    trinci_sdk::store_data(STATUS_KEY, &buf);
    Ok(())
}

/// Initialize contract status.
///
/// The caller shall be the asset account owner.
fn init(ctx: AppContext, args: InitArgs) -> WasmResult<()> {
    if get_status().is_ok() {
        return Ok(());
    }

    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }

    // Withdraw lock for the asset under escrow.
    trinci_sdk::asset_lock(args.asset, ctx.owner, trinci_sdk::tai::LockType::Withdraw)?;

    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &args)?;

    set_status(EscrowStatus::Open)
}

fn close_on_success(escrow_id: &str, config: &EscrowConfig) -> WasmResult<()> {
    // Sanity checks
    let total_amount = config.merchants.iter().fold(0, |acc, (_, &val)| acc + val);
    let balance: u64 = trinci_sdk::asset_balance(config.asset)?;
    if balance < total_amount {
        return Err(WasmError::new("insufficient funds"));
    }

    for (&to_id, &amount) in config.merchants.iter() {
        trinci_sdk::log!(
            "Transfering {} {} to merchant {}",
            amount,
            config.asset,
            to_id
        );
        trinci_sdk::asset_transfer(escrow_id, to_id, config.asset, amount)?;
    }
    Ok(())
}

fn close_on_fail(escrow_id: &str, config: &EscrowConfig) -> WasmResult<()> {
    // Get the total amount
    let amount = config.merchants.iter().fold(0, |acc, (_, &val)| acc + val);
    let balance: u64 = trinci_sdk::asset_balance(config.asset)?;

    let amount = cmp::min(amount, balance);

    trinci_sdk::log!(
        "Returning {} {} to customer {}",
        amount,
        config.asset,
        config.customer
    );

    trinci_sdk::asset_transfer(escrow_id, config.customer, config.asset, amount)
}

/// Finalize the escrow account.
///
/// Method UPDATE called from the smart contract guarantor to check
/// if conditions are satisfied and the pay
fn update(ctx: AppContext, args: UpdateArgs) -> WasmResult<()> {
    let success = args.status.to_uppercase();
    if success != "OK" && success != "KO" {
        return Err(WasmError::new("bad `status` value. Shall be OK or KO"));
    }

    let status = get_status()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: EscrowConfig =
        trinci_sdk::rmp_deserialize(&buf).map_err(|_| WasmError::new("bad account `data`"))?;

    if ctx.caller != config.guarantor {
        return Err(WasmError::new("not authorized"));
    }

    if status != EscrowStatus::Open {
        trinci_sdk::log!("request rejected (status = {})", status);
        return Err(WasmError::new("closed escrow"));
    }

    // Unlock the asset
    trinci_sdk::asset_lock(config.asset, ctx.owner, trinci_sdk::tai::LockType::None)?;

    let status = if success == "OK" {
        close_on_success(ctx.owner, &config)?;
        EscrowStatus::Success
    } else {
        close_on_fail(ctx.owner, &config)?;
        EscrowStatus::Failure
    };

    set_status(status)
}

/// Get caller account asset balance.
///
/// Caller shall be in the merchants list.
fn balance(ctx: AppContext, args: BalanceArgs) -> WasmResult<u64> {
    get_status()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let data: EscrowConfig = trinci_sdk::rmp_deserialize(&buf)?;

    let mut auth_list = vec![data.customer, data.guarantor];
    data.merchants
        .iter()
        .for_each(|(&merchant, _)| auth_list.push(merchant));

    let authorized = auth_list.iter().find(|&&elem| elem == ctx.caller);
    match authorized {
        Some(_) => {
            trinci_sdk::asset_lock(args.asset, ctx.owner, LockType::None)?;
            let balance = trinci_sdk::asset_balance(args.asset);
            trinci_sdk::asset_lock(args.asset, ctx.owner, LockType::Full)?;
            balance
        }
        None => Err(WasmError::new("not authorized")),
    }
}

/// Get the contract information
///
/// Returns a structure that cointains config, amount and status.
fn get_info(ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    get_status()?;

    // Load the contract config
    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: EscrowConfig = trinci_sdk::rmp_deserialize(&buf)
        .map_err(|_err| WasmError::new("contract not initialized"))?;

    let mut auth_list = vec![config.customer, config.guarantor];
    config
        .merchants
        .iter()
        .for_each(|(&merchant, _)| auth_list.push(merchant));

    let authorized = auth_list.iter().find(|&&elem| elem == ctx.caller);

    trinci_sdk::asset_lock(config.asset, ctx.owner, LockType::None)?;

    let amount = match authorized {
        Some(_) => trinci_sdk::asset_balance(config.asset),
        None => Err(WasmError::new("not authorized")),
    }?;
    trinci_sdk::asset_lock(config.asset, ctx.owner, LockType::Full)?;

    let status = get_status()?;

    let info = EscrowInfo {
        config,
        amount,
        status: &status.to_string(),
    };

    let buf = trinci_sdk::rmp_serialize_named(&info)?;
    Ok(PackedValue(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use trinci_sdk::{
        not_wasm, rmp_deserialize, rmp_serialize,
        tai::{Asset, LockPrivilege},
    };

    impl PartialEq for EscrowInfo<'_> {
        fn eq(&self, other: &EscrowInfo) -> bool {
            self.config == other.config && self.amount == other.amount
        }
    }
    impl Eq for EscrowInfo<'_> {}

    const ESCROW_ID: &str = "QmT48ijWd7RqEzdV3gKjqXN1kGBgYxFWsxajjguLkyTjy7";
    const ASSET_ID: &str = types::tests::ASSET_ID;
    const GUARANTOR_ID: &str = types::tests::GUARANTOR_ID;
    const CUSTOMER_ID: &str = types::tests::CUSTOMER_ID;
    const MERCHANT1_ID: &str = types::tests::MERCHANT1_ID;
    const MERCHANT2_ID: &str = types::tests::MERCHANT2_ID;

    fn prepare_full_env(status: EscrowStatus) -> AppContext<'static> {
        let config = types::tests::create_escrow_config();
        let buf = trinci_sdk::rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(ESCROW_ID, CONFIG_KEY, &buf);

        let buf = trinci_sdk::rmp_serialize(&status).unwrap();
        not_wasm::set_account_data(ESCROW_ID, STATUS_KEY, &buf);

        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::create_app_context(ESCROW_ID, GUARANTOR_ID)
    }

    #[test]
    fn initialization_with_founds() {
        let ctx = not_wasm::create_app_context(ESCROW_ID, ESCROW_ID);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_account_asset_gen(ESCROW_ID, ASSET_ID, Asset::new(1000));
        let args = types::tests::create_escrow_config();

        not_wasm::call_wrap(init, ctx, args.clone()).unwrap();

        let status = not_wasm::get_account_data(ESCROW_ID, STATUS_KEY);
        assert_eq!(status, rmp_serialize(&EscrowStatus::Open).unwrap());
        let config = not_wasm::get_account_data(ESCROW_ID, CONFIG_KEY);
        assert_eq!(config, rmp_serialize(&args).unwrap());
        let asset: Asset = not_wasm::get_account_asset_gen(ESCROW_ID, ASSET_ID);
        assert_eq!(asset.units, 1000);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Withdraw)));
    }

    #[test]
    fn initialization_with_no_funds() {
        let ctx = not_wasm::create_app_context(ESCROW_ID, ESCROW_ID);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);

        let args = types::tests::create_escrow_config();

        not_wasm::call_wrap(init, ctx, args.clone()).unwrap();

        let status = not_wasm::get_account_data(ESCROW_ID, STATUS_KEY);
        assert_eq!(status, rmp_serialize(&EscrowStatus::Open).unwrap());
        let config = not_wasm::get_account_data(ESCROW_ID, CONFIG_KEY);
        assert_eq!(config, rmp_serialize(&args).unwrap());
        let asset: Asset = not_wasm::get_account_asset_gen(ESCROW_ID, ASSET_ID);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Withdraw)));
    }

    #[test]
    fn update_closed_status() {
        let ctx = prepare_full_env(EscrowStatus::Success);
        let args = UpdateArgs { status: "OK" };

        let err = not_wasm::call_wrap(update, ctx, args).unwrap_err();

        assert_eq!("closed escrow", err.to_string());
    }

    #[test]
    fn update_pending_status_with_ok() {
        let ctx = prepare_full_env(EscrowStatus::Open);
        not_wasm::set_account_asset_gen(ESCROW_ID, ASSET_ID, Asset::new(300));
        let args = UpdateArgs { status: "Ok" };

        not_wasm::call_wrap(update, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CUSTOMER_ID, ASSET_ID);
        assert_eq!(asset.units, 0);
        let asset: Asset = not_wasm::get_account_asset_gen(MERCHANT1_ID, ASSET_ID);
        assert_eq!(asset.units, 195);
        let asset: Asset = not_wasm::get_account_asset_gen(MERCHANT2_ID, ASSET_ID);
        assert_eq!(asset.units, 5);
        // Escrow funds now are unlocked
        let asset: Asset = not_wasm::get_account_asset_gen(ESCROW_ID, ASSET_ID);
        assert_eq!(asset.units, 100);
        assert_eq!(asset.lock, None);
    }

    #[test]
    fn update_pending_status_with_ko() {
        let ctx = prepare_full_env(EscrowStatus::Open);
        not_wasm::set_account_asset_gen(ESCROW_ID, ASSET_ID, Asset::new(300));
        let args = UpdateArgs { status: "Ko" };

        not_wasm::call_wrap(update, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CUSTOMER_ID, ASSET_ID);
        assert_eq!(asset.units, 200);
        let asset: Asset = not_wasm::get_account_asset_gen(MERCHANT1_ID, ASSET_ID);
        assert_eq!(asset.units, 0);
        let asset: Asset = not_wasm::get_account_asset_gen(MERCHANT2_ID, ASSET_ID);
        assert_eq!(asset.units, 0);
        // Escrow funds now are unlocked
        let asset: Asset = not_wasm::get_account_asset_gen(ESCROW_ID, ASSET_ID);
        assert_eq!(asset.units, 100);
        assert_eq!(asset.lock, None);
    }

    #[test]
    fn update_unauthorized() {
        let mut ctx = prepare_full_env(EscrowStatus::Open);
        ctx.caller = "DummyAccount";
        not_wasm::set_account_asset_gen(ESCROW_ID, ASSET_ID, Asset::new(200));
        let args = UpdateArgs { status: "OK" };

        let err = not_wasm::call_wrap(update, ctx, args).unwrap_err();

        assert_eq!("not authorized", err.to_string());
    }

    #[test]
    fn balance_authorized() {
        let ctx = prepare_full_env(EscrowStatus::Open);
        not_wasm::set_account_asset_gen(ESCROW_ID, ASSET_ID, Asset::new(200));
        let args = BalanceArgs { asset: ASSET_ID };

        let value = not_wasm::call_wrap(balance, ctx, args).unwrap();

        assert_eq!(value, 200);
    }

    #[test]
    fn balance_not_authorized() {
        let mut ctx = prepare_full_env(EscrowStatus::Open);
        ctx.caller = "DummyAccountId";
        not_wasm::set_account_asset_gen(ESCROW_ID, ASSET_ID, Asset::new(200));
        let args = BalanceArgs { asset: ASSET_ID };

        let err = not_wasm::call_wrap(balance, ctx, args).unwrap_err();

        assert_eq!("not authorized", err.to_string());
    }

    #[test]
    fn test_get_info() {
        let mut ctx = prepare_full_env(EscrowStatus::Open);
        ctx.caller = MERCHANT1_ID;

        not_wasm::set_account_asset_gen(ESCROW_ID, ASSET_ID, Asset::new(10));

        let args = PackedValue::default();

        let res = not_wasm::call_wrap(get_info, ctx, args).unwrap();

        let value: EscrowInfo = rmp_deserialize(&res.0).unwrap();

        let expected = EscrowInfo {
            config: types::tests::create_escrow_config(),
            amount: 10,
            status: "open",
        };

        assert_eq!(value, expected);
    }

    #[test]
    fn test_get_unauthorized_info() {
        let mut ctx = prepare_full_env(EscrowStatus::Open);
        ctx.caller = "AnyOne";
        not_wasm::set_account_asset_gen(ESCROW_ID, ASSET_ID, Asset::new(10));

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(get_info, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }
}
