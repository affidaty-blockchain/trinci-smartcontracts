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

use lazy_static::lazy_static;
use serde_value::Value;
use std::collections::HashMap;
use trinci_core::{crypto::Hash, Receipt, Transaction};

use trinci_sdk::{rmp_deserialize, value};

use integration::{
    common::{
        self, AccountInfo, Asset, ASSET_APP_HASH, PUB_KEY1, PUB_KEY2, PUB_KEY3, PUB_KEY4, PUB_KEY5,
        PVT_KEY1, PVT_KEY2, PVT_KEY3, PVT_KEY4, PVT_KEY5,
    },
    TestApp,
};

lazy_static! {
    pub static ref WITHDRAW_APP_HASH: Hash = common::app_hash("withdraw.wasm").unwrap();
}

const EXCHANGE_ALIAS: &str = "Exchange";
const WITHDRAW_ALIAS: &str = "Withdraw";
const CUSTOMER_ALIAS: &str = "Customer";
const CURRENCY_ASSET_ALIAS: &str = "Currency_Asset";
const WITHDRAWN_ASSET_ALIAS: &str = "Withdrawn_Asset";

// TODO: insert this in the trinci-integration library.
// TODO: maybe could be an idea to insert the asset transactions functions in
// the trinci-integration library as well.
pub const PUB_KEY7: &str = "044d469f3a0d6b50ab570ae4724446a5730fd7311d399dcf3cb3542eb6773b0e571f63197255ae29c4f154c89962ca1e36b78a90295fa6a242973743a14685fe60b644c7923530edf0967f4f639a35fb5ed081e660b2606e5f1f394eaa75f2630c";
pub const PVT_KEY7: &str = "fa646fa1f6d3b876b0f57700d0134d11fd1913073092e23f3df753289db64a73cc7b8920a39136e697a01e677f4834b5";

