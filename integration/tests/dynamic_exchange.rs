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

//! Dynamic Exchange integration tests

use integration::{
    common::{
        self, AccountInfo, Asset, ASSET_APP_HASH, PUB_KEY1, PUB_KEY2, PUB_KEY3, PUB_KEY4, PUB_KEY5,
        PUB_KEY6, PUB_KEY7, PUB_KEY8, PVT_KEY1, PVT_KEY2, PVT_KEY3, PVT_KEY4, PVT_KEY5, PVT_KEY6,
        PVT_KEY7, PVT_KEY8,
    },
    TestApp,
};
use lazy_static::lazy_static;
use serde_value::Value;
use std::collections::HashMap;
use trinci_core::{
    base::serialize::{self, rmp_deserialize},
    crypto::Hash,
    Receipt, Transaction,
};
use trinci_sdk::{value, PackedValue};

lazy_static! {
    pub static ref DYNAMIC_EXCHANGE_APP_HASH: Hash =
        common::app_hash("dynamic_exchange.wasm").unwrap();
}

use serde::{Deserialize, Serialize};

const EXCHANGE_ALIAS: &str = "Exchange";
const SELLER_ALIAS: &str = "Seller";
const GUARANTOR_ALIAS: &str = "Guarantor";
const BUYER_ALIAS: &str = "Buyer";
const SRC_ASSET_ALIAS: &str = "Asset_src";
const DST_ASSET_1_ALIAS: &str = "Asset_dst_1";
const DST_ASSET_2_ALIAS: &str = "Asset_dst_2";
const PENALTY_ASSET_ALIAS: &str = "PenaltyAsset";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(EXCHANGE_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(SELLER_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(GUARANTOR_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(BUYER_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map.insert(SRC_ASSET_ALIAS, AccountInfo::new(PUB_KEY5, PVT_KEY5));
        map.insert(DST_ASSET_1_ALIAS, AccountInfo::new(PUB_KEY6, PVT_KEY6));
        map.insert(DST_ASSET_2_ALIAS, AccountInfo::new(PUB_KEY7, PVT_KEY7));
        map.insert(PENALTY_ASSET_ALIAS, AccountInfo::new(PUB_KEY8, PVT_KEY8));
        map
    };
}

/// NOTE: This struct must be the same as in rust/dynamic_exchange
/// Dynamic Exchange Apply arguments
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct ApplyArgs<'a> {
    pub asset: &'a str,
    pub amount: u64,
}

/// Struct to delegate not-owner account to perform payment
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Delegation<'a> {
    /// The delegate account (will be the transfer caller)
    pub delegate: &'a str,
    /// Amount of asset to allow the transfer
    pub units: u64,
    /// Destination account for the transfer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<&'a str>,
}

fn create_config_value(
    src_asset: &AccountInfo,
    guarantor: &AccountInfo,
    guarantor_fee: u64,
    seller: &AccountInfo,
    dst_asset_1: &AccountInfo,
    dst_asset_2: &AccountInfo,
    penalty_asset: &AccountInfo,
    penalty_amount: u64,
) -> Value {
    value!({
        "asset": src_asset.id,
        "guarantor": guarantor.id,
        "seller": seller.id,
        "guarantor_fee": guarantor_fee,     // 35/1000*100 = 3.5%
        "penalty_fee": penalty_amount,      // 45/1000*100 = 4.5%
        "penalty_asset": penalty_asset.id,
        "assets": {
            dst_asset_1.id.clone(): 250,    // 250/100 => dst_asset1 = 2.5 * src_asset
            dst_asset_2.id.clone(): 200,    // 200/100 => dst_asset1 = 2 * src_asset
        },
    })
}

fn dynamic_exchange_init_tx(
    exchange: &AccountInfo,
    src_asset: &AccountInfo,
    guarantor: &AccountInfo,
    guarantor_fee: u64,
    seller: &AccountInfo,
    dst_asset_1: &AccountInfo,
    dst_asset_2: &AccountInfo,
    penalty_asset: &AccountInfo,
    penalty_amount: u64,
) -> Transaction {
    // Initialization data
    let args = create_config_value(
        src_asset,
        guarantor,
        guarantor_fee,
        seller,
        dst_asset_1,
        dst_asset_2,
        penalty_asset,
        penalty_amount,
    );

    common::create_test_tx(
        &exchange.id,
        &exchange.pub_key,
        &exchange.pvt_key,
        *DYNAMIC_EXCHANGE_APP_HASH,
        "init",
        args,
    )
}

