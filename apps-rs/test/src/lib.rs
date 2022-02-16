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
//!
use random::Source;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use std::{
    alloc::{alloc, Layout},
    convert::TryInto,
    mem::align_of,
};
use trinci_sdk::{
    rmp_deserialize, rmp_serialize, tai::AssetTransferArgs, value, AppContext, PackedValue, Value,
    WasmError, WasmResult,
};
trinci_sdk::app_export!(
    init,
    // Input and output serialization.
    echo_generic,
    echo_typed,
    echo_packed,
    // Default smart contract functionalities.
    nested_call,
    balance,
    transfer,
    notify,
    // Utility methods
    mint,
    store_data,
    // Host function tests
    get_account_keys,
    test_sha256,
    test_get_account_contract,
    secure_call_test,
    // Trigger exceptional conditions.
    divide_by_zero,
    trigger_panic,
    exhaust_memory,
    infinite_recursion,
    infinite_loop,
    null_pointer_indirection,
    // Deterministic contract
    get_random_sequence,
    get_hashmap,
    get_time
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

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq, Clone, Default))]
struct StoreDataArgs<'a> {
    pub key: &'a str,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq, Clone, Default))]
struct SecureCallArgs {
    account: String,
    #[serde(with = "serde_bytes")]
    pub contract: Vec<u8>,
    method: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

/// Only binds the contract to an account
fn init(_ctx: AppContext, _args: PackedValue) -> WasmResult<()> {
    Ok(())
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

/// Mint some `account asset` units on the account
fn mint(ctx: AppContext, args: u64) -> WasmResult<u64> {
    trinci_sdk::log("Called method `mint`");

    let value = load_my_asset(ctx.owner);
    store_my_asset(ctx.owner, value + args);

    Ok(value + args)
}

/// Store the data with the given key in the current account
fn store_data(_ctx: AppContext, args: StoreDataArgs) -> WasmResult<()> {
    trinci_sdk::store_account_data_mp!(args.key, &args.data)
}

/// Call the host function hf_get_data_keys
fn get_account_keys(_ctx: AppContext, pattern: &str) -> WasmResult<Vec<String>> {
    trinci_sdk::get_data_keys(pattern)
}

/// Call the host function hf_sha256
fn test_sha256(_ctx: AppContext, data: PackedValue) -> WasmResult<Vec<u8>> {
    Ok(trinci_sdk::sha256(&data.0))
}

/// Call the host function hf_get_account_contract
fn test_get_account_contract(_ctx: AppContext, account_id: &str) -> WasmResult<Vec<u8>> {
    Ok(trinci_sdk::get_account_contract(account_id))
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

/// Secure call host function method
fn secure_call_test(_ctx: AppContext, args: SecureCallArgs) -> WasmResult<Vec<u8>> {
    trinci_sdk::log("Called method `secure_call_test`");
    trinci_sdk::s_call(&args.account, &args.contract, &args.method, &args.data)
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
            trinci_sdk::log("Called method `infinite_recursion`");
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
    trinci_sdk::emit_data_mp!("event_a", &data)?;
    trinci_sdk::emit_data_mp!(ctx.caller, &[1, 2, 3])?;
    Ok(())
}

/// Return a random sequence (shall be deterministic)
fn get_random_sequence(_ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    trinci_sdk::log("Called method `random_sequence`");

    let mut source = random::default();
    let vector = source.iter().take(3).collect::<Vec<u64>>();

    let buf = trinci_sdk::rmp_serialize(&vector)?;
    Ok(PackedValue(buf))
}

/// Return an hashmap (shall be deterministic)
fn get_hashmap(_ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    trinci_sdk::log("Called method `return_hashmap`");

    let mut hashmap: HashMap<&str, u64> = HashMap::default();

    hashmap.insert("val1", 123);
    hashmap.insert("val2", 456);
    hashmap.insert("val3", 789);

    let buf = trinci_sdk::rmp_serialize(&hashmap)?;
    Ok(PackedValue(buf))
}

/// Try to access to system time.
fn get_time(_ctx: AppContext, _args: PackedValue) -> WasmResult<u64> {
    trinci_sdk::log("Called method `get_time`");

    let sys_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    Ok(sys_time.as_secs())
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
        let mut map = HashMap::default();
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

    #[test]
    fn store_data_test() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        let args = StoreDataArgs {
            key: "foo",
            data: vec![1, 2, 3],
        };
        let res = not_wasm::call_wrap(store_data, ctx, args).unwrap();

        assert_eq!(res, ());
        let buf = not_wasm::get_account_data(OWNER_ID, "foo");

        let data: Vec<u8> = rmp_deserialize(&buf).unwrap();
        let expected = vec![1, 2, 3];

        assert_eq!(data, expected);
    }

    #[test]
    fn get_keys_with_empty_pattern() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        let args = "";
        let err = not_wasm::call_wrap(get_account_keys, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "last char of search pattern must be '*'");
    }

    #[test]
    fn get_keys_with_invalid_pattern_1() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        let args = "x";
        let err = not_wasm::call_wrap(get_account_keys, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "last char of search pattern must be '*'");
    }

