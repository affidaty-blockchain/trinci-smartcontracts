//! Withdraw contract
//!
//! RULES:
//!
//! ### Methods
//!
//!  - `init`
//!    - can be called but the account owner or the
//!    - checks if there are enough funds, otherwise set the status to "idle"
//!  - `update`
//!    - "ok"  - make the burn of the assets
//!    - "ko"  - return the assets to the customer and the eschange

use trinci_sdk::{
    call, rmp_deserialize, rmp_serialize, AppContext, PackedValue, WasmError, WasmResult,
};

mod types;
use types::*;

const STATUS_KEY: &str = "status";
const CONFIG_KEY: &str = "config";

trinci_sdk::app_export!(init, update, get_info);

#[inline]
fn get_status() -> WasmResult<WithdrawStatus> {
    let buf = trinci_sdk::load_data(STATUS_KEY);
    trinci_sdk::rmp_deserialize(&buf).map_err(|_err| WasmError::new("contract not initialized"))
}

#[inline]
fn set_status(status: WithdrawStatus) -> WasmResult<()> {
    let buf = trinci_sdk::rmp_serialize(&status)?;
    trinci_sdk::store_data(STATUS_KEY, &buf);
    Ok(())
}

pub fn asset_burn(from: &str, asset: &str, units: u64) -> WasmResult<()> {
    let data = rmp_serialize(&BurnArgs { from, units })?;
    call(asset, "burn", &data).map(|_buf| ())
}

/// Initialize the withdraw contract
fn init(ctx: AppContext, args: InitArgs) -> WasmResult<()> {
    if get_status().is_ok() {
        return Ok(());
    }

    if ctx.caller != ctx.owner && ctx.caller != args.exchange {
        return Err(WasmError::new("not authorized"));
    }

    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &args)?;

    let withdrawn_asset_balance = trinci_sdk::asset_balance(args.withdrawn_asset.id)?;
    let currency_asset_balance = trinci_sdk::asset_balance(args.currency_asset.id)?;

    if !(withdrawn_asset_balance >= args.withdrawn_asset.units
        && currency_asset_balance >= args.currency_asset.units)
    {
        return Err(WasmError::new("not enough funds"));
    }

    // Withdraw lock for the asset under escrow.
    trinci_sdk::asset_lock(
        args.withdrawn_asset.id,
        ctx.owner,
        trinci_sdk::tai::LockType::Withdraw,
    )?;
    trinci_sdk::asset_lock(
        args.currency_asset.id,
        ctx.owner,
        trinci_sdk::tai::LockType::Withdraw,
    )?;

    set_status(WithdrawStatus::Open)
}

/// Get the contract information: config, amount and status.
fn get_info(ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    if get_status().is_err() {
        return Err(WasmError::new("not initialized"));
    }
    // Load the contract config
    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: WithdrawConfig = trinci_sdk::rmp_deserialize(&buf)?;

    let auth_list = vec![config.customer, config.exchange];

    let authorized = auth_list.iter().find(|&&elem| elem == ctx.caller);

    trinci_sdk::asset_lock(
        config.currency_asset.id,
        ctx.owner,
        trinci_sdk::tai::LockType::None,
    )?;
    trinci_sdk::asset_lock(
        config.withdrawn_asset.id,
        ctx.owner,
        trinci_sdk::tai::LockType::None,
    )?;

    let (currency_asset_amount, withdrawn_asset_amount) = match authorized {
        Some(_) => (
            trinci_sdk::asset_balance(config.currency_asset.id)?,
            trinci_sdk::asset_balance(config.withdrawn_asset.id)?,
        ),
        None => return Err(WasmError::new("not authorized")),
    };

    trinci_sdk::asset_lock(
        config.currency_asset.id,
        ctx.owner,
        trinci_sdk::tai::LockType::Full,
    )?;
    trinci_sdk::asset_lock(
        config.withdrawn_asset.id,
        ctx.owner,
        trinci_sdk::tai::LockType::Full,
    )?;

    let status = get_status()?;

    let info = WithdrawInfo {
        config,
        currency_asset_amount,
        withdrawn_asset_amount,
        status: &status.to_string(),
    };

    let buf = trinci_sdk::rmp_serialize_named(&info)?;
    Ok(PackedValue(buf))
}