fn dynamic_exchange_get_info_tx(exchange: &AccountInfo, buyer: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &exchange.id,
        &buyer.pub_key,
        &buyer.pvt_key,
        *DYNAMIC_EXCHANGE_APP_HASH,
        "get_info",
        args,
    )
}

fn dynamic_exchange_abort(exchange: &AccountInfo, caller: &AccountInfo) -> Transaction {
    let args = PackedValue::default();

    common::create_test_tx(
        &exchange.id,
        &caller.pub_key,
        &caller.pvt_key,
        *DYNAMIC_EXCHANGE_APP_HASH,
        "abort",
        args.0,
    )
}

fn dynamic_exchange_apply_tx(
    exchange: &AccountInfo,
    buyer: &AccountInfo,
    asset_id: &str,
    amount: u64,
) -> Transaction {
    let args = ApplyArgs {
        asset: asset_id,
        amount: amount,
    };

    common::create_test_tx(
        &exchange.id,
        &buyer.pub_key,
        &buyer.pvt_key,
        *DYNAMIC_EXCHANGE_APP_HASH,
        "apply",
        args,
    )
}

pub fn asset_init_tx(asset_info: &AccountInfo, asset_name: &str) -> Transaction {
    let args = value!({
        "name": asset_name,
        "description": "My Cool Coin",
        "url": "https://fck.you",
        "max_units": 100_000,
        "authorized": [],
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

pub fn asset_mint_tx(asset_info: &AccountInfo, to_info: &AccountInfo, units: u64) -> Transaction {
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

pub fn asset_transfer_tx(
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

fn asset_add_delegation_tx(
    asset_info: &AccountInfo,
    delegator_info: &AccountInfo,
    delegate_info: &AccountInfo,
    to_info: &AccountInfo,
    units: u64,
) -> Transaction {
    let args = Delegation {
        delegate: &delegate_info.id,
        units,
        to: Some(&to_info.id),
    };

    common::create_test_tx(
        &asset_info.id,
        &delegator_info.pub_key,
        &delegator_info.pvt_key,
        *ASSET_APP_HASH,
        "add_delegation",
        args,
    )
}

fn create_dynamic_exchange_info() -> Value {
    let src_asset = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let seller = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let guarantor = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let dst_asset_1 = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let dst_asset_2 = ACCOUNTS_INFO.get(DST_ASSET_2_ALIAS).unwrap();

    let config = create_config_value(
        src_asset,
        guarantor,
        35,
        seller,
        dst_asset_1,
        dst_asset_2,
        src_asset,
        45,
    );

    value!({
        "config": config,
        "amount": 100,
        "status": "open"
    })
}

fn create_txs() -> Vec<Transaction> {
    let src_asset_info = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let seller_info = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let buyer_info = ACCOUNTS_INFO.get(BUYER_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let dst_asset1_info = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let dst_asset2_info = ACCOUNTS_INFO.get(DST_ASSET_2_ALIAS).unwrap();

    vec![
        // 0. Initialize src asset
        asset_init_tx(src_asset_info, SRC_ASSET_ALIAS),
        // 1. Mint some units in seller account.
        asset_mint_tx(src_asset_info, seller_info, 1000),
        // 2. Initialize exchange account.
        dynamic_exchange_init_tx(
            exchange_info,
            src_asset_info,
            guarantor_info,
            35,
            seller_info,
            dst_asset1_info,
            dst_asset2_info,
            src_asset_info,
            45,
        ),
        // 3. Transfer funds from seller to exchange account.
        asset_transfer_tx(src_asset_info, seller_info, exchange_info, 100),
        // 4. Initialize dst1 asset
        asset_init_tx(dst_asset1_info, DST_ASSET_1_ALIAS),
        // 5. Initialize dst2 asset
        asset_init_tx(dst_asset2_info, DST_ASSET_2_ALIAS),
        // 6. Mint some units dst1_asset in buyer account.
        asset_mint_tx(dst_asset1_info, buyer_info, 100),
        // 7. Mint some units dst2_asset in buyer account.
        asset_mint_tx(dst_asset2_info, buyer_info, 100),
        // 8. Get exchange configuration.
        dynamic_exchange_get_info_tx(exchange_info, buyer_info),
        // 9. Apply to exchange with not accepted asset. This shall fail
        dynamic_exchange_apply_tx(exchange_info, buyer_info, "not_existing_asset", 3),
        // 10. Add delegation to dst_asset1 for 20 units to dynamic exchange account
        asset_add_delegation_tx(
            dst_asset1_info,
            buyer_info,
            exchange_info,
            exchange_info,
            20,
        ),
        // 11. Apply with dst asset 1.
        dynamic_exchange_apply_tx(exchange_info, buyer_info, &dst_asset1_info.id, 20),
        // 12. Apply with too much dst asset 2. This shall fail
        dynamic_exchange_apply_tx(exchange_info, buyer_info, &dst_asset2_info.id, 1000),
        // 13. Add delegation to dst_asset2 for 20 units to dynamic exchange account
        asset_add_delegation_tx(
            dst_asset2_info,
            buyer_info,
            exchange_info,
            exchange_info,
            25,
        ),
        // 14. Apply with asset 2 to exhaust the exhange.
        dynamic_exchange_apply_tx(exchange_info, buyer_info, &dst_asset2_info.id, 25),
        // 15. Apply with asset 1 again, this is expected to fail.
        dynamic_exchange_apply_tx(exchange_info, buyer_info, &dst_asset1_info.id, 1),
    ]
}

fn check_rxs(rxs: Vec<Receipt>) {
    // 0. Initialize src asset
    assert!(rxs[0].success);
    // 1. Mint some units in seller account.
    assert!(rxs[1].success);
    // 2. Initialize exchange account.
    assert!(rxs[2].success);
    // 3. Transfer funds from seller to exchange account.
    assert!(rxs[3].success);
    // 4. Initialize dst1 asset
    assert!(rxs[4].success);
    // 5. Initialize dst2 asset
    assert!(rxs[5].success);
    // 6. Mint some units dst1_asset in buyer account.
    assert!(rxs[6].success);
    // 7. Mint some units dst2_asset in buyer account.
    assert!(rxs[7].success);
    // 8. Get exchange configuration.
    assert!(rxs[8].success);
    // 9. Apply to exchange with not accepted asset
    assert!(!rxs[9].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[9].returns),
        "smart contract fault: not exchangeable asset"
    );
    // 10. Add delegation to dst_asset1 for 40 units to dynamic exchange account
    assert!(rxs[10].success);
    // 11. Apply with dst asset 1.
    assert!(rxs[11].success);
    // 12. Apply with too much dst asset 2. This shall fail
    assert!(!rxs[12].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[12].returns),
        "smart contract fault: insufficient funds"
    );
    // 13. Add delegation to dst_asset2 for 20 units to dynamic exchange account
    assert!(rxs[13].success);
    // 14. Apply with asset 2 to exhaust the exhange.
    assert!(rxs[14].success);
    // 15. Apply with asset 1 again, this is expected to fail.
    assert!(!rxs[15].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[15].returns),
        "smart contract fault: exchange not open"
    );
}

#[test]
fn dynamic_exchange_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);

    // Blockchain check.

    let src_asset_info = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let seller_info = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let buyer_info = ACCOUNTS_INFO.get(BUYER_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let dst_asset1_info = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let dst_asset2_info = ACCOUNTS_INFO.get(DST_ASSET_2_ALIAS).unwrap();

    let exchange_account = app.account(&exchange_info.id).unwrap();
    let exchange_src_asset: Asset =
        serialize::rmp_deserialize(&exchange_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(exchange_src_asset.units, 0);

    let seller_account = app.account(&seller_info.id).unwrap();
    let seller_src_asset: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(seller_src_asset.units, 900);

    let seller_asset1: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(seller_asset1.units, 19);

    let seller_asset2: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&dst_asset2_info.id)).unwrap();
    assert_eq!(seller_asset2.units, 24);

    let buyer_account = app.account(&buyer_info.id).unwrap();
    let buyer_src_asset: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(buyer_src_asset.units, 100);

    let buyer_asset1: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(buyer_asset1.units, 80);

    let buyer_asset2: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&dst_asset2_info.id)).unwrap();
    assert_eq!(buyer_asset2.units, 75);

    let guarantor_account = app.account(&guarantor_info.id).unwrap();

    let guarantor_asset1: Asset =
        serialize::rmp_deserialize(&guarantor_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(guarantor_asset1.units, 1);

    let guarantor_asset2: Asset =
        serialize::rmp_deserialize(&guarantor_account.load_asset(&dst_asset2_info.id)).unwrap();
    assert_eq!(guarantor_asset2.units, 1);
}

