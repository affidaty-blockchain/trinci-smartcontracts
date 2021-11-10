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

//! NFT contract integration tests

use integration::{
    common::{
        self, AccountInfo, Asset, ASSET_APP_HASH, PUB_KEY1, PUB_KEY2, PUB_KEY3, PUB_KEY4, PUB_KEY5,
        PUB_KEY6, PVT_KEY1, PVT_KEY2, PVT_KEY3, PVT_KEY4, PVT_KEY5, PVT_KEY6,
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
use trinci_sdk::value;

lazy_static! {
    pub static ref NFT_APP_HASH: Hash = common::app_hash("nft.wasm").unwrap();
}

use serde::{Deserialize, Serialize};

// Note: This must be the same of the NFT contract
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct NFTConfig<'a> {
    pub creator: &'a str,
    pub owner: &'a str,
    #[serde(with = "serde_bytes")]
    pub identifier: &'a [u8],
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
    pub sellable: bool,
    pub asset: &'a str,
    pub price: u64,
    pub minimum_price: u64,
    pub creator_fee: u16, // thousandth
    pub intermediary: &'a str,
    pub intermediary_fee: u16, // thousandth
}

const NFT_ALIAS: &str = "NFT";
const CREATOR: &str = "Creator";
const INTERMEDIARY_ALIAS: &str = "Intermediary";
const BUYER_1_ALIAS: &str = "Buyer1";
const BUYER_2_ALIAS: &str = "Buyer2";
const ASSET_ALIAS: &str = "Asset";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(NFT_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(CREATOR, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(INTERMEDIARY_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(BUYER_1_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map.insert(BUYER_2_ALIAS, AccountInfo::new(PUB_KEY5, PVT_KEY5));
        map.insert(ASSET_ALIAS, AccountInfo::new(PUB_KEY6, PVT_KEY6));
        map
    };
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

fn nft_init_tx(
    nft: &AccountInfo,
    asset: &AccountInfo,
    creator: &AccountInfo,
    price: u64,
) -> Transaction {
    // Initialization data
    let args = NFTConfig {
        creator: &creator.id,
        owner: &creator.id,
        identifier: &vec![1, 2, 3],
        data: &vec![4, 5, 6],
        sellable: false,
        asset: &asset.id,
        price: price,
        minimum_price: price,
        creator_fee: 35, // 3.5 %
        intermediary: "",
        intermediary_fee: 0,
    };

    common::create_test_tx(
        &nft.id,
        &creator.pub_key,
        &creator.pvt_key,
        *NFT_APP_HASH,
        "init",
        args,
    )
}

fn nft_get_info_tx(nft: &AccountInfo, caller: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &nft.id,
        &caller.pub_key,
        &caller.pvt_key,
        *NFT_APP_HASH,
        "get_info",
        args,
    )
}

fn nft_buy_tx(
    nft: &AccountInfo,
    buyer: &AccountInfo,
    sellable: bool,
    new_price: u64,
) -> Transaction {
    let args = value!( {
        "sellable": sellable,
        "new_price": new_price,
    });

    common::create_test_tx(
        &nft.id,
        &buyer.pub_key,
        &buyer.pvt_key,
        *NFT_APP_HASH,
        "buy",
        args,
    )
}

fn nft_set_intermediary(
    nft: &AccountInfo,
    caller: &AccountInfo,
    intermediary: &AccountInfo,
    intermediary_fee: u16,
) -> Transaction {
    let args = value!( {
        "intermediary": intermediary.id,
        "intermediary_fee": intermediary_fee,
    });

    common::create_test_tx(
        &nft.id,
        &caller.pub_key,
        &caller.pvt_key,
        *NFT_APP_HASH,
        "set_intermediary",
        args,
    )
}

fn nft_set_new_price(nft: &AccountInfo, caller: &AccountInfo, price: u64) -> Transaction {
    let args = value!( {
        "price": price,
    });

    common::create_test_tx(
        &nft.id,
        &caller.pub_key,
        &caller.pvt_key,
        *NFT_APP_HASH,
        "set_price",
        args,
    )
}

fn nft_set_sellable(nft: &AccountInfo, caller: &AccountInfo, sellable: bool) -> Transaction {
    let args = value!( {
        "sellable": sellable,
    });

    common::create_test_tx(
        &nft.id,
        &caller.pub_key,
        &caller.pvt_key,
        *NFT_APP_HASH,
        "set_sellable",
        args,
    )
}

pub fn asset_init_tx(asset_info: &AccountInfo, asset_name: &str) -> Transaction {
    let args = value!({
        "name": asset_name,
        "description": "My Cool Coin",
        "url": "https://fck.you",
        "max_units": 500_000,
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
    units: u64,
) -> Transaction {
    let args = Delegation {
        delegate: &delegate_info.id,
        units,
        to: Some(&delegate_info.id),
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

fn create_txs() -> Vec<Transaction> {
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let nft_info = ACCOUNTS_INFO.get(NFT_ALIAS).unwrap();
    let creator_info = ACCOUNTS_INFO.get(CREATOR).unwrap();
    let buyer_1_info = ACCOUNTS_INFO.get(BUYER_1_ALIAS).unwrap();
    let buyer_2_info = ACCOUNTS_INFO.get(BUYER_2_ALIAS).unwrap();
    let intermediary_info = ACCOUNTS_INFO.get(INTERMEDIARY_ALIAS).unwrap();

    vec![
        // 0. Initialize src asset
        asset_init_tx(asset_info, ASSET_ALIAS),
        // 1. Mint some units in buyer 1 account.
        asset_mint_tx(asset_info, buyer_1_info, 50_000),
        // 2. Mint some units in buyer 2 account.
        asset_mint_tx(asset_info, buyer_2_info, 150_000),
        // 3. Create NFT account.
        nft_init_tx(nft_info, asset_info, creator_info, 50_000),
        // 4. Get info from NFT
        nft_get_info_tx(nft_info, buyer_1_info),
        // 5. Buyer 1 Add delegation to asset for 50_000 units fo NFT account
        asset_add_delegation_tx(asset_info, buyer_1_info, nft_info, 50_000),
        // 6. Buy with not sellable NFT. This shall fail.
        nft_buy_tx(nft_info, buyer_1_info, false, 100_000),
        // 7. Creator set NFT sellable
        nft_set_sellable(nft_info, creator_info, true),
        // 8. Buy with sellable NFT.
        nft_buy_tx(nft_info, buyer_1_info, false, 100_000),
        // 9. Buyer 2 try to buy not sellable NFT. This shall fail.
        nft_buy_tx(nft_info, buyer_1_info, false, 100_000),
        // 10. Buyer 2 try to set NFT sellable. This shall fail
        nft_set_sellable(nft_info, buyer_2_info, true),
        // 11. Buyer 1 set NFT sellable.
        nft_set_sellable(nft_info, buyer_1_info, true),
        // 12. Buyer 1 set intermediary
        nft_set_intermediary(nft_info, buyer_1_info, intermediary_info, 5), // fee 0.5%
        // 13. Buyer 1 set new_price to 150
        nft_set_new_price(nft_info, buyer_1_info, 150_000),
        // 14. Buyer 2 try to buy with not delegation. This shall fail.
        nft_buy_tx(nft_info, buyer_2_info, false, 200_000),
        // 15. Buyer 2 Add delegation to asset for 150_000 units fo NFT account
        asset_add_delegation_tx(asset_info, buyer_2_info, nft_info, 150_000),
        // 16. Buyer 2 buy the NFT. The NFT will be set to not sellable and the new
        // price will be the same because the new price is lower than the current.
        nft_buy_tx(nft_info, buyer_2_info, true, 50_000),
    ]
}

fn check_rxs(rxs: Vec<Receipt>) {
    let creator_info = ACCOUNTS_INFO.get(CREATOR).unwrap();

    // 0. Initialize src asset
    assert!(rxs[0].success);
    // 1. Mint some units in buyer 1 account.
    assert!(rxs[1].success);
    // 2. Mint some units in buyer 2 account.
    assert!(rxs[2].success);
    // 3. Create NFT account.
    assert!(rxs[3].success);
    // 4. Get info from NFT
    assert!(rxs[4].success);
    let config: Value = rmp_deserialize(&rxs[4].returns).unwrap();
    let owner = config.get(&value!("owner")).unwrap().as_str().unwrap();
    assert_eq!(owner, creator_info.id);
    // 5. Buyer 1 Add delegation to asset for 50 units fo NFT account
    assert!(rxs[5].success);
    // 6. Buy with not sellable NFT. This shall fail.
    assert!(!rxs[6].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[6].returns),
        "smart contract fault: item not sellable".to_string()
    );
    // 7. Creator set NFT sellable
    assert!(rxs[7].success);

    // 8. Buy with sellable NFT.
    assert!(rxs[8].success);

    // 9. Buyer 2 try to buy not sellable NFT. This shall fail.
    assert!(!rxs[9].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[9].returns),
        "smart contract fault: item not sellable".to_string()
    );

    // 10. Buyer 2 try to set NFT sellable. This shall fail
    assert!(!rxs[10].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[10].returns),
        "smart contract fault: not authorized".to_string()
    );

    // 11. Buyer 1 set NFT sellable.
    assert!(rxs[11].success);

    // 12. Buyer 1 set intermediary
    assert!(rxs[12].success);

    // 13. Buyer 1 set new_price to 150
    assert!(rxs[13].success);

    // 14. Buyer 2 try to buy with not delegation. This shall fail.
    assert!(!rxs[14].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[14].returns),
        "smart contract fault: error during withdraw from buyer".to_string()
    );
    // 15. Buyer 2 Add delegation to asset for 150 units fo NFT account
    assert!(rxs[15].success);
    // 16. Buyer 2 buy the NFT.
    assert!(rxs[16].success);
}

#[test]
fn nft_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();

    let rxs = app.exec_txs(txs);
    check_rxs(rxs);

    // Blockchain check.

    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let nft_info = ACCOUNTS_INFO.get(NFT_ALIAS).unwrap();
    let creator_info = ACCOUNTS_INFO.get(CREATOR).unwrap();
    let buyer_1_info = ACCOUNTS_INFO.get(BUYER_1_ALIAS).unwrap();
    let buyer_2_info = ACCOUNTS_INFO.get(BUYER_2_ALIAS).unwrap();
    let intermediary_info = ACCOUNTS_INFO.get(INTERMEDIARY_ALIAS).unwrap();

    let nft_account = app.account(&nft_info.id).unwrap();
    let nft_asset: Asset =
        serialize::rmp_deserialize(&nft_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(nft_asset.units, 0);

    let creator_account = app.account(&creator_info.id).unwrap();
    let creator_asset: Asset =
        serialize::rmp_deserialize(&creator_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(creator_asset.units, 55_250);

    let buyer_1_account = app.account(&buyer_1_info.id).unwrap();
    let buyer_1_asset: Asset =
        serialize::rmp_deserialize(&buyer_1_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(buyer_1_asset.units, 144_000);

    let buyer_2_account = app.account(&buyer_2_info.id).unwrap();
    let buyer_2_asset: Asset =
        serialize::rmp_deserialize(&buyer_2_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(buyer_2_asset.units, 0);

    let intermediary_account = app.account(&intermediary_info.id).unwrap();
    let intermediary_asset: Asset =
        serialize::rmp_deserialize(&intermediary_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(intermediary_asset.units, 750);
}