// Performs the burns
fn burn_the_assets(
    withdraw_id: &str,
    currency_asset_id: &str,
    currency_asset_balance: u64,
    withdrawn_asset_id: &str,
    withdrawn_asset_balance: u64,
) -> WasmResult<()> {
    asset_burn(withdraw_id, currency_asset_id, currency_asset_balance)?;

    asset_burn(withdraw_id, withdrawn_asset_id, withdrawn_asset_balance)
}

// Performs the refunds
fn transfer_the_assets(
    withdraw_id: &str,
    exchange_id: &str,
    currency_asset_id: &str,
    currency_asset_balance: u64,
    customer_id: &str,
    withdrawn_asset_id: &str,
    withdrawn_asset_balance: u64,
) -> WasmResult<()> {
    trinci_sdk::asset_transfer(
        withdraw_id,
        exchange_id,
        currency_asset_id,
        currency_asset_balance,
    )?;

    trinci_sdk::asset_transfer(
        withdraw_id,
        customer_id,
        withdrawn_asset_id,
        withdrawn_asset_balance,
    )
}

fn close_contract(withdraw_id: &str, config: WithdrawConfig, success: bool) -> WasmResult<()> {
    // Unlock the assets
    trinci_sdk::asset_lock(
        config.currency_asset.id,
        withdraw_id,
        trinci_sdk::tai::LockType::None,
    )?;
    trinci_sdk::asset_lock(
        config.withdrawn_asset.id,
        withdraw_id,
        trinci_sdk::tai::LockType::None,
    )?;
    let currency_asset_balance = trinci_sdk::asset_balance(config.currency_asset.id)?;
    let withdrawn_asset_balance = trinci_sdk::asset_balance(config.withdrawn_asset.id)?;
    let status: WithdrawStatus;

    if success {
        burn_the_assets(
            withdraw_id,
            config.currency_asset.id,
            currency_asset_balance,
            config.withdrawn_asset.id,
            withdrawn_asset_balance,
        )?;
        status = WithdrawStatus::Success;
    } else {
        transfer_the_assets(
            withdraw_id,
            config.exchange,
            config.currency_asset.id,
            currency_asset_balance,
            config.customer,
            config.withdrawn_asset.id,
            withdrawn_asset_balance,
        )?;
        status = WithdrawStatus::Failure;
    }
    // Lock the assets
    trinci_sdk::asset_lock(
        config.currency_asset.id,
        withdraw_id,
        trinci_sdk::tai::LockType::Full,
    )?;
    trinci_sdk::asset_lock(
        config.withdrawn_asset.id,
        withdraw_id,
        trinci_sdk::tai::LockType::Full,
    )?;
    set_status(status)
}

/// Close the contract if not already close preforming
/// the burning or the transfer of its assets
fn update(ctx: AppContext, args: UpdateArgs) -> WasmResult<()> {
    match get_status() {
        Ok(WithdrawStatus::Failure) | Ok(WithdrawStatus::Success) => {
            return Err(WasmError::new("already closed"))
        }
        Err(_) => return Err(WasmError::new("not initialized")),
        Ok(WithdrawStatus::Open) => {}
    };

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: WithdrawConfig = rmp_deserialize(&buf)?;

    if ctx.caller != config.exchange {
        return Err(WasmError::new("not authorized"));
    }

    match args.status.to_uppercase().as_ref() {
        "OK" => close_contract(ctx.owner, config, true),
        "KO" => close_contract(ctx.owner, config, false),
        _ => Err(WasmError::new("bad update arguments")),
    }
}

#[cfg(test)]
mod tests {

    use trinci_sdk::{
        load_asset_typed, not_wasm, rmp_deserialize, store_asset_typed, tai::Asset, PackedValue,
    };

