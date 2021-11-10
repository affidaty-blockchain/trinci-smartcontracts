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

//! Test Contract.
//!
//! Smart contract allowing to execute tests from the host environment
//! Performs the test for:
//!  - Input and output serialization.
//!  - Default smart contract functionalities.
//!  - Trigger exceptional conditions.

use serde_derive::{Deserialize, Serialize};
use std::{
    alloc::{alloc, Layout},
    collections::HashMap,
    convert::TryInto,
    mem::align_of,
};
use trinci_sdk::{
    rmp_deserialize, rmp_serialize, tai::AssetTransferArgs, value, AppContext, PackedValue, Value,
    WasmError, WasmResult,
};

trinci_sdk::app_export!(
    // Input and output serialization.
    echo_generic,
    echo_typed,
    echo_packed,
    // Default smart contract functionalities.
    nested_call,
    balance,
    transfer,
    notify,
    // Trigger exceptional conditions.
    divide_by_zero,
    trigger_panic,
    exhaust_memory,
    infinite_recursion,
    infinite_loop,
    null_pointer_indirection
);

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq, Clone, Default))]
struct SubStruct<'a> {
    pub field1: u32,
    pub field2: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq, Clone, Default))]
struct EchoArgs<'a> {
    pub name: &'a str,
    pub surname: String,
    #[serde(with = "serde_bytes")]
    pub buf: Vec<u8>,
    pub vec8: Vec<u8>,
    pub vec16: Vec<u16>,
    pub map: HashMap<&'a str, SubStruct<'a>>,
}

/// Returns the input data "as is".
fn echo_generic(ctx: AppContext, args: Value) -> WasmResult<Value> {
    trinci_sdk::log!("Caller: {}, Args: {:?}", ctx.caller, args);
    Ok(args)
}

/// Returns the input data "as is".
fn echo_typed<'a>(ctx: AppContext, args: EchoArgs<'a>) -> WasmResult<EchoArgs<'a>> {
    trinci_sdk::log!("Caller: {}, Args: {:?}", ctx.caller, args);
    Ok(args)
}

/// Returns the input data "as is".
fn echo_packed(ctx: AppContext, args: PackedValue) -> WasmResult<PackedValue> {
    trinci_sdk::log!("Caller: {}, Args: {:?}", ctx.caller, args);
    Ok(args)
}

/// Trampoline method to call another one via `sdk::call`.
fn nested_call(ctx: AppContext, args: Value) -> WasmResult<Value> {
    let input = rmp_serialize(&args).unwrap();
    let output = trinci_sdk::call(ctx.owner, "echo_packed", &input)?;
    rmp_deserialize(&output)
}

#[inline]
fn load_my_asset(account: &str) -> u64 {
    let buf = trinci_sdk::load_asset(account);
    let buf: [u8; 8] = buf.try_into().unwrap_or_default();
    u64::from_be_bytes(buf)
}

#[inline]
fn store_my_asset(account: &str, value: u64) {
    let buf = value.to_be_bytes();
    trinci_sdk::store_asset(account, buf.as_ref());
}

/// Call the host function hf_balance to get the caller account balance
fn balance(ctx: AppContext, _args: PackedValue) -> WasmResult<u64> {
    trinci_sdk::log("Called method `balance`");
    let value = load_my_asset(ctx.caller);
    Ok(value)
}

/// Call the host function hf_transfer
///
/// Transfer an *amount* of *asset* from the *caller account* to the *dest account*
fn transfer(ctx: AppContext, args: AssetTransferArgs) -> WasmResult<()> {
    trinci_sdk::log("Called method `transfer`");

    // Only the owner is allowed to transfer its asset.
    #[allow(clippy::suspicious_operation_groupings)]
    if ctx.caller != ctx.owner && ctx.caller != args.from {
        return Err(WasmError::new("not authorized"));
    }

    // Withdraw
    let value = load_my_asset(args.from);
    if args.units > value {
        return Err(WasmError::new("not enough funds"));
    }
    store_my_asset(ctx.caller, value - args.units);

    // Deposit
    let value = load_my_asset(args.to);
    store_my_asset(args.to, value + args.units);

    Ok(())
}

/// Divide by zero test method
fn divide_by_zero(_ctx: AppContext, args: Value) -> WasmResult<Value> {
    trinci_sdk::log("Called method `divide_by_zero`");

    let value1 = 100u64;
    let value2 = trinci_sdk::get_value_as_u64!(args, "zero")?;

    Ok(value!(value1 / value2))
}

/// Trigger panic test method
fn trigger_panic(_ctx: AppContext, _args: Value) -> WasmResult<Value> {
    trinci_sdk::log("Called method `trigger_panic`");

    panic!("This is a panic message into wasm method!");

    #[allow(unreachable_code)]
    Ok(value!(null))
}

// Prevents optimization.
#[inline(never)]
fn do_alloc(size: usize) -> *const u8 {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align_of::<usize>());
        alloc(layout)
    }
}

/// Exhaust memory test method
fn exhaust_memory(_ctx: AppContext, _args: Value) -> WasmResult<Value> {
    trinci_sdk::log("Called method `exhaust_memory`");

    loop {
        let ptr = do_alloc(10000000);
        if ptr.is_null() {
            break;
        }
    }
    Ok(value!(null))
}