fn create_abort_txs() -> Vec<Transaction> {
    let src_asset_info = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let seller_info = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let buyer_info = ACCOUNTS_INFO.get(BUYER_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let dst_asset1_info = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let dst_asset2_info = ACCOUNTS_INFO.get(DST_ASSET_2_ALIAS).unwrap();

    vec![
        // 0. Initialize src asset
        asset_init_tx(src_asset_info, SRC_ASSET_ALIAS),
        // 1. Mint some units in seller account.
        asset_mint_tx(src_asset_info, seller_info, 1000),
        // 2. Initialize exchange account.
        dynamic_exchange_init_tx(
            exchange_info,
            src_asset_info,
            guarantor_info,
            35,
            seller_info,
            dst_asset1_info,
            dst_asset2_info,
            src_asset_info,
            45,
        ),
        // 3. Transfer funds from seller to exchange account.
        asset_transfer_tx(src_asset_info, seller_info, exchange_info, 100),
        // 4. Initialize dst1 asset
        asset_init_tx(dst_asset1_info, DST_ASSET_1_ALIAS),
        // 5. Initialize dst2 asset
        asset_init_tx(dst_asset2_info, DST_ASSET_2_ALIAS),
        // 6. Mint some units dst1_asset in buyer account.
        asset_mint_tx(dst_asset1_info, buyer_info, 100),
        // 7. Mint some units dst2_asset in buyer account.
        asset_mint_tx(dst_asset2_info, buyer_info, 100),
        // 8. Get exchange configuration.
        dynamic_exchange_get_info_tx(exchange_info, buyer_info),
        // 9. Exchange Abort from unauthorized user. This shall fail
        dynamic_exchange_abort(exchange_info, buyer_info),
        // 10. Add delegation to dst_asset1 for 20 units to dynamic exchange account
        asset_add_delegation_tx(
            dst_asset1_info,
            buyer_info,
            exchange_info,
            exchange_info,
            20,
        ),
        // 11. Apply with dst asset 1.
        dynamic_exchange_apply_tx(exchange_info, buyer_info, &dst_asset1_info.id, 20),
        // 12. Exchange Abort from the guarantor
        dynamic_exchange_abort(exchange_info, guarantor_info),
        // 13. Exchange Abort from the seller. This shall fail
        dynamic_exchange_abort(exchange_info, seller_info),
    ]
}

fn check_abort_rxs(rxs: Vec<Receipt>) {
    // 0. Initialize src asset
    assert!(rxs[0].success);
    // 1. Mint some units in seller account.
    assert!(rxs[1].success);
    // 2. Initialize exchange account.
    assert!(rxs[2].success);
    // 3. Transfer funds from seller to exchange account.
    assert!(rxs[3].success);
    // 4. Initialize dst1 asset
    assert!(rxs[4].success);
    // 5. Initialize dst2 asset
    assert!(rxs[5].success);
    // 6. Mint some units dst1_asset in buyer account.
    assert!(rxs[6].success);
    // 7. Mint some units dst2_asset in buyer account.
    assert!(rxs[7].success);
    // 8. Get exchange configuration.
    assert!(rxs[8].success);
    let result: Value = rmp_deserialize(&rxs[8].returns).unwrap();
    let expected: Value = create_dynamic_exchange_info();
    assert_eq!(
        result.get(&value!("amount")).unwrap().as_u64().unwrap(),
        expected.get(&value!("amount")).unwrap().as_u64().unwrap()
    );
    assert_eq!(
        result.get(&value!("status")).unwrap().as_str().unwrap(),
        expected.get(&value!("status")).unwrap().as_str().unwrap()
    );
    // 9. Exchange Abort from unauthorized user. This shall fail
    assert!(!rxs[9].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[9].returns),
        "smart contract fault: not authorized"
    );
    // 10. Add delegation to dst_asset1 for 20 units to dynamic exchange account
    assert!(rxs[10].success);
    // 11. Apply with dst asset 1.
    assert!(rxs[11].success);
    // 12. Exchange Abort from the guarantor
    assert!(rxs[12].success);
    // 13. Exchange Abort from the seller. This shall fail
    assert!(!rxs[13].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[13].returns),
        "smart contract fault: exchange not open"
    );
}