    use crate::types::tests::{
        create_withdraw_config, create_withdraw_info, CURRENCY_ASSET_ID, CUSTOMER_ID, EXCHANGE_ID,
        WITHDRAWN_ASSET_ID, WITHDRAW_ID,
    };

    use super::*;

    fn prepare_full_env(
        caller: &'static str,
        set_config: bool,
        status: Option<WithdrawStatus>,
    ) -> AppContext<'static> {
        if set_config {
            let config = types::tests::create_withdraw_config(100, 500);
            let buf = trinci_sdk::rmp_serialize(&config).unwrap();
            not_wasm::set_account_data(WITHDRAW_ID, CONFIG_KEY, &buf);
        }
        if status.is_some() {
            let buf = trinci_sdk::rmp_serialize(&status).unwrap();
            not_wasm::set_account_data(WITHDRAW_ID, STATUS_KEY, &buf);
        }
        not_wasm::set_contract_method(CURRENCY_ASSET_ID, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(CURRENCY_ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(CURRENCY_ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::set_contract_method(WITHDRAWN_ASSET_ID, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(WITHDRAWN_ASSET_ID, "balance", not_wasm::asset_balance);
        not_wasm::set_contract_method(WITHDRAWN_ASSET_ID, "lock", not_wasm::asset_lock);
        not_wasm::create_app_context(WITHDRAW_ID, caller)
    }

    /// Mocked Asset `burn` method used by the tests.
    pub fn asset_burn(_ctx: AppContext, args: PackedValue) -> WasmResult<PackedValue> {
        let args: BurnArgs = rmp_deserialize(&args).unwrap();

        // Withdraw
        let mut value: Asset = load_asset_typed(args.from);
        if value.lock.is_some() {
            return Err(WasmError::new("source account locked"));
        }
        if value.units < args.units {
            return Err(WasmError::new("error during transfer"));
        }
        value.units -= args.units;
        store_asset_typed(args.from, value);

        let buf = rmp_serialize(&()).unwrap();
        Ok(PackedValue(buf))
    }

    #[test]
    fn init_not_enough_funds_test() {
        let ctx = prepare_full_env(WITHDRAW_ID, false, None);
        let args = create_withdraw_config(42, 1000);

        let err = not_wasm::call_wrap(init, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "not enough funds");
    }

    #[test]
    fn init_not_authorized_test() {
        let ctx = prepare_full_env("NotAuthorized", false, None);
        let args = create_withdraw_config(100, 500);

        let err = not_wasm::call_wrap(init, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn init_test() {
        let ctx = prepare_full_env(WITHDRAW_ID, false, None);
        let args = create_withdraw_config(42, 1000);

        not_wasm::set_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID, Asset::new(42));
        not_wasm::set_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID, Asset::new(1000));

        not_wasm::call_wrap(init, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(WITHDRAW_ID, CONFIG_KEY);
        let config: WithdrawConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(config, args);

        let buf = not_wasm::get_account_data(WITHDRAW_ID, STATUS_KEY);
        let status: WithdrawStatus = rmp_deserialize(&buf).unwrap();

        assert_eq!(status, WithdrawStatus::Open);
    }

    #[test]
    fn update_ok_test() {
        let ctx = prepare_full_env(EXCHANGE_ID, true, Some(WithdrawStatus::Open));

        not_wasm::set_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID, Asset::new(100));
        not_wasm::set_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID, Asset::new(500));

        not_wasm::set_contract_method(CURRENCY_ASSET_ID, "burn", asset_burn);
        not_wasm::set_contract_method(WITHDRAWN_ASSET_ID, "burn", asset_burn);

        let args = UpdateArgs { status: "Ok" };

        not_wasm::call_wrap(update, ctx, args).unwrap();

        let account_currency_asset: Asset =
            not_wasm::get_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID);
        assert_eq!(account_currency_asset.units, 0);

        let account_withdrawn_asset: Asset =
            not_wasm::get_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID);
        assert_eq!(account_withdrawn_asset.units, 0);

        let exchange_currency_asset: Asset =
            not_wasm::get_account_asset_gen(EXCHANGE_ID, CURRENCY_ASSET_ID);
        assert_eq!(exchange_currency_asset.units, 0);

        let exchange_withdrawn_asset: Asset =
            not_wasm::get_account_asset_gen(CUSTOMER_ID, WITHDRAWN_ASSET_ID);
        assert_eq!(exchange_withdrawn_asset.units, 0);

        let buf = not_wasm::get_account_data(WITHDRAW_ID, STATUS_KEY);
        let status: WithdrawStatus = rmp_deserialize(&buf).unwrap();
        assert_eq!(status, WithdrawStatus::Success);
    }

