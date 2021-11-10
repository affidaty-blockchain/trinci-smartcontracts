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

//! Time Oracle integration tests

use integration::{
    common::{self, AccountInfo, PUB_KEY1, PUB_KEY2, PVT_KEY1, PVT_KEY2},
    TestApp,
};
use lazy_static::lazy_static;
use serde_value::Value;
use std::collections::HashMap;
use trinci_core::{base::serialize, crypto::Hash, Receipt, Transaction};
use trinci_sdk::value;

lazy_static! {
    pub static ref TIME_ORACLE_APP_HASH: Hash = common::app_hash("time_oracle.wasm").unwrap();
}

const OWNER_ALIAS: &str = "Owner";
const USER_ALIAS: &str = "User";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(OWNER_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(USER_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map
    };
}

pub fn init_tx(owner_info: &AccountInfo) -> Transaction {
    let args = value!({
        "name": "Time Oracle",
        "description": "This will mark time in the blockchain",
        "update_interval": 3600,
        "initial_time": 1623429073_u64,
    });

    common::create_test_tx(
        &owner_info.id,
        &owner_info.pub_key,
        &owner_info.pvt_key,
        *TIME_ORACLE_APP_HASH,
        "init",
        args,
    )
}

fn get_config_tx(owner_info: &AccountInfo, user_info: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &owner_info.id,
        &user_info.pub_key,
        &user_info.pvt_key,
        *TIME_ORACLE_APP_HASH,
        "get_config",
        args,
    )
}

fn get_time_tx(owner_info: &AccountInfo, user_info: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &owner_info.id,
        &user_info.pub_key,
        &user_info.pvt_key,
        *TIME_ORACLE_APP_HASH,
        "get_time",
        args,
    )
}

fn update_tx(owner_info: &AccountInfo, user_info: &AccountInfo) -> Transaction {
    let args = 1623429879_u64;

    common::create_test_tx(
        &owner_info.id,
        &user_info.pub_key,
        &user_info.pvt_key,
        *TIME_ORACLE_APP_HASH,
        "update",
        value!(args),
    )
}

fn create_txs() -> Vec<Transaction> {
    let owner_info = ACCOUNTS_INFO.get(OWNER_ALIAS).unwrap();
    let user_info = ACCOUNTS_INFO.get(USER_ALIAS).unwrap();

    vec![
        // 0. Initialize oracle account.
        init_tx(owner_info),
        // 1. Get oracle configuration.
        get_config_tx(owner_info, user_info),
        // 2. Get oracle timestamp.
        get_time_tx(owner_info, user_info),
        // 3. Try to update the oracle from not authorized account. Shall fail.
        update_tx(owner_info, user_info),
        // 4. Update oracle from owned account.
        update_tx(owner_info, owner_info),
        // 5. Get oracle timestamp.
        get_time_tx(owner_info, user_info),
    ]
}

fn check_rxs(rxs: Vec<Receipt>) {
    // 0.
    assert!(rxs[0].success);
    // 1.
    assert!(rxs[1].success);
    let value: Value = serialize::rmp_deserialize(&rxs[1].returns).unwrap();
    let value = value.as_map().unwrap();
    let name = value.get(&value!("name")).unwrap().as_str().unwrap();
    assert_eq!(name, "Time Oracle");
    // 2.
    assert!(rxs[2].success);
    let time: u64 = serialize::rmp_deserialize(&rxs[2].returns).unwrap();
    assert_eq!(time, 1623429073);
    // 3.
    assert!(!rxs[3].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[3].returns),
        "smart contract fault: not authorized"
    );
    // 4.
    assert!(rxs[4].success);
    // 5.
    assert!(rxs[5].success);
    let time: u64 = serialize::rmp_deserialize(&rxs[5].returns).unwrap();
    assert_eq!(time, 1623429879);
}

#[test]
fn time_oracle_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);
}
