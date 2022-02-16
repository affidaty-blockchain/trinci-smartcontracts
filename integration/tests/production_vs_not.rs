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

//! Testnet vs Production integration test

use integration::{
    common::{self, *},
    TestApp,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_value::Value;
use std::collections::HashMap;
use trinci_core::{base::serialize::rmp_deserialize, crypto::Hash};
use trinci_core::{Receipt, Transaction};

const HASH_HEX: &str = "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81"; // [1,2,3] sha256

const TEST_ALIAS: &str = "Test";
const CRYPTO_ALIAS: &str = "Crypto";
const CONTRACT_ALIAS: &str = "Contract";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(CONTRACT_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(CRYPTO_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(TEST_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map
    };
}

lazy_static! {
    pub static ref TEST_APP_HASH: Hash = app_hash("test.wasm").unwrap();
    pub static ref CRYPTO_APP_HASH: Hash = app_hash("crypto.wasm").unwrap();
}

// Hash Algorithms available
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
}
// Hash arguments
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct HashArgs<'a> {
    pub algorithm: HashAlgorithm,
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
}

fn init_tx(owner: &AccountInfo, target: Hash) -> Transaction {
    let args = value!({});

    common::create_test_tx(
        &owner.id,
        &owner.pub_key,
        &owner.pvt_key,
        target,
        "init",
        args,
    )
}

fn hash_tx(owner: &AccountInfo, target: Hash) -> Transaction {
    let args = HashArgs {
        algorithm: HashAlgorithm::Sha256,
        data: &[1, 2, 3],
    };

    common::create_test_tx(
        &owner.id,
        &owner.pub_key,
        &owner.pvt_key,
        target,
        "hash",
        args,
    )
}

fn echo_generic_tx(owner: &AccountInfo, target: Hash) -> Transaction {
    let args = value! ({
        "greet": "hello!"
    });

    common::create_test_tx(
        &owner.id,
        &owner.pub_key,
        &owner.pvt_key,
        target,
        "echo_generic",
        args,
    )
}

fn create_txs() -> Vec<Transaction> {
    let contract_info = ACCOUNTS_INFO.get(CONTRACT_ALIAS).unwrap();

    vec![
        // 0. Init Test on Account
        init_tx(contract_info, *TEST_APP_HASH),
        // 1. Call echo on Account
        echo_generic_tx(contract_info, *TEST_APP_HASH),
        // 2. Init Crypto on Account
        init_tx(contract_info, *CRYPTO_APP_HASH),
        // 3. Call hash on Account
        hash_tx(contract_info, *CRYPTO_APP_HASH),
        // 4. Call echo on Account
        echo_generic_tx(contract_info, *TEST_APP_HASH),
    ]
}

fn check_production_rxs(rxs: Vec<Receipt>) {
    // 0. Init Test on Account
    assert!(rxs[0].success);
    // 1. Call echo on Account
    assert!(rxs[1].success);
    let res: Value = rmp_deserialize(&rxs[1].returns).unwrap();
    let expected = value! ({
        "greet": "hello!"
    });
    assert_eq!(expected, res);
    // 2. Init Crypto on Account
    assert!(!rxs[2].success);
    assert_eq!(
        "resource not found: incompatible contract app",
        String::from_utf8_lossy(&rxs[2].returns)
    );
    // 3. Call hash on Account
    assert!(!rxs[3].success);
    assert_eq!(
        "resource not found: incompatible contract app",
        String::from_utf8_lossy(&rxs[3].returns)
    );
    // 4. Call echo on Account
    assert!(rxs[4].success);
    let res: Value = rmp_deserialize(&rxs[4].returns).unwrap();
    let expected = value! ({
        "greet": "hello!"
    });
    assert_eq!(expected, res);
}

fn check_not_production_rxs(rxs: Vec<Receipt>) {
    // 0. Init Test on Account
    assert!(rxs[0].success);
    // 1. Call echo on Account
    assert!(rxs[1].success);
    let res: Value = rmp_deserialize(&rxs[1].returns).unwrap();
    let expected = value! ({
        "greet": "hello!"
    });
    assert_eq!(expected, res);
    // 2. Init Crypto on Account
    assert!(rxs[2].success);
    // 3. Call hash on Account
    assert!(rxs[3].success);
    let res: Vec<u8> = rmp_deserialize(&rxs[3].returns).unwrap();
    assert_eq!(HASH_HEX, hex::encode(&res));
    // 4. Call echo on Account
    assert!(rxs[4].success);
    let res: Value = rmp_deserialize(&rxs[4].returns).unwrap();
    let expected = value! ({
        "greet": "hello!"
    });
    assert_eq!(expected, res);
}

#[test]
fn test_contract_production() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_production_rxs(rxs);
}

#[test]
fn test_contract_not_production() {
    // Instance the application.
    let mut app = TestApp::new(&common::apps_path(), false);

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_not_production_rxs(rxs);
}
