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

//! Crypto integration tests
use integration::common::SerdeValue;
use integration::{
    common::{self, AccountInfo, PUB_KEY1, PVT_KEY1},
    TestApp,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use trinci_sdk::{value, PackedValue};

use std::collections::HashMap;
use trinci_core::{base::serialize::rmp_deserialize, crypto::Hash, Receipt, Transaction};

lazy_static! {
    pub static ref TEST_APP_HASH: Hash = common::app_hash("test.wasm").unwrap();
}

const CALLER_ALIAS: &str = "Owner";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(CALLER_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map
    };
}

fn echo_generic_tx(account_info: &AccountInfo) -> Transaction {
    let args = value! ({
        "test": true,
        "value": 123,
    });

    common::create_test_tx(
        &account_info.id,
        &account_info.pub_key,
        &account_info.pvt_key,
        *TEST_APP_HASH,
        "echo_generic",
        args,
    )
}

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

fn create_echo_typed_args() -> EchoArgs<'static> {
    let mut map = HashMap::<&str, SubStruct>::new();

    map.insert(
        "a",
        SubStruct {
            field1: 42,
            field2: "skynet",
        },
    );

    EchoArgs {
        name: "John",
        surname: "Doe".to_string(),
        buf: vec![7, 11, 13],
        vec8: vec![1, 2, 5],
        vec16: vec![23, 37, 43],
        map,
    }
}
fn echo_typed_tx(account_info: &AccountInfo) -> Transaction {
    let args = create_echo_typed_args();

    common::create_test_tx(
        &account_info.id,
        &account_info.pub_key,
        &account_info.pvt_key,
        *TEST_APP_HASH,
        "echo_typed",
        args,
    )
}

fn echo_packed_tx(account_info: &AccountInfo) -> Transaction {
    let args = "hello".as_bytes().to_vec();

    common::create_test_tx(
        &account_info.id,
        &account_info.pub_key,
        &account_info.pvt_key,
        *TEST_APP_HASH,
        "echo_packed",
        args,
    )
}

fn create_txs() -> Vec<Transaction> {
    let caller_info = ACCOUNTS_INFO.get(CALLER_ALIAS).unwrap();

    vec![
        // 0. echo generic
        echo_generic_tx(caller_info),
        // 1. echo typed
        echo_typed_tx(caller_info),
        // 2. echo packed
        echo_packed_tx(caller_info),
    ]
}

fn check_rxs(rxs: Vec<Receipt>) {
    // 0. echo generic
    assert!(rxs[0].success);

    let res: SerdeValue = rmp_deserialize(&rxs[0].returns).unwrap();

    assert!(res.get(&value!("test")).unwrap().as_bool().unwrap());
    assert_eq!(res.get(&value!("value")).unwrap().as_i64().unwrap(), 123);

    // 1. echo typed
    assert!(rxs[1].success);

    let res: EchoArgs = trinci_sdk::rmp_deserialize(&rxs[1].returns).unwrap();

    assert_eq!(res, create_echo_typed_args());

    // 2. echo packed
    assert!(rxs[2].success);

    let res: Vec<u8> = trinci_sdk::rmp_deserialize(&rxs[2].returns).unwrap();

    assert_eq!("hello".as_bytes().to_vec(), res);
}

#[test]
fn crypto_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);
}
