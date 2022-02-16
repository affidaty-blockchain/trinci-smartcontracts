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

//! Test contract integration tests
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

const TEST_ALIAS: &str = "Test";
const TEST2_ALIAS: &str = "Test2";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(TEST_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(TEST2_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map
    };
}

lazy_static! {
    pub static ref TEST_APP_HASH: Hash = app_hash("test.wasm").unwrap();
    pub static ref TEST2_APP_HASH: Hash = app_hash("service.wasm").unwrap();
}

fn store_data_tx(test_info: &AccountInfo, key: &str, data: &[u8]) -> Transaction {
    let args = value!({
        "key": key,
        "data": data,
    });
    common::create_test_tx(
        &test_info.id,
        &test_info.pub_key,
        &test_info.pvt_key,
        *TEST_APP_HASH,
        "store_data",
        args,
    )
}

fn get_account_keys_tx(test_info: &AccountInfo, pattern: &str) -> Transaction {
    let args = pattern;
    common::create_test_tx(
        &test_info.id,
        &test_info.pub_key,
        &test_info.pvt_key,
        *TEST_APP_HASH,
        "get_account_keys",
        args,
    )
}

fn get_account_contract(test_info: &AccountInfo, account_id: &str) -> Transaction {
    let args = account_id;
    common::create_test_tx(
        &test_info.id,
        &test_info.pub_key,
        &test_info.pvt_key,
        *TEST_APP_HASH,
        "test_get_account_contract",
        args,
    )
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

fn echo_generic_tx(owner_info: &AccountInfo, user_info: &AccountInfo) -> Transaction {
    let args = value! ({
        "greet": "hello!"
    });

    common::create_test_tx(
        &owner_info.id,
        &user_info.pub_key,
        &user_info.pvt_key,
        *TEST_APP_HASH,
        "echo_generic",
        args,
    )
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

fn secure_call(
    test_info: &AccountInfo,
    account: &AccountInfo,
    contract: Vec<u8>,
    method: &str,
) -> Transaction {
    let args = SecureCallArgs {
        account: account.id.clone(),
        contract,
        method: method.to_string(),
        data: vec![1, 2, 3],
    };
    common::create_test_tx(
        &test_info.id,
        &test_info.pub_key,
        &test_info.pvt_key,
        *TEST_APP_HASH,
        "secure_call_test",
        args,
    )
}

fn create_txs() -> Vec<Transaction> {
    let test_info = ACCOUNTS_INFO.get(TEST_ALIAS).unwrap();
    let test2_info = ACCOUNTS_INFO.get(TEST2_ALIAS).unwrap();

    vec![
        // 0. Get keys with empty pattern. This shall fail
        get_account_keys_tx(test_info, ""),
        // 1. Get keys with wildcard pattern. This shall return an empty Vec
        get_account_keys_tx(test_info, "*"),
        // 2. Store some data
        store_data_tx(test_info, "abc", &vec![1, 2, 3]),
        // 3. Store some data
        store_data_tx(test_info, "abc:xyz", &vec![1, 2, 3]),
        // 4. Store some data
        store_data_tx(test_info, "xyz", &vec![1, 2, 3]),
        // 5. Get keys with bad pattern. This shall fail
        get_account_keys_tx(test_info, "abc"),
        // 6. Get keys with abc pattern.
        get_account_keys_tx(test_info, "abc*"),
        // 7. Store some data
        store_data_tx(test_info, "*", &vec![1, 2, 3]),
        // 8. Store some data
        store_data_tx(test_info, "abc*", &vec![1, 2, 3]),
        // 9. Store some data
        store_data_tx(test_info, "ab*xyz", &vec![1, 2, 3]),
        // 10. Get keys with ab pattern.
        get_account_keys_tx(test_info, "ab*"),
        // 11. Get keys with wildcard pattern.
        get_account_keys_tx(test_info, "*"),
        // 12. Get test account contract.
        get_account_contract(test_info, &test_info.id),
        // 13. Get test not existing account contract.
        get_account_contract(test_info, "not-existing"),
        // 14. Secure call on not existing account
        secure_call(
            test_info,
            test2_info,
            TEST_APP_HASH.as_bytes().to_vec(),
            "init",
        ),
        // 15. Secure call with wrong hash
        secure_call(
            test_info,
            test2_info,
            TEST2_APP_HASH.as_bytes().to_vec(),
            "init",
        ),
        // 16. Call Echo generic on the new account
        echo_generic_tx(test2_info, test_info),
    ]
}

fn check_basic_rxs(rxs: Vec<Receipt>) {
    // 0. Get keys with empty pattern. This shall fail
    assert!(!rxs[0].success);
    assert_eq!(
        "smart contract fault: last char of search pattern must be '*'",
        String::from_utf8_lossy(&rxs[0].returns)
    );
    // 1. Get keys with wildcard pattern. This shall return an empty Vec
    assert!(rxs[1].success);
    let res: Vec<String> = rmp_deserialize(&rxs[1].returns).unwrap();
    assert_eq!(res, Vec::<String>::new());
    // 2. Store some data
    assert!(rxs[2].success);
    // 3. Store some data
    assert!(rxs[3].success);
    // 4. Store some data
    assert!(rxs[4].success);
    // 5. Get keys with bad pattern. This shall fail
    assert!(!rxs[5].success);
    assert_eq!(
        "smart contract fault: last char of search pattern must be '*'",
        String::from_utf8_lossy(&rxs[5].returns)
    );

    // 6. Get keys with abc pattern.
    assert!(rxs[6].success);
    let mut res: Vec<String> = rmp_deserialize(&rxs[6].returns).unwrap();
    let mut expected = vec!["abc".to_string(), "abc:xyz".to_string()];
    res.sort();
    expected.sort();
    assert_eq!(res, expected);
    // 8. Store some data
    assert!(rxs[8].success);
    // 9. Store some data
    assert!(rxs[9].success);
    // 10. Get keys with ab pattern.
    assert!(rxs[10].success);
    let mut res: Vec<String> = rmp_deserialize(&rxs[10].returns).unwrap();
    let mut expected = vec![
        "ab*xyz".to_string(),
        "abc".to_string(),
        "abc*".to_string(),
        "abc:xyz".to_string(),
    ];
    res.sort();
    expected.sort();
    assert_eq!(res, expected);
    // 11. Get keys with wildcard pattern.
    assert!(rxs[11].success);
    let mut res: Vec<String> = rmp_deserialize(&rxs[11].returns).unwrap();
    let mut expected = vec![
        "abc".to_string(),
        "abc:xyz".to_string(),
        "xyz".to_string(),
        "*".to_string(),
        "abc*".to_string(),
        "ab*xyz".to_string(),
    ];
    res.sort();
    expected.sort();
    assert_eq!(res, expected);

    // 12. Get test account contract.
    assert!(rxs[12].success);
    let buf: Vec<u8> = rmp_deserialize(&rxs[12].returns).unwrap();
    let hash = Hash::from_bytes(&buf).unwrap();
    assert_eq!(*TEST_APP_HASH, hash);

    // 13. Get test not existing account contract.
    assert!(rxs[13].success);
    let buf: Vec<u8> = rmp_deserialize(&rxs[13].returns).unwrap();
    assert_eq!(buf, Vec::<u8>::new());

    // 14. Secure call on not existing account
    assert!(rxs[14].success);
    // 15. Secure call with wrong hash
    assert!(!rxs[15].success);
    assert_eq!(
        "smart contract fault: resource not found: incompatible contract app",
        String::from_utf8_lossy(&rxs[15].returns)
    );
    // 16. Call Echo generic on the new account
    assert!(rxs[16].success);
    let res: Value = rmp_deserialize(&rxs[16].returns).unwrap();
    let expected = value! ({
        "greet": "hello!"
    });
    assert_eq!(expected, res);
}

#[test]
fn test_contract() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_basic_rxs(rxs);
}