pub const PUB_KEY8: &str = "044717583406373a9b47f564e6af4c28d9bc45b11da5de0fdfcd9928dab12eaacaedfabc7357565f2ecfa222f4b4e654a727397c3cad00a2af4c21defe5a0b403d3e62390b71633b203c268fd35ffe2e83fc7c602c2ae19274707a96f579e5439e";
pub const PVT_KEY8: &str = "f9a2619f076ca99870bb90b4faf63a9ddedc031b07a1f2ea82305b71dc43d040b64ff56af043c887a24f5c4148b15dad";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(EXCHANGE_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(WITHDRAW_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(CUSTOMER_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(CURRENCY_ASSET_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map.insert(WITHDRAWN_ASSET_ALIAS, AccountInfo::new(PUB_KEY5, PVT_KEY5));
        map
    };
}

fn withdraw_init_tx(
    withdraw: &AccountInfo,
    exchange: &AccountInfo,
    customer: &AccountInfo,
    currency_asset: &AccountInfo,
    currency_units: u64,
    withdrawn_asset: &AccountInfo,
    withdrawn_units: u64,
) -> Transaction {
    // Initialization data
    let args = value!({
        "customer": customer.id,
        "exchange": exchange.id,
        "currency_asset": {
            "id": currency_asset.id,
            "units": currency_units
        },
        "withdrawn_asset": {
            "id": withdrawn_asset.id,
            "units": withdrawn_units,
        },
    });

    common::create_test_tx(
        &withdraw.id,
        &exchange.pub_key,
        &exchange.pvt_key,
        *WITHDRAW_APP_HASH,
        "init",
        args,
    )
}

fn withdraw_update_tx(withdraw: &AccountInfo, caller: &AccountInfo, status: &str) -> Transaction {
    let args = value!({
        "status": status,
    });

    common::create_test_tx(
        &withdraw.id,
        &caller.pub_key,
        &caller.pvt_key,
        *WITHDRAW_APP_HASH,
        "update",
        args,
    )
}

fn withdraw_get_info_tx(withdraw: &AccountInfo, caller: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &withdraw.id,
        &caller.pub_key,
        &caller.pvt_key,
        *WITHDRAW_APP_HASH,
        "get_info",
        args,
    )
}

pub fn asset_init_tx(
    asset_info: &AccountInfo,
    asset_name: &str,
    authorized_info: &AccountInfo,
) -> Transaction {
    let args = value!({
        "name": asset_name,
        "description": "My Cool Coin",
        "url": "https://fck.you",
        "max_units": 100_000,
        "authorized": [
            authorized_info.id
        ],
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

fn create_txs(update_status: &str) -> Vec<Transaction> {
    let withdraw_info = ACCOUNTS_INFO.get(WITHDRAW_ALIAS).unwrap();
    let currency_asset_info = ACCOUNTS_INFO.get(CURRENCY_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();
    let withdrawn_asset_info = ACCOUNTS_INFO.get(WITHDRAWN_ASSET_ALIAS).unwrap();

    vec![
        // 0. Initialize currency asset
        asset_init_tx(currency_asset_info, CURRENCY_ASSET_ALIAS, withdraw_info), // For now withdraw account must be authorized to burn
        // 1. Mint some units in exchange account.
        asset_mint_tx(currency_asset_info, exchange_info, 7_500),
        // 2. Initialize withdrawn asset
        asset_init_tx(withdrawn_asset_info, WITHDRAWN_ASSET_ALIAS, withdraw_info), // For now withdraw account must be authorized to burn
        // 3. Mint some units in customer account.
        asset_mint_tx(withdrawn_asset_info, customer_info, 5_000),
        // 4. Transfer funds from exchange to withdraw account.
        asset_transfer_tx(currency_asset_info, exchange_info, withdraw_info, 1_000),
        // 5. Initialize withdraw account. This shall fail because there are not enough funds
        withdraw_init_tx(
            withdraw_info,
            exchange_info,
            customer_info,
            currency_asset_info,
            1_000,
            withdrawn_asset_info,
            300,
        ),
        // 6. Transfer funds from customer to withdraw account.
        asset_transfer_tx(withdrawn_asset_info, customer_info, withdraw_info, 300),
        // 7. Initialize withdraw account.
        withdraw_init_tx(
            withdraw_info,
            exchange_info,
            customer_info,
            currency_asset_info,
            1_000,
            withdrawn_asset_info,
            300,
        ),
        // 8. Get information from not authorized account. This shall fail
        withdraw_get_info_tx(withdraw_info, currency_asset_info),
        // 9. Get information from customer
        withdraw_get_info_tx(withdraw_info, customer_info),
        // 10. Update with bad argument. This shall fail
        withdraw_update_tx(withdraw_info, exchange_info, "123"),
        // 11. Update from customer. This shall fail because the customer is not authorized
        withdraw_update_tx(withdraw_info, customer_info, "OK"),
        // 12. Update from exchange.
        withdraw_update_tx(withdraw_info, exchange_info, update_status),
        // 13. Update from exchange. This shall fail
        withdraw_update_tx(withdraw_info, exchange_info, update_status),
        // 14. Get information from exchange
        withdraw_get_info_tx(withdraw_info, exchange_info),
    ]
}

fn check_rxs(rxs: Vec<Receipt>, update_status: &str) {
    // 0. Initialize currency asset
    assert!(rxs[0].success);
    // 1. Mint some units in exchange account.
    assert!(rxs[1].success);
    // 2. Initialize withdrawn asset
    assert!(rxs[2].success);
    // 3. Mint some units in customer account.
    assert!(rxs[3].success);
    // 4. Transfer funds from exchange to withdraw account.
    assert!(rxs[4].success);
    // 5. Initialize exchange account. This shall fail because there are not enough funds
    assert!(!rxs[5].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[5].returns),
        "smart contract fault: not enough funds"
    );
    // 6. Transfer funds from customer to withdraw account.
    assert!(rxs[6].success);
    // 7. Initialize withdraw account.
    assert!(rxs[7].success);

    // 8. Get information from not authorized account. This shall fail
    assert!(!rxs[8].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[8].returns),
        "smart contract fault: not authorized"
    );
    // 9. Get information from customer
    assert!(rxs[9].success);

    let info: Value = rmp_deserialize(&rxs[9].returns).unwrap();
    let status = info.get(&value!("status")).unwrap().as_str().unwrap();
    assert_eq!(status, "open");

    // 10. Update with bad argument. This shall fail
    assert!(!rxs[10].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[10].returns),
        "smart contract fault: bad update arguments"
    );
    // 11. Update from customer. This shall fail because the customer is not authorized
    assert!(!rxs[11].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[11].returns),
        "smart contract fault: not authorized"
    );
    // 12. Update from exchange.
    assert!(rxs[12].success);
    // 13. Update from exchange. This shall fail
    assert!(!rxs[13].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[13].returns),
        "smart contract fault: already closed"
    );
    // 14. Get information from exchange
    assert!(rxs[14].success);

    let info: Value = rmp_deserialize(&rxs[14].returns).unwrap();
    let status = info.get(&value!("status")).unwrap().as_str().unwrap();
    if update_status == "OK" {
        assert_eq!(status, "closed success");
    } else {
        assert_eq!(status, "closed failure");
    }
}

