/*!
Dynamic Exchange

Allows a user (the `seller`) to exchange an asset for other assets

### Rules
 1. Only the account owner can initialize the Dynamic Exchange.
 2. Anyone can get the Dynamic Exchange information.
 3. Who `apply` to the Dynamic Exchange must have added a delegation
    to the `destination asset` or the quantity he wishes to purchase and
    must own that quantity of Asset in his account.
 4. The Dynamic Exchange Abort can be performed only by the Seller
    or the guarantor.

See other at: <https://gitlab.affidaty.net/developer2/t2-contracts/-/issues/2>
*/

use trinci_sdk::{tai::LockType, AppContext, PackedValue, WasmError, WasmResult};

mod types;
use types::*;

trinci_sdk::app_export!(init, get_info, apply, abort);

const CONFIG_KEY: &str = "config";
const STATUS_KEY: &str = "status";

#[inline]
fn get_status() -> WasmResult<DynamicExchangeStatus> {
    let buf = trinci_sdk::load_data(STATUS_KEY);

    trinci_sdk::rmp_deserialize(&buf)
}

#[inline]
fn set_status(status: DynamicExchangeStatus) -> WasmResult<()> {
    let buf = trinci_sdk::rmp_serialize(&status)?;
    trinci_sdk::store_data(STATUS_KEY, &buf);
    Ok(())
}

/// Checks if the init args are valid
fn check_init_args(init_args: &DynamicExchangeInitArgs) -> WasmResult<()> {
    if init_args.seller.is_empty()
        || init_args.asset.is_empty()
        || init_args.guarantor.is_empty()
        || init_args.penalty_asset.is_empty()
    {
        return Err(WasmError::new("account fields cannot be empty"));
    }

    if init_args.assets.is_empty() {
        return Err(WasmError::new("the assets for exchange must be specified"));
    }

    Ok(())
}

/// Initialize contract status.
/// The caller shall be the asset account owner.
pub fn init(ctx: AppContext, args: DynamicExchangeInitArgs) -> WasmResult<()> {
    if get_status().is_ok() {
        return Ok(());
    }

    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }

    check_init_args(&args)?;

    // Withdraw lock for the asset under dynamic exchange.
    trinci_sdk::asset_lock(args.asset, ctx.owner, trinci_sdk::tai::LockType::Withdraw)?;

    if args.penalty_asset != args.asset {
        trinci_sdk::asset_lock(
            args.penalty_asset,
            ctx.owner,
            trinci_sdk::tai::LockType::Withdraw,
        )?;
    }

    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &args)?;

    set_status(DynamicExchangeStatus::Open)?;

    Ok(())
}

/// Get the contract information.
pub fn get_info(ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    let status = get_status()?;

    // Load the contract config
    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: DynamicExchangeConfig = trinci_sdk::rmp_deserialize(&buf)
        .map_err(|_err| WasmError::new("contract not initialized"))?;

    trinci_sdk::asset_lock(config.asset, ctx.owner, LockType::None)?;
    let amount = trinci_sdk::asset_balance(config.asset)?;
    trinci_sdk::asset_lock(config.asset, ctx.owner, LockType::Full)?;

    let info = DynamicExchangeInfo {
        config,
        amount,
        status: &status.to_string(),
    };

    let buf = trinci_sdk::rmp_serialize_named(&info)?;
    Ok(PackedValue(buf))
}

