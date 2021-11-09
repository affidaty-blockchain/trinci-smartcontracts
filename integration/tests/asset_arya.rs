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
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use trinci_core::crypto::{sign::PublicKey, Hash};
use trinci_core::{
    base::serialize::{self},
    crypto::ecdsa::{CurveId, KeyPair as EcdsaKeyPair},
    KeyPair,
};
use trinci_core::{Receipt, Transaction};

const ASSET_ALIAS: &str = "FCK";
const ALICE_ALIAS: &str = "Alice";
const BOB_ALIAS: &str = "Bob";
const DAVE_ALIAS: &str = "Dave";
const CRYPTO_ALIAS: &str = "Crypto";
const ARYA_ALIAS: &str = "Arya";
const DELEGATE_ALIAS: &str = "Delegate";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(ASSET_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(ALICE_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(BOB_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(DAVE_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map.insert(CRYPTO_ALIAS, AccountInfo::new(PUB_KEY5, PVT_KEY5));
        map.insert(ARYA_ALIAS, AccountInfo::new(PUB_KEY6, PVT_KEY6));
        map.insert(DELEGATE_ALIAS, AccountInfo::new(PUB_KEY7, PVT_KEY7));
        map
    };
}
lazy_static! {
    pub static ref ARYA_APP_HASH: Hash = common::app_hash("arya.wasm").unwrap();
    pub static ref CRYPTO_APP_HASH: Hash = common::app_hash("crypto.wasm").unwrap();
    pub static ref ASSET_ARYA_APP_HASH: Hash = common::app_hash("asset_arya.wasm").unwrap();
}

fn asset_init_tx(asset_info: &AccountInfo, arya_info: &AccountInfo) -> Transaction {
    let args = value!({
        "name": ASSET_ALIAS,
        "authorized": Vec::<&str>::new(),
        "arya_id": &arya_info.id,

        "description": "My Cool Coin",
        "url": "https://fck.you",
        "max_units": 100_000,
    });
    common::create_test_tx(
        &asset_info.id,
        &asset_info.pub_key,
        &asset_info.pvt_key,
        *ASSET_ARYA_APP_HASH,
        "init",
        args,
    )
}

fn asset_mint_tx(
    asset_info: &AccountInfo,
    caller_info: &AccountInfo,
    to_info: &AccountInfo,
    units: u64,
) -> Transaction {
    let args = value!({
        "to": to_info.id,
        "units": units,
    });
    common::create_test_tx(
        &asset_info.id,
        &caller_info.pub_key,
        &caller_info.pvt_key,
        *ASSET_ARYA_APP_HASH,
        "mint",
        args,
    )
}

fn asset_burn_tx(asset_info: &AccountInfo, from_info: &AccountInfo, units: u64) -> Transaction {
    let args = value!({
        "from": from_info.id,
        "units": units,
    });
    common::create_test_tx(
        &asset_info.id,
        &asset_info.pub_key,
        &asset_info.pvt_key,
        *ASSET_ARYA_APP_HASH,
        "burn",
        args,
    )
}

fn asset_balance_tx(asset_info: &AccountInfo, from_info: &AccountInfo) -> Transaction {
    common::create_test_tx(
        &asset_info.id,
        &from_info.pub_key,
        &from_info.pvt_key,
        *ASSET_ARYA_APP_HASH,
        "balance",
        value!(null),
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
        *ASSET_ARYA_APP_HASH,
        "transfer",
        args,
    )
}

// // This is signed by the destination account
// fn transfer_delegate_tx(
//     asset_info: &AccountInfo,
//     from_info: &AccountInfo,
//     to_info: &AccountInfo,
//     units: u64,
// ) -> Transaction {
//     let args = value!({
//         "from": from_info.id,
//         "to": to_info.id,
//         "units": units,
//     });
//     common::create_test_tx(
//         &asset_info.id,
//         &to_info.pub_key,
//         &to_info.pvt_key,
//         *ASSET_ARYA_APP_HASH,
//         "transfer",
//         args,
//     )
// }

fn asset_stats_tx(asset_info: &AccountInfo, caller_info: &AccountInfo) -> Transaction {
    common::create_test_tx(
        &asset_info.id,
        &caller_info.pub_key,
        &caller_info.pvt_key,
        *ASSET_ARYA_APP_HASH,
        "stats",
        value!(null),
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

fn crypto_hash_tx(owner_info: &AccountInfo, user_info: &AccountInfo) -> Transaction {
    let args = HashArgs {
        algorithm: HashAlgorithm::Sha256,
        data: &[1, 2, 3],
    };

    common::create_test_tx(
        &owner_info.id,
        &user_info.pub_key,
        &user_info.pvt_key,
        *CRYPTO_APP_HASH,
        "hash",
        args,
    )
}

fn arya_init_tx(arya_info: &AccountInfo, crypto_info: &AccountInfo) -> Transaction {
    let args = value! ({"crypto": crypto_info.id});

    common::create_test_tx(
        &arya_info.id,
        &arya_info.pub_key,
        &arya_info.pvt_key,
        *ARYA_APP_HASH,
        "init",
        args,
    )
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
struct DelegationData<'a> {
    delegate: &'a str,
    delegator: PublicKey,
    network: &'a str,
    target: &'a str,
    expiration: u64,
    capabilities: BTreeMap<&'a str, bool>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
struct Delegation<'a> {
    data: DelegationData<'a>,
    #[serde(with = "serde_bytes")]
    signature: &'a [u8],
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
struct SetDelegationArgs<'a> {
    key: &'a str,
    #[serde(with = "serde_bytes")]
    delegation: &'a [u8],
}

fn arya_set_delegation_tx(
    arya_info: &AccountInfo,
    delegate_info: &AccountInfo,
    delegator_info: &AccountInfo,
    target_account: &AccountInfo,
    method: &str,
) -> Transaction {
    let mut capabilities = BTreeMap::<&str, bool>::new();
    capabilities.insert("*", false);
    capabilities.insert(method, true);

    let public_bytes = hex::decode(delegator_info.pub_key.clone()).unwrap();
    let private_bytes = hex::decode(delegator_info.pvt_key.clone()).unwrap();
    let ecdsa_keypair =
        EcdsaKeyPair::new(CurveId::Secp384R1, &private_bytes, &public_bytes).unwrap();
    let delegator_kp = KeyPair::Ecdsa(ecdsa_keypair);

    let data = DelegationData {
        delegate: &delegate_info.id,
        delegator: delegator_kp.public_key(),
        network: "skynet",
        target: &target_account.id,
        expiration: 123u64,
        capabilities: capabilities.clone(),
    };

    let data_to_sign = serialize::rmp_serialize(&data).unwrap();

    let sign = delegator_kp.sign(&data_to_sign).unwrap();

    let delegation = Delegation {
        data,
        signature: &sign,
    };

    let delegation = serialize::rmp_serialize(&delegation).unwrap();

    let args = SetDelegationArgs {
        key: "",
        delegation: &delegation,
    };

    common::create_test_tx(
        &arya_info.id,
        &delegator_info.pub_key,
        &delegator_info.pvt_key,
        *ARYA_APP_HASH,
        "set_delegation",
        args,
    )
}

fn create_basic_txs() -> Vec<Transaction> {
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let alice_info = ACCOUNTS_INFO.get(ALICE_ALIAS).unwrap();
    let bob_info = ACCOUNTS_INFO.get(BOB_ALIAS).unwrap();
    let dave_info = ACCOUNTS_INFO.get(DAVE_ALIAS).unwrap();
    let delegate_info = ACCOUNTS_INFO.get(DELEGATE_ALIAS).unwrap();
    let crypto_info = ACCOUNTS_INFO.get(CRYPTO_ALIAS).unwrap();
    let arya_info = ACCOUNTS_INFO.get(ARYA_ALIAS).unwrap();

    vec![
        // 0. Crypto Hash to initializate crypto contract in account
        crypto_hash_tx(crypto_info, crypto_info),
        // 1. Arya init
        arya_init_tx(arya_info, crypto_info),
        // 2. Asset-arya init
        asset_init_tx(asset_info, arya_info),
        // 3. Mint asset in Alice's account from account not yet delegate. This shall fail
        asset_mint_tx(asset_info, delegate_info, alice_info, 10),
        // 4. Add delegation from asset creator to mint Delegate account
        arya_set_delegation_tx(arya_info, delegate_info, asset_info, asset_info, "mint"),
        // 5. Mint asset in Alice's account from delegate account.
        asset_mint_tx(asset_info, delegate_info, alice_info, 10),
        // 6. Mint asset in Bob's account.
        asset_mint_tx(asset_info, asset_info, bob_info, 3),
        // 7. Get Alice's balance.
        asset_balance_tx(asset_info, alice_info),
        // 8. Transfer from Alice to Bob account.
        asset_transfer_tx(asset_info, alice_info, bob_info, 1),
        // 9. Get Alice's balance.
        asset_balance_tx(asset_info, alice_info),
        // 10. Get Bob's balance.
        asset_balance_tx(asset_info, bob_info),
        // 11. Transfer from Bob to Dave account.
        asset_transfer_tx(asset_info, bob_info, dave_info, 1),
        // 12. Get Dave's balance.
        asset_balance_tx(asset_info, dave_info),
        // 13. Burn from Alice's account.
        asset_burn_tx(asset_info, alice_info, 1),
        // 14. Alice asks for asset's stats.
        asset_stats_tx(asset_info, alice_info),
        // 15. Try to mint too much. Shall fail.
        asset_mint_tx(asset_info, asset_info, bob_info, 200000),
        // 16. Try to burn too much. Shall fail.
        asset_burn_tx(asset_info, alice_info, 200000),
        // 17. Try to burn too much. Shall fail.
        asset_burn_tx(asset_info, alice_info, 9),
    ]
}

fn check_basic_rxs(rxs: Vec<Receipt>) {
    // 0. Crypto Hash to initializate crypto contract in account
    assert!(rxs[0].success);
    // 1. Arya init
    assert!(rxs[1].success);
    // 2. Asset-arya init
    assert!(rxs[2].success);
    // 3. Mint asset in Alice's account from account not yet delegate. This shall fail
    assert!(!rxs[3].success);
    assert_eq!(
        "smart contract fault: not authorized",
        String::from_utf8_lossy(&rxs[3].returns)
    );
    // 4. Add delegation from asset creator to mint Delegate account
    assert!(rxs[4].success);
    // 5. Mint asset in Alice's account from delegate account.
    assert!(rxs[5].success);
    // 6. Mint asset in Bob's account.
    assert!(rxs[6].success);
    // 7. Get Alice's balance.
    let value: u64 = serialize::rmp_deserialize(&rxs[7].returns).unwrap();
    assert_eq!(value, 10);
    // 8. Transfer from Alice to Bob account.
    assert!(rxs[8].success);
    // 9. Get Alice's balance.
    let value: u64 = serialize::rmp_deserialize(&rxs[9].returns).unwrap();
    assert_eq!(value, 9);
    // 10. Get Bob's balance.
    let value: u64 = serialize::rmp_deserialize(&rxs[10].returns).unwrap();
    assert_eq!(value, 4);
    // 11. Transfer from Bob to Dave account.
    assert!(rxs[11].success);
    // 12. Get Dave's balance.
    assert!(rxs[12].success);
    let value: u64 = serialize::rmp_deserialize(&rxs[12].returns).unwrap();
    assert_eq!(value, 1);
    // 13. Burn from Alice's account.
    assert!(rxs[13].success);
    // 14. Alice asks for asset's stats.
    assert!(rxs[14].success);
    let stats: SerdeValue = serialize::rmp_deserialize(&rxs[14].returns).unwrap();
    let name = stats.get(&value!("name")).unwrap();
    assert_eq!(name, "FCK");
    // 15. Try to mint too much. Shall fail.
    assert!(!rxs[15].success);
    assert_eq!(
        "smart contract fault: minting overcome the max_units value",
        String::from_utf8_lossy(&rxs[15].returns)
    );
    // 16. Try to burn too much. Shall fail.
    assert!(!rxs[16].success);
    assert_eq!(
        "smart contract fault: insufficient funds",
        String::from_utf8_lossy(&rxs[16].returns)
    );
    // 17. Try to burn too much. Shall fail.
    assert!(!rxs[17].success);
    assert_eq!(
        "smart contract fault: insufficient funds",
        String::from_utf8_lossy(&rxs[17].returns)
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
    assert_eq!(asset.units, 3);
    assert_eq!(asset.lock, None);

    let account = app.account(&dave_info.id).unwrap();
    let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(asset.units, 1);
    assert_eq!(asset.lock, None);
}

fn asset_lock_tx(
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
        *ASSET_ARYA_APP_HASH,
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
        asset_init_tx(asset_info, asset_info),
        // 1. Mint on Alice account
        asset_mint_tx(asset_info, asset_info, alice_info, 10),
        // 2. Creator locks Alice asset.
        asset_lock_tx(asset_info, asset_info, alice_info, LockType::Full),
        // 3. Alice tries to transfer to Bob while locked, shall fail.
        asset_transfer_tx(asset_info, alice_info, bob_info, 3),
        // 4. Alice tries to unlock, shall fail.
        asset_lock_tx(asset_info, alice_info, alice_info, LockType::None),
        // 5. Creator unlocks Alice asset.
        asset_lock_tx(asset_info, asset_info, alice_info, LockType::None),
        // 6. Alice transfer some funds to Bob.
        asset_transfer_tx(asset_info, alice_info, bob_info, 3),
        // 7. Alice locks the asset.
        asset_lock_tx(asset_info, alice_info, alice_info, LockType::Full),
        // 8. Alice tries to transfer to Bob while locked, shall fail.
        asset_transfer_tx(asset_info, alice_info, bob_info, 3),
        // 9. Bob tries to transfer to Alice while she's locked, shall fail as well.
        asset_transfer_tx(asset_info, bob_info, alice_info, 3),
        // 10. Alice unlocks the asset.
        asset_lock_tx(asset_info, alice_info, alice_info, LockType::None),
        // 11. Now Bob can transfer to Alice.
        asset_transfer_tx(asset_info, bob_info, alice_info, 1),
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