#[test]
fn dynamic_exchange_abort_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_abort_txs();
    let rxs = app.exec_txs(txs);
    check_abort_rxs(rxs);

    // Blockchain check.

    let src_asset_info = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let seller_info = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let buyer_info = ACCOUNTS_INFO.get(BUYER_ALIAS).unwrap();
    let dst_asset1_info = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();

    let exchange_account = app.account(&exchange_info.id).unwrap();
    let exchange_src_asset: Asset =
        serialize::rmp_deserialize(&exchange_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(exchange_src_asset.units, 0);

    let guarantor_account = app.account(&guarantor_info.id).unwrap();
    let guarantor_src_asset: Asset =
        serialize::rmp_deserialize(&guarantor_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(guarantor_src_asset.units, 2);
    let guarantor_dst_asset1: Asset =
        serialize::rmp_deserialize(&guarantor_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(guarantor_dst_asset1.units, 1);

    let seller_account = app.account(&seller_info.id).unwrap();
    let seller_src_asset: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(seller_src_asset.units, 948);
    let seller_dst_asset1: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(seller_dst_asset1.units, 19);

    let buyer_account = app.account(&buyer_info.id).unwrap();
    let buyer_src_asset: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(buyer_src_asset.units, 50);
    let buyer_dst_asset1: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(buyer_dst_asset1.units, 80);
}

fn create_abort_with_different_penalty_asset_txs() -> Vec<Transaction> {
    let src_asset_info = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let seller_info = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let buyer_info = ACCOUNTS_INFO.get(BUYER_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let dst_asset1_info = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let dst_asset2_info = ACCOUNTS_INFO.get(DST_ASSET_2_ALIAS).unwrap();
    let penalty_asset_info = ACCOUNTS_INFO.get(PENALTY_ASSET_ALIAS).unwrap();

    vec![
        // 0. Initialize src asset
        asset_init_tx(src_asset_info, SRC_ASSET_ALIAS),
        // 1. Mint some units in seller account.
        asset_mint_tx(src_asset_info, seller_info, 1000),
        // 2. Initialize penalty asset
        asset_init_tx(penalty_asset_info, PENALTY_ASSET_ALIAS),
        // 3. Initialize exchange account.
        dynamic_exchange_init_tx(
            exchange_info,
            src_asset_info,
            guarantor_info,
            35,
            seller_info,
            dst_asset1_info,
            dst_asset2_info,
            penalty_asset_info,
            45,
        ),
        // 4. Transfer funds from seller to exchange account.
        asset_transfer_tx(src_asset_info, seller_info, exchange_info, 100),
        // 5. Initialize dst1 asset
        asset_init_tx(dst_asset1_info, DST_ASSET_1_ALIAS),
        // 6. Initialize dst2 asset
        asset_init_tx(dst_asset2_info, DST_ASSET_2_ALIAS),
        // 7. Mint some units penalty_asset in seller account.
        asset_mint_tx(penalty_asset_info, seller_info, 50),
        // 8. Mint some units dst1_asset in buyer account.
        asset_mint_tx(dst_asset1_info, buyer_info, 100),
        // 9. Mint some units dst2_asset in buyer account.
        asset_mint_tx(dst_asset2_info, buyer_info, 100),
        // 10. Get exchange configuration.
        dynamic_exchange_get_info_tx(exchange_info, buyer_info),
        // 11. Exchange Abort from unauthorized user. This shall fail
        dynamic_exchange_abort(exchange_info, buyer_info),
        // 12. Add delegation to dst_asset1 for 20 units to dynamic exchange account
        asset_add_delegation_tx(
            dst_asset1_info,
            buyer_info,
            exchange_info,
            exchange_info,
            20,
        ),
        // 13. Apply with dst asset 1.
        dynamic_exchange_apply_tx(exchange_info, buyer_info, &dst_asset1_info.id, 20),
        // 14. Exchange Abort from the seller. This shall fail because there is not delegation for seller and penalty asset
        dynamic_exchange_abort(exchange_info, seller_info),
        // 15. Add delegation to dst_asset1 for 20 units to dynamic exchange account
        asset_add_delegation_tx(
            penalty_asset_info,
            seller_info,
            exchange_info,
            guarantor_info,
            45,
        ),
        // 16. Exchange Abort from the seller.
        dynamic_exchange_abort(exchange_info, guarantor_info),
        // 17. Exchange Abort from the seller. This shall fail because the exchange is not open.
        dynamic_exchange_abort(exchange_info, seller_info),
    ]
}

fn check_abort_with_different_penalty_asset_rxs(rxs: Vec<Receipt>) {
    // 0. Initialize src asset
    assert!(rxs[0].success);
    // 1. Mint some units in seller account.
    assert!(rxs[1].success);
    // 2. Initialize penalty asset
    assert!(rxs[2].success);
    // 3. Initialize exchange account.
    assert!(rxs[3].success);
    // 4. Transfer funds from seller to exchange account.
    assert!(rxs[4].success);
    // 5. Initialize dst1 asset
    assert!(rxs[5].success);
    // 6. Initialize dst2 asset
    assert!(rxs[6].success);
    // 7. Mint some units penalty_asset in seller account.
    assert!(rxs[7].success);
    // 8. Mint some units dst1_asset in buyer account.
    assert!(rxs[8].success);
    // 9. Mint some units dst2_asset in buyer account.
    assert!(rxs[9].success);
    // 10. Get exchange configuration.
    assert!(rxs[10].success);
    let result: Value = rmp_deserialize(&rxs[10].returns).unwrap();
    let expected: Value = create_dynamic_exchange_info();
    assert_eq!(
        result.get(&value!("amount")).unwrap().as_u64().unwrap(),
        expected.get(&value!("amount")).unwrap().as_u64().unwrap()
    );
    assert_eq!(
        result.get(&value!("status")).unwrap().as_str().unwrap(),
        expected.get(&value!("status")).unwrap().as_str().unwrap()
    );
    // 11. Exchange Abort from unauthorized user. This shall fail
    assert!(!rxs[11].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[11].returns),
        "smart contract fault: not authorized"
    );
    // 12. Add delegation to dst_asset1 for 20 units to dynamic exchange account
    assert!(rxs[12].success);
    // 13. Apply with dst asset 1.
    assert!(rxs[13].success);
    // 14. Exchange Abort from the seller. This shall fail because there is not delegation for seller and penalty asset
    assert!(!rxs[14].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[14].returns),
        "smart contract fault: failed transfer penalty_fee to guarantor"
    );
    // 15. Add delegation to dst_asset1 for 20 units to dynamic exchange account
    assert!(rxs[15].success);
    // 16. Exchange Abort from the seller.
    assert!(rxs[16].success);
    // 17. Exchange Abort from the seller. This shall fail because the exchange is not open.
    assert!(!rxs[17].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[17].returns),
        "smart contract fault: exchange not open"
    );
}