/// Prematurely ends the contract
/// Transfers the asset amount (less a possible fee) to the seller
pub fn abort(ctx: AppContext, _args: PackedValue) -> WasmResult<()> {
    let status = get_status()?;

    // Load the contract config
    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: DynamicExchangeConfig = trinci_sdk::rmp_deserialize(&buf)
        .map_err(|_err| WasmError::new("contract not initialized"))?;

    if ctx.caller != config.seller && ctx.caller != config.guarantor {
        return Err(WasmError::new("not authorized"));
    }

    if status != DynamicExchangeStatus::Open {
        return Err(WasmError::new("exchange not open"));
    }

    trinci_sdk::asset_lock(config.asset, ctx.owner, LockType::None)?;
    let remaining_amount = trinci_sdk::asset_balance(config.asset)?;

    let guarantor_part;
    let seller_part;
    let penalty_fee_origin;
    let lock_penalty_asset;
    if config.asset == config.penalty_asset {
        guarantor_part = math::round::half_up(
            remaining_amount as f64 * config.penalty_fee as f64 / 1000f64,
            0,
        ) as u64; // penalty fee
        seller_part = remaining_amount - guarantor_part;
        penalty_fee_origin = ctx.owner;
        lock_penalty_asset = false;
    } else {
        guarantor_part = config.penalty_fee; // penalty_fee is an amount and not a percentage
        seller_part = remaining_amount;
        penalty_fee_origin = config.seller;
        lock_penalty_asset = true;
    }

    // Transfer Asset1 from Exchange to Seller
    if seller_part > 0 {
        trinci_sdk::asset_transfer(ctx.owner, config.seller, config.asset, seller_part)
            .map_err(|_err| WasmError::new("failed transfer to seller"))?;
    }
    if guarantor_part > 0 {
        trinci_sdk::asset_transfer(
            penalty_fee_origin,
            config.guarantor,
            config.penalty_asset,
            guarantor_part,
        )
        .map_err(|_err| WasmError::new("failed transfer penalty_fee to guarantor"))?;
    }
    trinci_sdk::asset_lock(config.asset, ctx.owner, LockType::Full)?;
    if lock_penalty_asset {
        trinci_sdk::asset_lock(config.penalty_asset, ctx.owner, LockType::Full)?;
    }
    // Puts the exchange status to Aborted
    set_status(DynamicExchangeStatus::Aborted)?;

    Ok(())
}

/**
 * Apply to the dynamic exchange.
 * Transfer asset from the buyer to the exchange and put it in a list of buyers
 */