    #[test]
    fn update_ko_test() {
        let ctx = prepare_full_env(EXCHANGE_ID, true, Some(WithdrawStatus::Open));

        not_wasm::set_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID, Asset::new(100));
        not_wasm::set_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID, Asset::new(500));

        let args = UpdateArgs { status: "kO" };

        not_wasm::call_wrap(update, ctx, args).unwrap();

        let account_currency_asset: Asset =
            not_wasm::get_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID);
        assert_eq!(account_currency_asset.units, 0);

        let account_withdrawn_asset: Asset =
            not_wasm::get_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID);
        assert_eq!(account_withdrawn_asset.units, 0);

        let exchange_currency_asset: Asset =
            not_wasm::get_account_asset_gen(EXCHANGE_ID, CURRENCY_ASSET_ID);
        assert_eq!(exchange_currency_asset.units, 100);

        let exchange_withdrawn_asset: Asset =
            not_wasm::get_account_asset_gen(CUSTOMER_ID, WITHDRAWN_ASSET_ID);
        assert_eq!(exchange_withdrawn_asset.units, 500);

        let buf = not_wasm::get_account_data(WITHDRAW_ID, STATUS_KEY);
        let status: WithdrawStatus = rmp_deserialize(&buf).unwrap();

        assert_eq!(status, WithdrawStatus::Failure);
    }

    #[test]
    fn update_on_closed_contract_test() {
        let ctx = prepare_full_env(EXCHANGE_ID, true, Some(WithdrawStatus::Failure));

        not_wasm::set_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID, Asset::new(100));
        not_wasm::set_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID, Asset::new(500));

        let args = UpdateArgs { status: "Ok" };

        let err = not_wasm::call_wrap(update, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "already closed");
    }

    #[test]
    fn update_with_bad_arguments_test() {
        let ctx = prepare_full_env(EXCHANGE_ID, true, Some(WithdrawStatus::Open));

        not_wasm::set_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID, Asset::new(100));
        not_wasm::set_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID, Asset::new(500));

        let args = UpdateArgs { status: "123" };

        let err = not_wasm::call_wrap(update, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "bad update arguments");
    }

    #[test]
    fn update_not_authorized_test() {
        let ctx = prepare_full_env(CUSTOMER_ID, true, Some(WithdrawStatus::Open));

        not_wasm::set_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID, Asset::new(100));
        not_wasm::set_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID, Asset::new(500));

        let args = UpdateArgs { status: "Ok" };

        let err = not_wasm::call_wrap(update, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn get_info_test() {
        let ctx = prepare_full_env(EXCHANGE_ID, true, Some(WithdrawStatus::Open));

        not_wasm::set_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID, Asset::new(100));
        not_wasm::set_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID, Asset::new(500));

        let args = PackedValue::default();

        let res = not_wasm::call_wrap(get_info, ctx, args).unwrap();

        let info: WithdrawInfo = rmp_deserialize(&res.0).unwrap();
        let expected = create_withdraw_info(100, 500, "open");

        assert_eq!(info, expected);
    }

    #[test]
    fn get_info_not_authorized_test() {
        let ctx = prepare_full_env(WITHDRAW_ID, true, Some(WithdrawStatus::Open));

        not_wasm::set_account_asset_gen(WITHDRAW_ID, CURRENCY_ASSET_ID, Asset::new(100));
        not_wasm::set_account_asset_gen(WITHDRAW_ID, WITHDRAWN_ASSET_ID, Asset::new(500));

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(get_info, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }
}