#[test]
fn dynamic_exchange_abort_with_different_penalty_asset_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_abort_with_different_penalty_asset_txs();
    let rxs = app.exec_txs(txs);
    check_abort_with_different_penalty_asset_rxs(rxs);

    // Blockchain check.

    let src_asset_info = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let seller_info = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let buyer_info = ACCOUNTS_INFO.get(BUYER_ALIAS).unwrap();
    let dst_asset1_info = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let penalty_asset_info = ACCOUNTS_INFO.get(PENALTY_ASSET_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();

    let exchange_account = app.account(&exchange_info.id).unwrap();
    let exchange_src_asset: Asset =
        serialize::rmp_deserialize(&exchange_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(exchange_src_asset.units, 0);

    let guarantor_account = app.account(&guarantor_info.id).unwrap();
    let guarantor_dst_asset1: Asset =
        serialize::rmp_deserialize(&guarantor_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(guarantor_dst_asset1.units, 1);
    let guarantor_penalty_asset: Asset =
        serialize::rmp_deserialize(&guarantor_account.load_asset(&penalty_asset_info.id)).unwrap();
    assert_eq!(guarantor_penalty_asset.units, 45);

    let seller_account = app.account(&seller_info.id).unwrap();
    let seller_src_asset: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(seller_src_asset.units, 950);
    let seller_dst_asset1: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(seller_dst_asset1.units, 19);
    let seller_penalty_asset: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&penalty_asset_info.id)).unwrap();
    assert_eq!(seller_penalty_asset.units, 5);

    let buyer_account = app.account(&buyer_info.id).unwrap();
    let buyer_src_asset: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(buyer_src_asset.units, 50);
    let buyer_dst_asset1: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(buyer_dst_asset1.units, 80);
}