#[test]
fn withdraw_success_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs("OK");
    let rxs = app.exec_txs(txs);
    check_rxs(rxs, "OK");

    // Blockchain check.

    let withdraw_info = ACCOUNTS_INFO.get(WITHDRAW_ALIAS).unwrap();
    let currency_asset_info = ACCOUNTS_INFO.get(CURRENCY_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();
    let withdrawn_asset_info = ACCOUNTS_INFO.get(WITHDRAWN_ASSET_ALIAS).unwrap();

    let withdraw_account = app.account(&withdraw_info.id).unwrap();
    let withdraw_currency_asset: Asset =
        rmp_deserialize(&withdraw_account.load_asset(&currency_asset_info.id)).unwrap();
    assert_eq!(withdraw_currency_asset.units, 0);

    let withdraw_withdrawn_asset: Asset =
        rmp_deserialize(&withdraw_account.load_asset(&withdrawn_asset_info.id)).unwrap();
    assert_eq!(withdraw_withdrawn_asset.units, 0);

    let exchange_account = app.account(&exchange_info.id).unwrap();
    let exchange_currency_asset: Asset =
        rmp_deserialize(&exchange_account.load_asset(&currency_asset_info.id)).unwrap();
    assert_eq!(exchange_currency_asset.units, 6_500);

    let customer_account = app.account(&customer_info.id).unwrap();
    let seller_src_asset: Asset =
        rmp_deserialize(&customer_account.load_asset(&withdrawn_asset_info.id)).unwrap();
    assert_eq!(seller_src_asset.units, 4_700);
}

#[test]
fn withdraw_refund_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs("KO");
    let rxs = app.exec_txs(txs);
    check_rxs(rxs, "KO");

    // Blockchain check.

    let withdraw_info = ACCOUNTS_INFO.get(WITHDRAW_ALIAS).unwrap();
    let currency_asset_info = ACCOUNTS_INFO.get(CURRENCY_ASSET_ALIAS).unwrap();
    let exchange_info = ACCOUNTS_INFO.get(EXCHANGE_ALIAS).unwrap();
    let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();
    let withdrawn_asset_info = ACCOUNTS_INFO.get(WITHDRAWN_ASSET_ALIAS).unwrap();

    let withdraw_account = app.account(&withdraw_info.id).unwrap();
    let withdraw_currency_asset: Asset =
        rmp_deserialize(&withdraw_account.load_asset(&currency_asset_info.id)).unwrap();
    assert_eq!(withdraw_currency_asset.units, 0);

    let withdraw_withdrawn_asset: Asset =
        rmp_deserialize(&withdraw_account.load_asset(&withdrawn_asset_info.id)).unwrap();
    assert_eq!(withdraw_withdrawn_asset.units, 0);

    let exchange_account = app.account(&exchange_info.id).unwrap();
    let exchange_currency_asset: Asset =
        rmp_deserialize(&exchange_account.load_asset(&currency_asset_info.id)).unwrap();
    assert_eq!(exchange_currency_asset.units, 7_500);

    let customer_account = app.account(&customer_info.id).unwrap();
    let seller_src_asset: Asset =
        rmp_deserialize(&customer_account.load_asset(&withdrawn_asset_info.id)).unwrap();
    assert_eq!(seller_src_asset.units, 5_000);
}