/// Infinite recursion test method
fn infinite_recursion(ctx: AppContext, args: Value) -> WasmResult<Value> {
    if let Value::Bool(first_call) = args {
        if first_call {
            trinci_sdk::log("Called method `null_pointer_indirection`");
        }
        return infinite_recursion(ctx, value!(false));
    }
    Ok(value!(null))
}

#[allow(unreachable_code)]
/// Infinite loop test method
fn infinite_loop(_ctx: AppContext, _args: Value) -> WasmResult<Value> {
    trinci_sdk::log("Called method `infinite_loop`");

    loop {
        std::thread::sleep(std::time::Duration::from_micros(1));
    }
    Ok(value!(null))
}

/// Null pointer indirection test method
fn null_pointer_indirection(_ctx: AppContext, _args: Value) -> WasmResult<Value> {
    trinci_sdk::log("Called method `null_pointer_indirection`");

    let mut_null_ptr: *mut u16 = std::ptr::null_mut();
    unsafe {
        *mut_null_ptr = 25;
    }
    Ok(value!(null))
}

/// Send a notification to the host.
fn notify(ctx: AppContext, data: Value) -> WasmResult<()> {
    trinci_sdk::emit_data_mp!(ctx.caller, ctx.method, &data)?;
    trinci_sdk::emit_data_mp!(ctx.caller, ctx.method, &[1, 2, 3])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use trinci_sdk::not_wasm;

    const OWNER_ID: &str = "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";
    const CALLER_ID: &str = OWNER_ID;
    const ASSET_ID: &str = OWNER_ID;

    fn create_generic_echo_args() -> Value {
        trinci_sdk::value!({
            "name": "Davide",
            "surname": "Galassi",
            "buf": (0x01, 0xFF, 0x80),
            "vec8": [ 0x01, 0xFF, 0x80 ],
            "vec16": [ 0x01, 0xFFFF, 0x8000 ],
            "map": {
                "k1": {
                    "field1": 123,
                    "field2": "foo",
                },
                "k2": {
                    "field1": 456,
                    "field2": "bar",
                },
                "k3": {
                    "field1": 789,
                    "field2": "baz",
                },
            }
        })
    }

    fn create_typed_echo_args() -> EchoArgs<'static> {
        let mut map = HashMap::new();
        map.insert(
            "k1",
            SubStruct {
                field1: 123,
                field2: "foo",
            },
        );
        map.insert(
            "k2",
            SubStruct {
                field1: 456,
                field2: "bar",
            },
        );
        map.insert(
            "k3",
            SubStruct {
                field1: 789,
                field2: "baz",
            },
        );
        EchoArgs {
            name: "Davide",
            surname: "Galassi".to_string(),
            buf: vec![0x01, 0xFF, 0x80],
            vec8: vec![0x01, 0xFF, 0x80],
            vec16: vec![0x01, 0xFFFF, 0x8000],
            map,
        }
    }

    #[test]
    fn echo_generic_args() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        let input = create_generic_echo_args();

        let output = not_wasm::call_wrap(echo_generic, ctx, input.clone()).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn echo_typed_args() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        let input = create_typed_echo_args();

        let output = not_wasm::call_wrap(echo_typed, ctx, input.clone()).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn echo_nested_call() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        not_wasm::set_contract_method(ctx.owner, "echo_packed", echo_packed);
        let input = value!(42u8);

        let output = not_wasm::call_wrap(nested_call, ctx, input.clone()).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn unknown_nested_call() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        not_wasm::set_contract_method(ctx.owner, "dummy", echo_packed);
        let input = value!(42u8);

        let err = not_wasm::call_wrap(nested_call, ctx, input).unwrap_err();

        assert_eq!(err.to_string(), "method not found");
    }

    #[test]
    fn asset_balance() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        not_wasm::set_account_asset(CALLER_ID, ASSET_ID, 3_u64.to_be_bytes().as_ref());

        let amount = not_wasm::call_wrap(balance, ctx, PackedValue::default()).unwrap();

        assert_eq!(amount, 3);
    }

    #[test]
    fn asset_transfer() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        not_wasm::set_account_asset(CALLER_ID, ASSET_ID, 9_u64.to_be_bytes().as_ref());
        let args = AssetTransferArgs {
            from: CALLER_ID,
            to: "abcdef",
            units: 3,
        };

        not_wasm::call_wrap(transfer, ctx, args).unwrap();

        let buf = not_wasm::get_account_asset(CALLER_ID, ASSET_ID);
        let buf: [u8; 8] = buf.try_into().unwrap_or_default();
        let value = u64::from_be_bytes(buf);
        assert_eq!(value, 6);

        let buf = not_wasm::get_account_asset("abcdef", ASSET_ID);
        let buf: [u8; 8] = buf.try_into().unwrap_or_default();
        let value = u64::from_be_bytes(buf);
        assert_eq!(value, 3);
    }

    #[test]
    fn asset_transfer_no_funds() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        not_wasm::set_account_asset(CALLER_ID, ASSET_ID, 9_u64.to_be_bytes().as_ref());
        let args = AssetTransferArgs {
            from: CALLER_ID,
            to: "abcdef",
            units: 10,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not enough funds");
    }

    #[test]
    fn notification() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        let args = trinci_sdk::value!({
            "message": "Hello!",
        });
        let res = not_wasm::call_wrap(notify, ctx, args).unwrap();

        assert_eq!(res, ());
    }
}
