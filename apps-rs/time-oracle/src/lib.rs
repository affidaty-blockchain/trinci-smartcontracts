//! Time Oracle.
//!
//! Keeps track of time in the blockchain.
//!
//! ### Rules
//!
//! 1. Initialization can be performed only by the oracle creator.
//! 2. The oracle creator must periodically send an `update` tx.
//! 3. Anyone can call the `get_time` method.
//! 4. Anyone can call the `get_config` method.

use trinci_sdk::{AppContext, PackedValue, WasmError, WasmResult};

mod types;
use types::*;

trinci_sdk::app_export!(init, update, get_time, get_config);

const CONFIG_KEY: &str = "config";
const TIME_KEY: &str = "time";

#[inline]
fn is_initialized() -> bool {
    !trinci_sdk::load_data(TIME_KEY).is_empty()
}

#[inline]
fn init_check() -> WasmResult<()> {
    match is_initialized() {
        true => Ok(()),
        false => Err(WasmError::new("not initialized")),
    }
}

/// Initialize the oracle status.
/// The caller shall be the oracle account owner.
fn init(ctx: AppContext, args: InitArgs) -> WasmResult<()> {
    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }

    if is_initialized() {
        return Ok(());
    }

    let config = ConfigData {
        name: args.name,
        owner: ctx.caller,
        description: args.description,
        update_interval: args.update_interval,
    };

    trinci_sdk::store_account_data_mp!(TIME_KEY, &args.initial_time)?;
    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;

    Ok(())
}

/// Update the time
fn update(ctx: AppContext, args: u64) -> WasmResult<()> {
    if ctx.caller != ctx.owner {
        return Err(WasmError::new("not authorized"));
    }

    let prev = get_time(ctx, PackedValue::default())?;
    if prev > args {
        return Err(WasmError::new("value shall be monotonic"));
    }

    trinci_sdk::store_account_data_mp!(TIME_KEY, &args)
}

/// Get the oracle time
fn get_time(_ctx: AppContext, _args: PackedValue) -> WasmResult<u64> {
    let buf = trinci_sdk::load_data(TIME_KEY);
    if buf.is_empty() {
        return Err(WasmError::new("not initialized"));
    }
    let value: u64 = trinci_sdk::rmp_deserialize(&buf)?;
    Ok(value)
}

/// Get the oracle configuration
fn get_config(_ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: ConfigData = trinci_sdk::rmp_deserialize(&buf)?;
    let buf = trinci_sdk::rmp_serialize_named(&config)?;
    Ok(PackedValue(buf))
}

#[cfg(test)]
mod tests {
    use trinci_sdk::{not_wasm, rmp_deserialize};

    use super::*;
    use crate::types::tests::{create_config_data, create_init_args};

    const CALLER_ID: &str = "QmSCRCPFznxEX6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq";
    const ORACLE_ID: &str = "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";

    const INIT_TIME_VALUE: u64 = types::tests::INIT_TIME_VALUE;

    fn prepare_full_env() -> AppContext<'static> {
        let config = create_config_data();
        let buf = trinci_sdk::rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(ORACLE_ID, CONFIG_KEY, &buf);

        let time: u64 = INIT_TIME_VALUE;
        let buf = trinci_sdk::rmp_serialize(&time).unwrap();
        not_wasm::set_account_data(ORACLE_ID, TIME_KEY, &buf);

        not_wasm::create_app_context(ORACLE_ID, ORACLE_ID)
    }

    #[test]
    fn init_contract() {
        let ctx = not_wasm::create_app_context(ORACLE_ID, ORACLE_ID);
        let args: InitArgs = create_init_args();

        let expected_config = ConfigData {
            name: args.name,
            owner: ctx.caller,
            description: args.description,
            update_interval: args.update_interval,
        };

        not_wasm::call_wrap(init, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(ORACLE_ID, TIME_KEY);
        let time: u64 = rmp_deserialize(&buf).unwrap();
        assert_eq!(time, INIT_TIME_VALUE);

        let buf = not_wasm::get_account_data(ORACLE_ID, CONFIG_KEY);
        let config: ConfigData = rmp_deserialize(&buf).unwrap();
        assert_eq!(config, expected_config);
    }

    #[test]
    fn init_contract_not_authorized() {
        let ctx = not_wasm::create_app_context(ORACLE_ID, CALLER_ID);

        let args: InitArgs = create_init_args();

        let err = not_wasm::call_wrap(init, ctx, args).unwrap_err();
        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn update_time() {
        let ctx = prepare_full_env();
        let time = INIT_TIME_VALUE + 10;

        not_wasm::call_wrap(update, ctx, time).unwrap();

        let buf = not_wasm::get_account_data(ORACLE_ID, TIME_KEY);
        let value: u64 = rmp_deserialize(&buf).unwrap();
        assert_eq!(time, value);
    }

    #[test]
    fn update_time_no_monotonic() {
        let ctx = prepare_full_env();
        let time = INIT_TIME_VALUE - 10;

        let err = not_wasm::call_wrap(update, ctx, time).unwrap_err();

        assert_eq!(err.to_string(), "value shall be monotonic");
    }

    #[test]
    fn retrieving_the_time() {
        let ctx = prepare_full_env();
        let args = PackedValue::default();

        let time = not_wasm::call_wrap(get_time, ctx, args).unwrap();

        assert_eq!(time, INIT_TIME_VALUE);
    }

    #[test]
    fn retrieving_the_config() {
        let ctx = prepare_full_env();
        let args = PackedValue::default();
        let expected_config = create_config_data();
        let expected_config = trinci_sdk::rmp_serialize_named(&expected_config).unwrap();

        let config = not_wasm::call_wrap(get_config, ctx, args).unwrap();

        assert_eq!(config.0, expected_config);
    }
}