    #[test]
    fn get_keys_with_invalid_pattern_2() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        let args = "abc:*sdfsdf";
        let err = not_wasm::call_wrap(get_account_keys, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "last char of search pattern must be '*'");
    }

    #[test]
    fn get_all_keys_not_existent() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        let args = "*";
        let res = not_wasm::call_wrap(get_account_keys, ctx, args).unwrap();

        assert_eq!(res, Vec::<String>::new());
    }

    #[test]
    fn get_keys_not_existent() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        let args = "key:*";
        let res = not_wasm::call_wrap(get_account_keys, ctx, args).unwrap();

        assert_eq!(res, Vec::<String>::new());
    }

    #[test]
    fn test_sha256_ok() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        let args = PackedValue(vec![1, 2, 3]);

        let res = not_wasm::call_wrap(test_sha256, ctx, args).unwrap();

        let expected = [
            3, 144, 88, 198, 242, 192, 203, 73, 44, 83, 59, 10, 77, 20, 239, 119, 204, 15, 120,
            171, 204, 206, 213, 40, 125, 132, 161, 162, 1, 28, 251, 129,
        ];

        assert_eq!(res, expected);
    }

    #[test]
    fn get_not_existing_account_contract_test() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        not_wasm::set_account_contract(OWNER_ID, vec![1, 2, 3]);
        let args = "not-existing-account";

        let res = not_wasm::call_wrap(test_get_account_contract, ctx, args).unwrap();

        assert_eq!(res, Vec::<u8>::new());
    }

    #[test]
    fn get_existing_account_contract_test() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        not_wasm::set_account_contract(OWNER_ID, vec![1, 2, 3]);
        let args = OWNER_ID;

        let res = not_wasm::call_wrap(test_get_account_contract, ctx, args).unwrap();

        assert_eq!(res, vec![1, 2, 3]);
    }

    #[test]
    fn get_keys_with_pattern() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        not_wasm::set_account_data(OWNER_ID, "abc", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "abc1", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "abc*:123", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "xyz", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "ab", &[1, 2, 3]);

        let args = "abc*";
        let mut res = not_wasm::call_wrap(get_account_keys, ctx, args).unwrap();
        res.sort();
        let mut expected = vec![
            "abc".to_string(),
            "abc1".to_string(),
            "abc*:123".to_string(),
        ];
        expected.sort();
        assert_eq!(res, expected);
    }

    #[test]
    fn get_keys_with_wildcard() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        not_wasm::set_account_data(OWNER_ID, "abc", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "abc1", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "abc*:123", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "xyz", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "ab", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "*", &[1, 2, 3]);

        let args = "*";
        let mut res = not_wasm::call_wrap(get_account_keys, ctx, args).unwrap();
        res.sort();
        let mut expected = vec![
            "abc".to_string(),
            "abc1".to_string(),
            "abc*:123".to_string(),
            "xyz".to_string(),
            "ab".to_string(),
            "*".to_string(),
        ];
        expected.sort();
        assert_eq!(res, expected);
    }

    #[test]
    fn get_keys_with_wrong_pattern() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        not_wasm::set_account_data(OWNER_ID, "abc", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "abc1", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "abc*:123", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "xyz", &[1, 2, 3]);
        not_wasm::set_account_data(OWNER_ID, "ab", &[1, 2, 3]);

        let args = "hello*";
        let res = not_wasm::call_wrap(get_account_keys, ctx, args).unwrap();

        assert_eq!(res, Vec::<String>::new());
    }

    #[test]
    fn test_secure_call() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        not_wasm::set_contract_method("dummy", "get", echo_packed);
        not_wasm::set_contract_hash("dummy", &vec![1, 2, 3]);
        let args = SecureCallArgs {
            account: "dummy".to_string(),
            contract: vec![1, 2, 3],
            method: "get".to_string(),
            data: vec![0, 5, 7],
        };
        let res = not_wasm::call_wrap(secure_call_test, ctx, args).unwrap();
        assert_eq!(res, [0, 5, 7]);
    }

    #[test]
    fn test_secure_call_invalid() {
        let ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);

        let args = SecureCallArgs {
            account: "dummy".to_string(),
            contract: vec![1, 2, 3],
            method: "get".to_string(),
            data: vec![1, 2, 3],
        };
        let err = not_wasm::call_wrap(secure_call_test, ctx, args).unwrap_err();
        assert_eq!(err.to_string(), "incompatible contract app");
    }
}
