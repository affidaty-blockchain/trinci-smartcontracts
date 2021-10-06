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

//! Storage integration test

use integration::{
    common::{
        self, AccountInfo, Asset, ASSET_APP_HASH, PUB_KEY1, PUB_KEY2, PUB_KEY3, PUB_KEY4, PVT_KEY1,
        PVT_KEY2, PVT_KEY3, PVT_KEY4,
    },
    TestApp,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use trinci_core::crypto::Hash;
use trinci_core::{base::serialize, Receipt, Transaction};
use trinci_sdk::value;

lazy_static! {
    pub static ref STORAGE_APP_HASH: Hash = common::app_hash("storage.wasm").unwrap();
}

const ASSET_ALIAS: &str = "FCK";
const ALICE_ALIAS: &str = "Alice";
const BOB_ALIAS: &str = "Bob";
const DAVE_ALIAS: &str = "Dave";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(ASSET_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(ALICE_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(BOB_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(DAVE_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map
    };
}

fn asset_init_tx(asset_info: &AccountInfo) -> Transaction {
    let args = value!({
        "name": ASSET_ALIAS,
        "authorized": Vec::<&str>::new(),
        "description": "My Cool Coin",
        "url": "https://fck.you",
        "max_units": 100_000,
    });
    common::create_test_tx(
        &asset_info.id,
        &asset_info.pub_key,
        &asset_info.pvt_key,
        *ASSET_APP_HASH,
        "init",
        args,
    )
}

fn asset_mint_tx(asset_info: &AccountInfo, to_info: &AccountInfo, units: u64) -> Transaction {
    let args = value!({
        "to": to_info.id,
        "units": units,
    });
    common::create_test_tx(
        &asset_info.id,
        &asset_info.pub_key,
        &asset_info.pvt_key,
        *ASSET_APP_HASH,
        "mint",
        args,
    )
}

fn transfer_tx(
    asset_info: &AccountInfo,
    from_info: &AccountInfo,
    to_info: &AccountInfo,
    units: u64,
) -> Transaction {
    let args = value!({
        "to": to_info.id,
        "asset": asset_info.id,
        "units": units,
    });
    common::create_test_tx(
        &from_info.id,
        &from_info.pub_key,
        &from_info.pvt_key,
        *STORAGE_APP_HASH,
        "transfer",
        args,
    )
}

fn create_payments_txs() -> Vec<Transaction> {
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let alice_info = ACCOUNTS_INFO.get(ALICE_ALIAS).unwrap();
    let bob_info = ACCOUNTS_INFO.get(BOB_ALIAS).unwrap();
    let dave_info = ACCOUNTS_INFO.get(DAVE_ALIAS).unwrap();
    vec![
        // 0. Asset initialization.
        asset_init_tx(asset_info),
        // 1. Mint some funds in Alice's account.
        asset_mint_tx(asset_info, alice_info, 100),
        // 2. Transfer from Alice to Bob.
        transfer_tx(asset_info, alice_info, bob_info, 10),
        // 3. Transfer from Bob to Dave.
        transfer_tx(asset_info, bob_info, dave_info, 3),
        // 4. Transfer from Dave to Alice.
        transfer_tx(asset_info, dave_info, alice_info, 1),
        // 5. Transfer from Alice to Bob. Shall fail for insufficient funds.
        transfer_tx(asset_info, alice_info, bob_info, 100),
    ]
}

fn check_payment_rxs(rxs: Vec<Receipt>) {
    // 0.
    assert!(rxs[0].success);
    // 1.
    assert!(rxs[1].success);
    // 2.
    assert!(rxs[2].success);
    // 3.
    assert!(rxs[3].success);
    // 4.
    assert!(rxs[4].success);
    // 5.
    assert!(!rxs[5].success);
    let msg = String::from_utf8_lossy(&rxs[5].returns);
    assert_eq!(
        msg,
        "smart contract fault: smart contract fault: insufficient funds"
    );
}

#[test]
fn simple_transfers() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_payments_txs();
    let rxs = app.exec_txs(txs);
    check_payment_rxs(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let alice_info = ACCOUNTS_INFO.get(ALICE_ALIAS).unwrap();
    let bob_info = ACCOUNTS_INFO.get(BOB_ALIAS).unwrap();
    let dave_info = ACCOUNTS_INFO.get(DAVE_ALIAS).unwrap();

    let account = app.account(&alice_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 91);

    let account = app.account(&bob_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 7);

    let account = app.account(&dave_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 2);
}

pub fn store_data_tx(to: &AccountInfo, key: &str, data: &[u8]) -> Transaction {
    let args = value!({
        "key": key,
        "data": serde_value::Value::Bytes(data.to_owned()),
    });

    common::create_test_tx(
        &to.id,
        &to.pub_key,
        &to.pvt_key,
        *STORAGE_APP_HASH,
        "store_data",
        args,
    )
}

pub fn load_data_tx(from: &AccountInfo, key: &str) -> Transaction {
    let args = value!({
        "key": key,
    });

    common::create_test_tx(
        &from.id,
        &from.pub_key,
        &from.pvt_key,
        *STORAGE_APP_HASH,
        "load_data",
        args,
    )
}

pub fn remove_data_tx(from: &AccountInfo, key: &str) -> Transaction {
    let args = value!({
        "key": key,
    });

    common::create_test_tx(
        &from.id,
        &from.pub_key,
        &from.pvt_key,
        *STORAGE_APP_HASH,
        "remove_data",
        args,
    )
}

fn create_data_management_txs() -> Vec<Transaction> {
    let alice_info = ACCOUNTS_INFO.get(ALICE_ALIAS).unwrap();
    vec![
        // 0. Store some data in Alice account.
        store_data_tx(alice_info, "data", &[0, 1, 2]),
        // 1. Overwrite data in Alice account.
        store_data_tx(alice_info, "data", &[1, 2, 3]),
        // 2. Load Alice's data.
        load_data_tx(alice_info, "data"),
        // 3. Remove Alice's data.
        remove_data_tx(alice_info, "data"),
        // 4. Load Alice's data (expected an empty buffer).
        load_data_tx(alice_info, "data"),
    ]
}

fn check_data_managements_rxs(rxs: Vec<Receipt>) {
    // 0
    assert_eq!(rxs[0].height, 0);
    assert_eq!(rxs[0].index, 0);
    assert!(rxs[0].success);
    // 1
    assert_eq!(rxs[1].height, 0);
    assert_eq!(rxs[1].index, 1);
    assert!(rxs[1].success);
    // 2
    assert_eq!(rxs[2].height, 0);
    assert_eq!(rxs[2].index, 2);
    assert!(rxs[2].success);
    assert_eq!(rxs[2].returns, vec![1, 2, 3]);
    // 3
    assert_eq!(rxs[3].height, 0);
    assert_eq!(rxs[3].index, 3);
    assert!(rxs[3].success);
    // 4
    assert_eq!(rxs[4].height, 0);
    assert_eq!(rxs[4].index, 4);
    assert!(rxs[4].success);
    assert!(rxs[4].returns.is_empty());
}

#[test]
fn account_data_management() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_data_management_txs();
    let rxs = app.exec_txs(txs);
    check_data_managements_rxs(rxs);
}
