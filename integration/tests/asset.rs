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

//! Basic asset integration tests
use integration::{
    common::{self, *},
    TestApp,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use trinci_core::{
    base::serialize::{self},
    Receipt, Transaction,
};

const ASSET_ALIAS: &str = "FCK";
const ALICE_ALIAS: &str = "Alice";
const BOB_ALIAS: &str = "Bob";
const DAVE_ALIAS: &str = "Dave";
const CRYPTO_ALIAS: &str = "Dave";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(ASSET_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(ALICE_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(BOB_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(DAVE_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map.insert(CRYPTO_ALIAS, AccountInfo::new(PUB_KEY5, PVT_KEY5));
        map
    };
}

fn init_tx(asset_info: &AccountInfo) -> Transaction {
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

fn mint_tx(asset_info: &AccountInfo, to_info: &AccountInfo, units: u64) -> Transaction {
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

fn burn_tx(asset_info: &AccountInfo, from_info: &AccountInfo, units: u64) -> Transaction {
    let args = value!({
        "from": from_info.id,
        "units": units,
    });
    common::create_test_tx(
        &asset_info.id,
        &asset_info.pub_key,
        &asset_info.pvt_key,
        *ASSET_APP_HASH,
        "burn",
        args,
    )
}

fn balance_tx(asset_info: &AccountInfo, from_info: &AccountInfo) -> Transaction {
    common::create_test_tx(
        &asset_info.id,
        &from_info.pub_key,
        &from_info.pvt_key,
        *ASSET_APP_HASH,
        "balance",
        value!(null),
    )
}

fn transfer_tx(
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

// This is signed by the destination account
fn transfer_delegate_tx(
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
        &to_info.pub_key,
        &to_info.pvt_key,
        *ASSET_APP_HASH,
        "transfer",
        args,
    )
}

// /// Add Delegation method arguments
fn add_delegation_tx(
    asset_info: &AccountInfo,
    delegator_info: &AccountInfo,
    delegate_info: &AccountInfo,
) -> Transaction {
    let args = value!({
        "to": null,
        "units": 1,
        "delegate": delegate_info.id
    });

    common::create_test_tx(
        &asset_info.id,
        &delegator_info.pub_key,
        &delegator_info.pvt_key,
        *ASSET_APP_HASH,
        "add_delegation",
        args,
    )
}

fn stats_tx(asset_info: &AccountInfo, caller_info: &AccountInfo) -> Transaction {
    common::create_test_tx(
        &asset_info.id,
        &caller_info.pub_key,
        &caller_info.pvt_key,
        *ASSET_APP_HASH,
        "stats",
        value!(null),
    )
}

fn create_basic_txs() -> Vec<Transaction> {
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let alice_info = ACCOUNTS_INFO.get(ALICE_ALIAS).unwrap();
    let bob_info = ACCOUNTS_INFO.get(BOB_ALIAS).unwrap();
    let dave_info = ACCOUNTS_INFO.get(DAVE_ALIAS).unwrap();

    vec![
        // 0. Asset initialization.
        init_tx(asset_info),
        // 1. Mint asset in Alice's account.
        mint_tx(asset_info, alice_info, 10),
        // 2. Mint asset in Bob's account.
        mint_tx(asset_info, bob_info, 3),
        // 3. Get Alice's balance.
        balance_tx(asset_info, alice_info),
        // 4. Transfer from Alice to Bob account.
        transfer_tx(asset_info, alice_info, bob_info, 1),
        // 5. Get Alice's balance.
        balance_tx(asset_info, alice_info),
        // 6. Get Bob's balance.
        balance_tx(asset_info, bob_info),
        // 7. Transfer from Bob to Dave account.
        transfer_tx(asset_info, bob_info, dave_info, 1),
        // 8. Get Dave's balance.
        balance_tx(asset_info, dave_info),
        // 9. Burn from Alice's account.
        burn_tx(asset_info, alice_info, 1),
        // 10. Alice asks for asset's stats.
        stats_tx(asset_info, alice_info),
        // 11. Try to mint too much. Shall fail.
        mint_tx(asset_info, bob_info, 200000),
        // 12. Try to burn too much. Shall fail.
        burn_tx(asset_info, alice_info, 200000),
        // 13. Try to burn too much. Shall fail.
        burn_tx(asset_info, alice_info, 9),
        // 14. Bob adds a delegation to allow Dave to transfer from his account
        add_delegation_tx(asset_info, bob_info, dave_info),
        // 15. Transfer 1 asset from Bob to Dave account. The caller is Dave.
        transfer_delegate_tx(asset_info, bob_info, dave_info, 1),
        // 16. Transfer again 1 asset from Bob to Dave account.
        // The caller is Dave and the delegation is already been spent. This shall fail
        transfer_delegate_tx(asset_info, bob_info, dave_info, 1),
    ]
}

fn check_basic_rxs(rxs: Vec<Receipt>) {
    // 0.
    assert!(rxs[0].success);
    // 1.
    assert!(rxs[1].success);
    // 2.
    assert!(rxs[2].success);
    // 3.
    assert!(rxs[3].success);
    let value: u64 = serialize::rmp_deserialize(&rxs[3].returns).unwrap();
    assert_eq!(value, 10);
    // 4.
    assert!(rxs[4].success);
    // 5.
    assert!(rxs[5].success);
    let value: u64 = serialize::rmp_deserialize(&rxs[5].returns).unwrap();
    assert_eq!(value, 9);
    // 6.
    assert!(rxs[6].success);
    let value: u64 = serialize::rmp_deserialize(&rxs[6].returns).unwrap();
    assert_eq!(value, 4);
    // 7.
    assert!(rxs[7].success);
    // 8.
    assert!(rxs[8].success);
    let value: u64 = serialize::rmp_deserialize(&rxs[8].returns).unwrap();
    assert_eq!(value, 1);
    // 9.
    assert!(rxs[9].success);
    // 10.
    assert!(rxs[10].success);
    let stats: SerdeValue = serialize::rmp_deserialize(&rxs[10].returns).unwrap();
    let map = stats.as_map().unwrap();
    assert_eq!(map.get(&value!("minted")).unwrap().as_u64().unwrap(), 13);
    assert_eq!(map.get(&value!("burned")).unwrap().as_u64().unwrap(), 1);
    // 11.
    assert!(!rxs[11].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[11].returns),
        "smart contract fault: minting overcome the max_units value"
    );
    // 12.
    assert!(!rxs[12].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[12].returns),
        "smart contract fault: insufficient funds"
    );
    // Tx 13
    assert!(!rxs[13].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[13].returns),
        "smart contract fault: insufficient funds"
    );
    // 14. Bob adds a delegation to allow Dave to transfer from his account
    assert!(rxs[14].success);

    // 15. Transfer 1 asset from Bob to Dave account. The caller is Dave.
    assert!(rxs[15].success);
    // 16. Transfer again 1 asset from Bob to Dave account.
    // The caller is Dave and the delegation is already been spent. This shall fail
    assert!(!rxs[16].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[16].returns),
        "smart contract fault: not authorized"
    );
}

#[test]
fn basic_operations() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_basic_txs();
    let rxs = app.exec_txs(txs);
    check_basic_rxs(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let alice_info = ACCOUNTS_INFO.get(ALICE_ALIAS).unwrap();
    let bob_info = ACCOUNTS_INFO.get(BOB_ALIAS).unwrap();
    let dave_info = ACCOUNTS_INFO.get(DAVE_ALIAS).unwrap();

    let account = app.account(&alice_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 8);
    assert_eq!(asset.lock, None);

    let account = app.account(&bob_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 2);
    assert_eq!(asset.lock, None);

    let account = app.account(&dave_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 2);
    assert_eq!(asset.lock, None);
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

    create_test_tx(
        &asset_info.id,
        &who_info.pub_key,
        &who_info.pvt_key,
        *ASSET_APP_HASH,
        "lock",
        args,
    )
}

fn create_locking_txs() -> Vec<Transaction> {
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let alice_info = ACCOUNTS_INFO.get(ALICE_ALIAS).unwrap();
    let bob_info = ACCOUNTS_INFO.get(BOB_ALIAS).unwrap();

    vec![
        // 0. Initialization
        init_tx(asset_info),
        // 1. Mint on Alice account
        mint_tx(asset_info, alice_info, 10),
        // 2. Creator locks Alice asset.
        lock_tx(asset_info, asset_info, alice_info, LockType::Full),
        // 3. Alice tries to transfer to Bob while locked, shall fail.
        transfer_tx(asset_info, alice_info, bob_info, 3),
        // 4. Alice tries to unlock, shall fail.
        lock_tx(asset_info, alice_info, alice_info, LockType::None),
        // 5. Creator unlocks Alice asset.
        lock_tx(asset_info, asset_info, alice_info, LockType::None),
        // 6. Alice transfer some funds to Bob.
        transfer_tx(asset_info, alice_info, bob_info, 3),
        // 7. Alice locks the asset.
        lock_tx(asset_info, alice_info, alice_info, LockType::Full),
        // 8. Alice tries to transfer to Bob while locked, shall fail.
        transfer_tx(asset_info, alice_info, bob_info, 3),
        // 9. Bob tries to transfer to Alice while she's locked, shall fail as well.
        transfer_tx(asset_info, bob_info, alice_info, 3),
        // 10. Alice unlocks the asset.
        lock_tx(asset_info, alice_info, alice_info, LockType::None),
        // 11. Now Bob can transfer to Alice.
        transfer_tx(asset_info, bob_info, alice_info, 1),
    ]
}

fn check_locking_rxs(rxs: Vec<Receipt>) {
    // 0.
    assert!(rxs[0].success);
    // 1.
    assert!(rxs[1].success);
    // 2.
    assert!(rxs[2].success);
    // 3.
    assert!(!rxs[3].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[3].returns),
        "smart contract fault: asset withdraw locked by creator"
    );
    // 4.
    assert!(!rxs[4].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[4].returns),
        "smart contract fault: not authorized"
    );
    // 5.
    assert!(rxs[5].success);
    // 6.
    assert!(rxs[6].success);
    // 7.
    assert!(rxs[7].success);
    // 8.
    assert!(!rxs[8].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[8].returns),
        "smart contract fault: asset withdraw locked by owner"
    );
    // 9.
    assert!(!rxs[9].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[9].returns),
        "smart contract fault: asset deposit locked by owner"
    );
    // 10.
    assert!(rxs[10].success);
    // 11.
    assert!(rxs[11].success);
}

#[test]
fn lock_and_unlock() {
    // Instance the application.
    let mut app = TestApp::default();

    // Perform various asset transactions.
    let txs = create_locking_txs();
    let rxs = app.exec_txs(txs);
    check_locking_rxs(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let alice_info = ACCOUNTS_INFO.get(ALICE_ALIAS).unwrap();
    let bob_info = ACCOUNTS_INFO.get(BOB_ALIAS).unwrap();

    let account = app.account(&alice_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 8);
    assert_eq!(asset.lock, None);

    let account = app.account(&bob_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 2);
    assert_eq!(asset.lock, None);
}