fn create_penalty_asset_zero_txs() -> Vec<Transaction> {
    let src_asset_info = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let penalty_asset_info = ACCOUNTS_INFO.get(PENALTY_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let seller_info = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let buyer_info = ACCOUNTS_INFO.get(BUYER_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let dst_asset1_info = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let dst_asset2_info = ACCOUNTS_INFO.get(DST_ASSET_2_ALIAS).unwrap();

    vec![
        // 0. Initialize src asset
        asset_init_tx(src_asset_info, SRC_ASSET_ALIAS),
        // 1. Mint some units in seller account.
        asset_mint_tx(src_asset_info, seller_info, 1000),
        // 2. Initialize penalty asset
        asset_init_tx(penalty_asset_info, PENALTY_ASSET_ALIAS),
        // 3. Initialize exchange account.
        dynamic_exchange_init_tx(
            exchange_info,
            src_asset_info,
            guarantor_info,
            0,
            seller_info,
            dst_asset1_info,
            dst_asset2_info,
            penalty_asset_info,
            0,
        ),
        // 4. Transfer funds from seller to exchange account.
        asset_transfer_tx(src_asset_info, seller_info, exchange_info, 100),
        // 5. Initialize dst1 asset
        asset_init_tx(dst_asset1_info, DST_ASSET_1_ALIAS),
        // 6. Initialize dst2 asset
        asset_init_tx(dst_asset2_info, DST_ASSET_2_ALIAS),
        // 7. Mint some units penalty_asset in seller account.
        asset_mint_tx(penalty_asset_info, seller_info, 50),
        // 8. Mint some units dst1_asset in buyer account.
        asset_mint_tx(dst_asset1_info, buyer_info, 100),
        // 9. Mint some units dst2_asset in buyer account.
        asset_mint_tx(dst_asset2_info, buyer_info, 100),
        // 10. Get exchange configuration.
        dynamic_exchange_get_info_tx(exchange_info, buyer_info),
        // 11. Exchange Abort from unauthorized user. This shall fail
        dynamic_exchange_abort(exchange_info, buyer_info),
        // 12. Add delegation to dst_asset1 for 20 units to dynamic exchange account
        asset_add_delegation_tx(
            dst_asset1_info,
            buyer_info,
            exchange_info,
            exchange_info,
            20,
        ),
        // 13. Apply with dst asset 1.
        dynamic_exchange_apply_tx(exchange_info, buyer_info, &dst_asset1_info.id, 20),
        // 14. Exchange Abort from the seller.
        dynamic_exchange_abort(exchange_info, seller_info),
        // 15. Exchange Abort from the seller. This shall fail because the exchange is not open.
        dynamic_exchange_abort(exchange_info, seller_info),
    ]
}

fn check_penalty_asset_zero_rxs(rxs: Vec<Receipt>) {
    let penalty_asset_info = ACCOUNTS_INFO.get(PENALTY_ASSET_ALIAS).unwrap();

    // 0. Initialize src asset
    assert!(rxs[0].success);
    // 1. Mint some units in seller account.
    assert!(rxs[1].success);
    // 2. Initialize penalty asset
    assert!(rxs[2].success);
    // 3. Initialize exchange account.
    assert!(rxs[3].success);
    // 4. Transfer funds from seller to exchange account.
    assert!(rxs[4].success);
    // 5. Initialize dst1 asset
    assert!(rxs[5].success);
    // 6. Initialize dst2 asset
    assert!(rxs[6].success);
    // 7. Mint some units penalty_asset in seller account.
    assert!(rxs[7].success);
    // 8. Mint some units dst1_asset in buyer account.
    assert!(rxs[8].success);
    // 9. Mint some units dst2_asset in buyer account.
    assert!(rxs[9].success);
    // 10. Get exchange configuration.
    assert!(rxs[10].success);
    let result: Value = rmp_deserialize(&rxs[10].returns).unwrap();
    let expected: Value = create_dynamic_exchange_info();
    assert_eq!(
        result.get(&value!("amount")).unwrap().as_u64().unwrap(),
        expected.get(&value!("amount")).unwrap().as_u64().unwrap()
    );
    assert_eq!(
        result.get(&value!("status")).unwrap().as_str().unwrap(),
        expected.get(&value!("status")).unwrap().as_str().unwrap()
    );
    assert_eq!(
        result
            .get(&value!("config"))
            .unwrap()
            .as_map()
            .unwrap()
            .get(&value!("penalty_asset"))
            .unwrap()
            .as_str()
            .unwrap(),
        penalty_asset_info.id
    );
    assert_eq!(
        result
            .get(&value!("config"))
            .unwrap()
            .as_map()
            .unwrap()
            .get(&value!("penalty_fee"))
            .unwrap()
            .as_u64()
            .unwrap(),
        0
    );
    // 11. Exchange Abort from unauthorized user. This shall fail
    assert!(!rxs[11].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[11].returns),
        "smart contract fault: not authorized"
    );
    // 12. Add delegation to dst_asset1 for 20 units to dynamic exchange account
    assert!(rxs[12].success);
    // 13. Apply with dst asset 1.
    assert!(rxs[13].success);
    // 14. Exchange Abort from the seller. This shall fail because there is not delegation for seller and penalty asset
    assert!(rxs[14].success);
    // 15. Exchange Abort from the seller. This shall fail because the exchange is not open.
    assert!(!rxs[15].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[15].returns),
        "smart contract fault: exchange not open"
    );
}