pub fn apply(ctx: AppContext, args: ApplyArgs) -> WasmResult<()> {
    // Load the contract config
    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: DynamicExchangeConfig = trinci_sdk::rmp_deserialize(&buf)
        .map_err(|_err| WasmError::new("contract not initialized"))?;

    if get_status()? != DynamicExchangeStatus::Open {
        return Err(WasmError::new("exchange not open"));
    }

    if args.amount == 0 {
        return Err(WasmError::new("amount cannot be zero"));
    }

    // Check if the buyer asset2 is in the config assets list
    let asset_rate = *config
        .assets
        .get(args.asset)
        .ok_or_else(|| WasmError::new("not exchangeable asset"))?;

    // check if the buyer amount is equal or less the dynamic exchange amount
    let converted_asset =
        math::round::half_up(args.amount as f64 * (asset_rate as f64 / 100f64), 0) as u64;

    trinci_sdk::asset_lock(config.asset, ctx.owner, LockType::None)?;
    let max_amount = trinci_sdk::asset_balance(config.asset)?;

    if max_amount < converted_asset {
        return Err(WasmError::new("insufficient funds"));
    }

    let guarantor_part = math::round::half_up(
        args.amount as f64 * config.guarantor_fee as f64 / 1000f64,
        0,
    ) as u64;

    let seller_part = args.amount - guarantor_part;

    // Transfer Asset2 from Buyer to Dynamic Exchange
    trinci_sdk::asset_transfer(ctx.caller, ctx.owner, args.asset, args.amount)
        .map_err(|_err| WasmError::new("failed transfer from buyer"))?;
    // Transfer Asset2 from Dynamic Exchange to Seller
    trinci_sdk::asset_transfer(ctx.owner, config.seller, args.asset, seller_part)
        .map_err(|_err| WasmError::new("failed transfer to seller"))?;

    if guarantor_part > 0 {
        // Transfer Asset2 Fee from Dynamic Exchange to Guarantor
        trinci_sdk::asset_transfer(ctx.owner, config.guarantor, args.asset, guarantor_part)
            .map_err(|_err| WasmError::new("failed transfer to guarantor"))?;
    }
    // Transfer Asset1 from Exchange to Buyer
    trinci_sdk::asset_transfer(ctx.owner, ctx.caller, config.asset, converted_asset)
        .map_err(|_err| WasmError::new("failed transfer to buyer"))?;

    let new_amount = trinci_sdk::asset_balance(config.asset)?;
    trinci_sdk::asset_lock(config.asset, ctx.owner, LockType::Full)?;

    // Checks if the exchange is exhausted
    if new_amount == 0 {
        set_status(DynamicExchangeStatus::Exhausted)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::types::tests::{
        create_dynamic_exchange_config, ASSET1_ID, ASSET_ID, BUYER_ID, DYNAMIC_EXCHANGE_ID,
        GUARANTOR_ID, PENALTY_ASSET_ID, SELLER_ID,
    };

    use super::*;
    use trinci_sdk::{
        not_wasm, rmp_deserialize, rmp_serialize,
        tai::{Asset, LockPrivilege},
    };

    fn prepare_full_env(
        caller: &'static str,
        status: DynamicExchangeStatus,
        exchange_amount: u64,
        buyer_amount: u64,
        penalty_asset: &'static str,
    ) -> AppContext<'static> {
        let config = types::tests::create_dynamic_exchange_config(penalty_asset);
        let buf = trinci_sdk::rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(DYNAMIC_EXCHANGE_ID, CONFIG_KEY, &buf);

        let buf = trinci_sdk::rmp_serialize(&status).unwrap();
        not_wasm::set_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY, &buf);

        not_wasm::set_account_asset_gen(
            DYNAMIC_EXCHANGE_ID,
            ASSET_ID,
            Asset {
                units: exchange_amount,
                lock: None,
            },
        );

        not_wasm::set_account_asset_gen(
            caller,
            ASSET1_ID,
            Asset {
                units: buyer_amount,
                lock: None,
            },
        );

        not_wasm::set_contract_method(caller, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(ASSET1_ID, "transfer", not_wasm::asset_transfer);

        not_wasm::set_contract_method(caller, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET1_ID, "balance", not_wasm::asset_balance);

        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::create_app_context(DYNAMIC_EXCHANGE_ID, caller)
    }

    #[test]
    fn initialization_with_funds() {
        let ctx = not_wasm::create_app_context(DYNAMIC_EXCHANGE_ID, DYNAMIC_EXCHANGE_ID);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID, Asset::new(1000));
        let args = types::tests::create_dynamic_exchange_config(ASSET_ID);

        not_wasm::call_wrap(init, ctx, args.clone()).unwrap();

        let status = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY);
        assert_eq!(status, rmp_serialize(&DynamicExchangeStatus::Open).unwrap());
        let config = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, CONFIG_KEY);
        assert_eq!(config, rmp_serialize(&args).unwrap());
        let asset: Asset = not_wasm::get_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID);
        assert_eq!(asset.units, 1000);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Withdraw)));
    }

    #[test]
    fn initialization_with_penalty_asset_and_funds() {
        let ctx = not_wasm::create_app_context(DYNAMIC_EXCHANGE_ID, DYNAMIC_EXCHANGE_ID);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_contract_method(PENALTY_ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(PENALTY_ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID, Asset::new(1000));
        not_wasm::set_account_asset_gen(DYNAMIC_EXCHANGE_ID, PENALTY_ASSET_ID, Asset::new(100));
        let args = types::tests::create_dynamic_exchange_config(PENALTY_ASSET_ID);

        not_wasm::call_wrap(init, ctx, args.clone()).unwrap();

        let status = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY);
        assert_eq!(status, rmp_serialize(&DynamicExchangeStatus::Open).unwrap());

        let config = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, CONFIG_KEY);
        assert_eq!(config, rmp_serialize(&args).unwrap());

        let asset: Asset = not_wasm::get_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID);
        assert_eq!(asset.units, 1000);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Withdraw)));

        let asset: Asset = not_wasm::get_account_asset_gen(DYNAMIC_EXCHANGE_ID, PENALTY_ASSET_ID);
        assert_eq!(asset.units, 100);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Withdraw)));
    }

    #[test]
    fn bad_args_initialization_1() {
        // Missing assets
        let ctx = not_wasm::create_app_context(DYNAMIC_EXCHANGE_ID, DYNAMIC_EXCHANGE_ID);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID, Asset::new(1000));
        let mut args = types::tests::create_dynamic_exchange_config(ASSET_ID);
        args.assets = HashMap::new();

        let err = not_wasm::call_wrap(init, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "the assets for exchange must be specified");
    }

    #[test]
    fn bad_args_initialization_2() {
        // Missing penalty_asset
        let ctx = not_wasm::create_app_context(DYNAMIC_EXCHANGE_ID, DYNAMIC_EXCHANGE_ID);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID, Asset::new(1000));
        let mut args = types::tests::create_dynamic_exchange_config(ASSET_ID);
        args.penalty_asset = "";

        let err = not_wasm::call_wrap(init, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "account fields cannot be empty");
    }

    #[test]
    fn unhautorized_initialization() {
        let ctx = not_wasm::create_app_context(DYNAMIC_EXCHANGE_ID, "Unexistent_account");
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID, Asset::new(1000));
        let args = types::tests::create_dynamic_exchange_config(ASSET_ID);

        let err = not_wasm::call_wrap(init, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn initialization_with_no_funds() {
        let ctx = not_wasm::create_app_context(DYNAMIC_EXCHANGE_ID, DYNAMIC_EXCHANGE_ID);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);

        let args = types::tests::create_dynamic_exchange_config(ASSET_ID);

        not_wasm::call_wrap(init, ctx, args.clone()).unwrap();

        let status = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY);
        assert_eq!(status, rmp_serialize(&DynamicExchangeStatus::Open).unwrap());
        let config = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, CONFIG_KEY);
        assert_eq!(config, rmp_serialize(&args).unwrap());
        let asset: Asset = not_wasm::get_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Withdraw)));
    }

    #[test]
    fn test_get_info() {
        let ctx = prepare_full_env("Anyone", DynamicExchangeStatus::Open, 10, 0, ASSET_ID);
        let args = PackedValue::default();

        let res = not_wasm::call_wrap(get_info, ctx, args).unwrap();

        let value: DynamicExchangeInfo = rmp_deserialize(&res.0).unwrap();

        let expected = DynamicExchangeInfo {
            config: create_dynamic_exchange_config(ASSET_ID),
            amount: 10,
            status: "open",
        };

        assert_eq!(value, expected);
    }

    #[test]
    fn test_apply_wrong_asset() {
        let ctx = prepare_full_env(BUYER_ID, DynamicExchangeStatus::Open, 100, 0, ASSET_ID);

        let args = ApplyArgs {
            asset: "BTC",
            amount: 10,
        };

        let err = not_wasm::call_wrap(apply, ctx, args).unwrap_err();

        assert_eq!(&err.to_string(), "not exchangeable asset");
    }

    #[test]
    fn test_apply_with_amount_zero() {
        let ctx = prepare_full_env(BUYER_ID, DynamicExchangeStatus::Open, 100, 0, ASSET_ID);

        let args = ApplyArgs {
            asset: ASSET1_ID,
            amount: 0,
        };

        let err = not_wasm::call_wrap(apply, ctx, args).unwrap_err();

        assert_eq!(&err.to_string(), "amount cannot be zero");
    }

    #[test]

    fn test_apply_asset_with_insufficients_funds() {
        let ctx = prepare_full_env(BUYER_ID, DynamicExchangeStatus::Open, 100, 2, ASSET_ID);

        let args = ApplyArgs {
            asset: ASSET1_ID,
            amount: 10,
        };

        let err = not_wasm::call_wrap(apply, ctx, args).unwrap_err();

        assert_eq!(&err.to_string(), "failed transfer from buyer");
    }

    #[test]
    fn test_apply_asset() {
        let ctx = prepare_full_env(BUYER_ID, DynamicExchangeStatus::Open, 100, 99, ASSET_ID);

        let args = ApplyArgs {
            asset: ASSET1_ID,
            amount: 10,
        };

        not_wasm::call_wrap(apply, ctx, args).unwrap();

        let remaining_asset: Asset = not_wasm::get_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID);
        assert_eq!(remaining_asset.units, 75);

        let buyer_asset1: Asset = not_wasm::get_account_asset_gen(BUYER_ID, ASSET1_ID);
        assert_eq!(buyer_asset1.units, 89);

        let seller_asset1: Asset = not_wasm::get_account_asset_gen(SELLER_ID, ASSET1_ID);
        assert_eq!(seller_asset1.units, 9);

        let guarantor_asset1: Asset = not_wasm::get_account_asset_gen(GUARANTOR_ID, ASSET1_ID);
        assert_eq!(guarantor_asset1.units, 1);
    }

    #[test]
    fn test_apply_exhausting() {
        let ctx = prepare_full_env(BUYER_ID, DynamicExchangeStatus::Open, 100, 99, ASSET_ID);

        let args = ApplyArgs {
            asset: ASSET1_ID,
            amount: 40,
        };

        not_wasm::call_wrap(apply, ctx, args).unwrap();

        let status = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY);
        assert_eq!(
            status,
            rmp_serialize(&DynamicExchangeStatus::Exhausted).unwrap()
        );

        let remaining_asset: Asset = not_wasm::get_account_asset_gen(DYNAMIC_EXCHANGE_ID, ASSET_ID);
        assert_eq!(remaining_asset.units, 0);
    }

    #[test]
    fn test_unhauthorized_abort() {
        let ctx = prepare_full_env(BUYER_ID, DynamicExchangeStatus::Open, 100, 0, ASSET_ID);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(abort, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");

        let status = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY);

        assert_eq!(status, rmp_serialize(&DynamicExchangeStatus::Open).unwrap());
    }

    #[test]
    fn test_abort_from_seller() {
        let ctx = prepare_full_env(SELLER_ID, DynamicExchangeStatus::Open, 100, 0, ASSET_ID);

        let args = PackedValue::default();

        not_wasm::call_wrap(abort, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY);
        let status: DynamicExchangeStatus = rmp_deserialize(&buf).unwrap();

        assert_eq!(status, DynamicExchangeStatus::Aborted);
    }

    #[test]
    fn test_abort_from_guarantor() {
        let ctx = prepare_full_env(GUARANTOR_ID, DynamicExchangeStatus::Open, 100, 0, ASSET_ID);

        let args = PackedValue::default();

        not_wasm::call_wrap(abort, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY);
        let status: DynamicExchangeStatus = rmp_deserialize(&buf).unwrap();

        assert_eq!(status, DynamicExchangeStatus::Aborted);
    }

    #[test]
    fn test_abort_with_penalty_asset() {
        let ctx = prepare_full_env(
            GUARANTOR_ID,
            DynamicExchangeStatus::Open,
            250,
            0,
            PENALTY_ASSET_ID,
        );

        not_wasm::set_account_asset_gen(SELLER_ID, PENALTY_ASSET_ID, Asset::new(100));
        not_wasm::set_contract_method(PENALTY_ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_contract_method(PENALTY_ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(PENALTY_ASSET_ID, "transfer", not_wasm::asset_transfer);

        let args = PackedValue::default();

        not_wasm::call_wrap(abort, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(DYNAMIC_EXCHANGE_ID, STATUS_KEY);
        let status: DynamicExchangeStatus = rmp_deserialize(&buf).unwrap();

        assert_eq!(status, DynamicExchangeStatus::Aborted);

        let seller_asset: Asset = not_wasm::get_account_asset_gen(SELLER_ID, ASSET_ID);

        assert_eq!(seller_asset.units, 250);

        let guarantor_asset: Asset =
            not_wasm::get_account_asset_gen(GUARANTOR_ID, PENALTY_ASSET_ID);

        assert_eq!(guarantor_asset.units, 100);
    }
}
