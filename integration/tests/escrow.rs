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

use integration::{
    common::{
        self, AccountInfo, Asset, AssetLockArgs, LockType, ASSET_APP_HASH, PUB_KEY1, PUB_KEY2,
        PUB_KEY3, PUB_KEY4, PUB_KEY5, PUB_KEY6, PVT_KEY1, PVT_KEY2, PVT_KEY3, PVT_KEY4, PVT_KEY5,
        PVT_KEY6,
    },
    TestApp,
};

use lazy_static::lazy_static;
use std::collections::HashMap;
use trinci_core::{
    base::serialize::{self, rmp_deserialize},
    crypto::Hash,
    Receipt, Transaction,
};
use trinci_sdk::{value, Value};
lazy_static! {
    pub static ref ESCROW_APP_HASH: Hash = common::app_hash("escrow.wasm").unwrap();
}

const ASSET_ALIAS: &str = "FCK";
const GUARANTOR_ALIAS: &str = "Guarantor";
const CUSTOMER_ALIAS: &str = "Customer";
const MERCHANT1_ALIAS: &str = "Merchant1";
const MERCHANT2_ALIAS: &str = "Merchant2";
const ESCROW_ALIAS: &str = "Escrow";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(ASSET_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(GUARANTOR_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(CUSTOMER_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(MERCHANT1_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map.insert(MERCHANT2_ALIAS, AccountInfo::new(PUB_KEY5, PVT_KEY5));
        map.insert(ESCROW_ALIAS, AccountInfo::new(PUB_KEY6, PVT_KEY6));
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

fn asset_transfer_tx(
    asset_info: &AccountInfo,
    from_info: &AccountInfo,
    to_info: &AccountInfo,
    units: u64,
) -> Transaction {
    let args = value!({
        "from": from_info.id,
        "to": to_info.id,
        "units": units,
    });
    common::create_test_tx(
        &asset_info.id,
        &from_info.pub_key,
        &from_info.pvt_key,
        *ASSET_APP_HASH,
        "transfer",
        args,
    )
}

fn init_escrow_tx(
    asset_info: &AccountInfo,
    escrow_info: &AccountInfo,
    guarantor: &AccountInfo,
    customer: &AccountInfo,
    merchant1: &AccountInfo,
    merchant2: &AccountInfo,
) -> Transaction {
    // Initialization data
    let args = value!({
        "asset": asset_info.id,
        "guarantor": guarantor.id,
        "customer": customer.id,
        "merchants": {
            merchant1.id.clone(): 95,
            merchant2.id.clone(): 5,
        },
    });

    common::create_test_tx(
        &escrow_info.id,
        &escrow_info.pub_key,
        &escrow_info.pvt_key,
        *ESCROW_APP_HASH,
        "init",
        args,
    )
}

fn update_escrow_tx(
    escrow_info: &AccountInfo,
    guarantor: &AccountInfo,
    value: &str, // "OK" || "KO"
) -> Transaction {
    let args = value!({ "status": value });

    common::create_test_tx(
        &escrow_info.id,
        &guarantor.pub_key,
        &guarantor.pvt_key,
        *ESCROW_APP_HASH,
        "update",
        args,
    )
}

fn get_info_escrow_tx(escrow_info: &AccountInfo, caller: &AccountInfo) -> Transaction {
    let args = value!({});

    common::create_test_tx(
        &escrow_info.id,
        &caller.pub_key,
        &caller.pvt_key,
        *ESCROW_APP_HASH,
        "get_info",
        args,
    )
}

fn lock_tx(
    asset_info: &AccountInfo,
    who_info: &AccountInfo,
    where_info: &AccountInfo,
    lock_value: LockType,
) -> Transaction {
    let args = AssetLockArgs {
        to: &where_info.id,
        lock: lock_value,
    };

    common::create_test_tx(
        &asset_info.id,
        &who_info.pub_key,
        &who_info.pvt_key,
        *ASSET_APP_HASH,
        "lock",
        args,
    )
}

fn create_txs(status: &str) -> Vec<Transaction> {
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();
    let merchant1_info = ACCOUNTS_INFO.get(MERCHANT1_ALIAS).unwrap();
    let merchant2_info = ACCOUNTS_INFO.get(MERCHANT2_ALIAS).unwrap();

    vec![
        // 0. Initialize the asset.
        asset_init_tx(asset_info),
        // 1. Mint some units in customer account.
        asset_mint_tx(asset_info, customer_info, 1000),
        // 2. Transfer insufficient funds from customer to escrow account.
        asset_transfer_tx(asset_info, customer_info, escrow_info, 70),
        // 3. Initialize escrow account (this shall not fail despite the insufficient funds).
        init_escrow_tx(
            asset_info,
            escrow_info,
            guarantor_info,
            customer_info,
            merchant1_info,
            merchant2_info,
        ),
        // 4. Transfer enough funds for init to succeed.
        asset_transfer_tx(asset_info, customer_info, escrow_info, 130),
        // 5. Initialize escrow account.
        init_escrow_tx(
            asset_info,
            escrow_info,
            guarantor_info,
            customer_info,
            merchant1_info,
            merchant2_info,
        ),
        // 6. Get escrow info from not authorized user, shall fail.
        get_info_escrow_tx(escrow_info, asset_info),
        // 7. Get escrow info from authorized user.
        get_info_escrow_tx(escrow_info, merchant1_info),
        // 7. Try to unlock the asset, shall fail.
        lock_tx(asset_info, escrow_info, escrow_info, LockType::None),
        // 8. Try to transfer the funds using a direct asset transfer call, shall fail.
        asset_transfer_tx(asset_info, escrow_info, customer_info, 3),
        // 9. Update escrow.
        update_escrow_tx(escrow_info, guarantor_info, status),
        // 10. Second update, this is expected to fail.
        update_escrow_tx(escrow_info, guarantor_info, status),
        // 11. Transfer funds using a direct asset transfer call, after escrow is closed is ok.
        asset_transfer_tx(asset_info, escrow_info, customer_info, 3),
    ]
}

fn check_rxs(rxs: Vec<Receipt>) {
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();

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
    assert!(rxs[5].success);
    // 6. Get escrow info from not authorized user, shall fail.
    assert!(!rxs[6].success);
    let msg = String::from_utf8_lossy(&rxs[6].returns);
    assert_eq!(msg, "smart contract fault: not authorized");
    // 7. Get escrow info from authorized user.
    assert!(rxs[7].success);

    let res: Value = rmp_deserialize(&rxs[7].returns).unwrap();
    let config = res.get(&value!("config")).unwrap().as_map().unwrap();
    let guarantor = config.get(&value!("guarantor")).unwrap().as_str().unwrap();
    let amount = res.get(&value!("amount")).unwrap().as_u64().unwrap();
    let status = res.get(&value!("status")).unwrap().as_str().unwrap();

    assert_eq!(guarantor, guarantor_info.id);
    assert_eq!(amount, 200);
    assert_eq!(status, "open");

    // 8.
    assert!(!rxs[8].success);
    let msg = String::from_utf8_lossy(&rxs[8].returns);
    assert_eq!(msg, "smart contract fault: not authorized");
    // 9.
    assert!(!rxs[9].success);
    let msg = String::from_utf8_lossy(&rxs[9].returns);
    assert_eq!(
        msg,
        "smart contract fault: asset withdraw locked by contract"
    );
    // 10.
    assert!(rxs[10].success);
    // 11.
    assert!(!rxs[11].success);
    let msg = String::from_utf8_lossy(&rxs[11].returns);
    assert_eq!(msg, "smart contract fault: closed escrow");
    // 12.
    assert!(rxs[12].success);
}

fn create_txs_insufficient_funds(status: &str) -> Vec<Transaction> {
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();
    let merchant1_info = ACCOUNTS_INFO.get(MERCHANT1_ALIAS).unwrap();
    let merchant2_info = ACCOUNTS_INFO.get(MERCHANT2_ALIAS).unwrap();

    vec![
        // 0. Initialize the asset.
        asset_init_tx(asset_info),
        // 1. Mint some units in customer account.
        asset_mint_tx(asset_info, customer_info, 1000),
        // 2. Initialize escrow account (this shall not fail despite the insufficient funds).
        init_escrow_tx(
            asset_info,
            escrow_info,
            guarantor_info,
            customer_info,
            merchant1_info,
            merchant2_info,
        ),
        // 3. Transfer insufficient funds from customer to escrow account.
        asset_transfer_tx(asset_info, customer_info, escrow_info, 70),
        // 4. Try to unlock the asset, shall fail
        lock_tx(asset_info, escrow_info, escrow_info, LockType::None),
        // 5. Try to transfer the funds using a direct asset transfer call, shall fail.
        asset_transfer_tx(asset_info, escrow_info, customer_info, 3),
        // 6. Update escrow.
        update_escrow_tx(escrow_info, guarantor_info, status),
        // 7. Second update, this is expected to fail.
        update_escrow_tx(escrow_info, guarantor_info, status),
        // 8. Transfer funds using a direct asset transfer call, after escrow is closed is ok. - This shall fail for insufficient funds
        asset_transfer_tx(asset_info, escrow_info, customer_info, 3),
    ]
}

fn create_txs_no_funds(status: &str) -> Vec<Transaction> {
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();
    let merchant1_info = ACCOUNTS_INFO.get(MERCHANT1_ALIAS).unwrap();
    let merchant2_info = ACCOUNTS_INFO.get(MERCHANT2_ALIAS).unwrap();

    vec![
        // 0. Initialize the asset.
        asset_init_tx(asset_info),
        // 1. Mint some units in customer account.
        asset_mint_tx(asset_info, customer_info, 1000),
        // 2. Initialize escrow account (this shall not fail despite the insufficient funds).
        init_escrow_tx(
            asset_info,
            escrow_info,
            guarantor_info,
            customer_info,
            merchant1_info,
            merchant2_info,
        ),
        // 3. Try to unlock the asset, shall fail
        lock_tx(asset_info, escrow_info, escrow_info, LockType::None),
        // 4. Try to transfer the funds using a direct asset transfer call, shall fail.
        asset_transfer_tx(asset_info, escrow_info, customer_info, 3),
        // 5. Update escrow, this is expected to fail.
        update_escrow_tx(escrow_info, guarantor_info, status),
        // 6. Second update, this is expected to fail.
        update_escrow_tx(escrow_info, guarantor_info, status),
        // 7. Transfer funds using a direct asset transfer call, after escrow is closed is ok. - This shall fail for insufficient funds
        asset_transfer_tx(asset_info, escrow_info, customer_info, 3),
    ]
}

fn check_rxs_complete_with_insufficient_funds(rxs: Vec<Receipt>) {
    // 0. Initialize the asset.
    assert!(rxs[0].success);
    // 1. Mint some units in customer account.
    assert!(rxs[1].success);
    // 2. Initialize escrow account (this shall not fail despite the insufficient funds).
    assert!(rxs[2].success);
    // 3. Transfer insufficient funds from customer to escrow account.
    assert!(rxs[3].success);
    // 4. Try to unlock the asset, shall fail
    assert!(!rxs[4].success);
    let msg = String::from_utf8_lossy(&rxs[4].returns);
    assert_eq!(msg, "smart contract fault: not authorized");
    // 5. Try to transfer the funds using a direct asset transfer call, shall fail.
    assert!(!rxs[5].success);
    let msg = String::from_utf8_lossy(&rxs[5].returns);
    assert_eq!(
        msg,
        "smart contract fault: asset withdraw locked by contract"
    );
    // 6. Update escrow,this is expected to fail.
    assert!(!rxs[6].success);
    let msg = String::from_utf8_lossy(&rxs[7].returns);
    assert_eq!(msg, "smart contract fault: insufficient funds");
    // 7. Second update, this is expected to fail.
    assert!(!rxs[7].success);
    let msg = String::from_utf8_lossy(&rxs[7].returns);
    assert_eq!(msg, "smart contract fault: insufficient funds");
    // 8. Transfer funds using a direct asset transfer call, after escrow is closed is ok. - This shall fail
    assert!(!rxs[8].success);
    let msg = String::from_utf8_lossy(&rxs[8].returns);
    assert_eq!(
        msg,
        "smart contract fault: asset withdraw locked by contract"
    );
}

fn check_rxs_refund_with_insufficient_funds(rxs: Vec<Receipt>) {
    // 0. Initialize the asset.
    assert!(rxs[0].success);
    // 1. Mint some units in customer account.
    assert!(rxs[1].success);
    // 2. Initialize escrow account (this shall not fail despite the insufficient funds).
    assert!(rxs[2].success);
    // 3. Transfer insufficient funds from customer to escrow account.
    assert!(rxs[3].success);
    // 4. Try to unlock the asset, shall fail
    assert!(!rxs[4].success);
    let msg = String::from_utf8_lossy(&rxs[4].returns);
    assert_eq!(msg, "smart contract fault: not authorized");
    // 5. Try to transfer the funds using a direct asset transfer call, shall fail.
    assert!(!rxs[5].success);
    let msg = String::from_utf8_lossy(&rxs[5].returns);
    assert_eq!(
        msg,
        "smart contract fault: asset withdraw locked by contract"
    );
    // 6. Update escrow.
    assert!(rxs[6].success);
    // 7. Second update, this is expected to fail.
    assert!(!rxs[7].success);
    let msg = String::from_utf8_lossy(&rxs[7].returns);
    assert_eq!(msg, "smart contract fault: closed escrow");
    // 8. Transfer funds using a direct asset transfer call, after escrow is closed is ok. - This shall fail for insufficient funds
    assert!(!rxs[8].success);
    let msg = String::from_utf8_lossy(&rxs[8].returns);
    assert_eq!(msg, "smart contract fault: insufficient funds");
}

fn check_rxs_refund_with_no_funds(rxs: Vec<Receipt>) {
    // 0. Initialize the asset.
    assert!(rxs[0].success);
    // 1. Mint some units in customer account.
    assert!(rxs[1].success);
    // 2. Initialize escrow account (this shall not fail despite the insufficient funds).
    assert!(rxs[2].success);
    // 3. Try to unlock the asset, shall fail
    assert!(!rxs[3].success);
    let msg = String::from_utf8_lossy(&rxs[3].returns);
    assert_eq!(msg, "smart contract fault: not authorized");
    // 4. Try to transfer the funds using a direct asset transfer call, shall fail.
    assert!(!rxs[4].success);
    let msg = String::from_utf8_lossy(&rxs[4].returns);
    assert_eq!(
        msg,
        "smart contract fault: asset withdraw locked by contract"
    );
    // 5. Update escrow.
    assert!(rxs[5].success);
    // 6. Second update, this is expected to fail.
    assert!(!rxs[6].success);
    let msg = String::from_utf8_lossy(&rxs[6].returns);
    assert_eq!(msg, "smart contract fault: closed escrow");
    // 7. Transfer funds using a direct asset transfer call, after escrow is closed is ok. - This shall fail for insufficient funds
    assert!(!rxs[7].success);
    let msg = String::from_utf8_lossy(&rxs[7].returns);
    assert_eq!(msg, "smart contract fault: insufficient funds");
}

#[test]
fn escrow_complete() {
    // Instance application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs("OK");
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();
    let merchant1_info = ACCOUNTS_INFO.get(MERCHANT1_ALIAS).unwrap();
    let merchant2_info = ACCOUNTS_INFO.get(MERCHANT2_ALIAS).unwrap();

    let account = app.account(&escrow_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 97);

    let account = app.account(&customer_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 803);

    let account = app.account(&merchant1_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 95);

    let account = app.account(&merchant2_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 5);
}

#[test]
fn escrow_refund() {
    // Instance application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs("KO");
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();

    let account = app.account(&escrow_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 97);

    let account = app.account(&customer_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 903);
}

#[test]
fn escrow_complete_with_insufficient_funds() {
    // Instance application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs_insufficient_funds("OK");
    let rxs = app.exec_txs(txs);
    check_rxs_complete_with_insufficient_funds(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();

    let account = app.account(&escrow_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 70);

    let account = app.account(&customer_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 930);
}

#[test]
fn escrow_refund_with_insufficient_funds() {
    // Instance application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs_insufficient_funds("KO");
    let rxs = app.exec_txs(txs);
    check_rxs_refund_with_insufficient_funds(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();

    let account = app.account(&escrow_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 0);

    let account = app.account(&customer_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 1000);
}

#[test]
fn escrow_refund_with_no_funds() {
    // Instance application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs_no_funds("KO");
    let rxs = app.exec_txs(txs);
    check_rxs_refund_with_no_funds(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();

    let account = app.account(&escrow_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 0);

    let account = app.account(&customer_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 1000);
}