#[test]
fn dynamic_exchange_with_penalty_asset_zero_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_penalty_asset_zero_txs();
    let rxs = app.exec_txs(txs);
    check_penalty_asset_zero_rxs(rxs);

    // Blockchain check.

    let src_asset_info = ACCOUNTS_INFO.get(SRC_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let guarantor_info = ACCOUNTS_INFO.get(GUARANTOR_ALIAS).unwrap();
    let seller_info = ACCOUNTS_INFO.get(SELLER_ALIAS).unwrap();
    let buyer_info = ACCOUNTS_INFO.get(BUYER_ALIAS).unwrap();
    let dst_asset1_info = ACCOUNTS_INFO.get(DST_ASSET_1_ALIAS).unwrap();
    let dst_asset2_info = ACCOUNTS_INFO.get(DST_ASSET_2_ALIAS).unwrap();

    let exchange_account = app.account(&exchange_info.id).unwrap();
    let exchange_src_asset: Asset =
        serialize::rmp_deserialize(&exchange_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(exchange_src_asset.units, 0);

    let seller_account = app.account(&seller_info.id).unwrap();
    let seller_src_asset: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(seller_src_asset.units, 950);

    let seller_asset1: Asset =
        serialize::rmp_deserialize(&seller_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(seller_asset1.units, 20);

    let buyer_account = app.account(&buyer_info.id).unwrap();
    let buyer_src_asset: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&src_asset_info.id)).unwrap();
    assert_eq!(buyer_src_asset.units, 50);

    let buyer_asset1: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&dst_asset1_info.id)).unwrap();
    assert_eq!(buyer_asset1.units, 80);

    let buyer_asset2: Asset =
        serialize::rmp_deserialize(&buyer_account.load_asset(&dst_asset2_info.id)).unwrap();
    assert_eq!(buyer_asset2.units, 100);

    // The guarantor account does not exist in the db!
    assert!(app.account(&guarantor_info.id).is_none());
}
