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

//! Pay meals integration tests

use integration::{
    common::{self, *},
    TestApp,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use trinci_core::{base::serialize, crypto::Hash, Receipt, Transaction};
use trinci_sdk::rmp_deserialize;

use serde_value::Value;

lazy_static! {
    pub static ref PAYMEALS_APP_HASH: Hash = common::app_hash("pay_meal.wasm").unwrap();
    pub static ref ASSET_APP_HASH: Hash = common::app_hash("asset.wasm").unwrap();
}

use serde::{Deserialize, Serialize};

const PAY_ALIAS: &str = "PayMeals";
const RESTAURATEUR_ALIAS: &str = "Mario's_Pizza";
const MARCO_ALIAS: &str = "Marco";
const LUIGI_ALIAS: &str = "Luigi";
const BRUNO_ALIAS: &str = "Bruno";

const PIERO_ALIAS: &str = "Piero";

const ASSET_ALIAS: &str = "Asset";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(PAY_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(RESTAURATEUR_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(MARCO_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(LUIGI_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map.insert(BRUNO_ALIAS, AccountInfo::new(PUB_KEY5, PVT_KEY5));
        map.insert(PIERO_ALIAS, AccountInfo::new(PUB_KEY6, PVT_KEY6));
        map.insert(ASSET_ALIAS, AccountInfo::new(PUB_KEY7, PVT_KEY7));
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

fn contract_init_tx(
    contract: &AccountInfo,
    restaurateur: &AccountInfo,
    asset: &AccountInfo,
    customer1: &AccountInfo,
    customer2: &AccountInfo,
    customer3: &AccountInfo,
    part: u64,
) -> Transaction {
    // Initialization data
    let args = value!({
        "restaurateur": restaurateur.id,
        "asset": asset.id,
        "part": part,
        "customers": {
            customer1.id.clone(): false,
            customer2.id.clone(): false,
            customer3.id.clone(): false,
        },
        "status": "open",
    });

    common::create_test_tx(
        &contract.id,
        &contract.pub_key,
        &contract.pvt_key,
        *PAYMEALS_APP_HASH,
        "init",
        args,
    )
}

fn contract_get_info_tx(contract: &AccountInfo, caller: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &contract.id,
        &caller.pub_key,
        &caller.pvt_key,
        *PAYMEALS_APP_HASH,
        "get_info",
        args,
    )
}

fn contract_apply_tx(contract: &AccountInfo, customer: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &contract.id,
        &customer.pub_key,
        &customer.pvt_key,
        *PAYMEALS_APP_HASH,
        "apply",
        args,
    )
}

fn contract_close_tx(contract: &AccountInfo, caller: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &contract.id,
        &caller.pub_key,
        &caller.pvt_key,
        *PAYMEALS_APP_HASH,
        "close",
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
    create_test_tx(
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
    create_test_tx(
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
    create_test_tx(
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
    let contract_info = ACCOUNTS_INFO.get(PAY_ALIAS).unwrap();
    let restaurateur_info = ACCOUNTS_INFO.get(RESTAURATEUR_ALIAS).unwrap();
    let marco_info = ACCOUNTS_INFO.get(MARCO_ALIAS).unwrap();
    let luigi_info = ACCOUNTS_INFO.get(LUIGI_ALIAS).unwrap();
    let bruno_info = ACCOUNTS_INFO.get(BRUNO_ALIAS).unwrap();
    let piero_info = ACCOUNTS_INFO.get(PIERO_ALIAS).unwrap();
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();

    vec![
        // 0. Initialize src asset
        asset_init_tx(asset_info, ASSET_ALIAS),
        // 1. Mint some units in customers account.
        asset_mint_tx(asset_info, marco_info, 100),
        // 2. Mint some units in customers account.
        asset_mint_tx(asset_info, luigi_info, 100),
        // 3. Mint some units in customers account.
        asset_mint_tx(asset_info, bruno_info, 100),
        // 4. Mint some units in customers account.
        asset_mint_tx(asset_info, piero_info, 100),
        // 5. Initialize contract account.
        contract_init_tx(
            contract_info,
            restaurateur_info,
            asset_info,
            marco_info,
            luigi_info,
            bruno_info,
            30,
        ),
        // 6. Marco get the contract info
        contract_get_info_tx(contract_info, marco_info),
        // 7. Luigi add delegation to pay the bill
        asset_add_delegation_tx(asset_info, luigi_info, contract_info, 30),
        // 8. Luigi pays his bill
        contract_apply_tx(contract_info, luigi_info),
        // 9. Piero tries to pay. This shall fail.
        contract_apply_tx(contract_info, piero_info),
        // 10. Piero tries to get contract information. This shall fail.
        contract_get_info_tx(contract_info, piero_info),
        // 11. Marco tries to close the contract. This shall fail.
        contract_close_tx(contract_info, marco_info),
        // 12. Mario (the restaurateur) tries to close the contract.
        contract_close_tx(contract_info, restaurateur_info),
        // 13. Bruno add delegation to pay the bill
        asset_add_delegation_tx(asset_info, bruno_info, contract_info, 30),
        // 14. Bruno pays his bill
        contract_apply_tx(contract_info, bruno_info),
        // 15. Marco add delegation to pay the bill
        asset_add_delegation_tx(asset_info, marco_info, contract_info, 30),
        // 16. Marco pays his bill
        contract_apply_tx(contract_info, marco_info),
        // 17. Mario (the restaurateur) tries to close the contract.
        contract_close_tx(contract_info, restaurateur_info),
        // 18. Mario get the contract information
        contract_get_info_tx(contract_info, restaurateur_info),
    ]
}

fn check_rxs(rxs: Vec<Receipt>) {
    // 0. Initialize src asset
    assert!(rxs[0].success);
    // 1. Mint some units in customers account.
    assert!(rxs[1].success);
    // 2. Mint some units in customers account.
    assert!(rxs[2].success);
    // 3. Mint some units in customers account.
    assert!(rxs[3].success);
    // 4. Mint some units in customers account.
    assert!(rxs[4].success);
    // 5. Initialize contract account.
    assert!(rxs[5].success);
    // 6. Marco get the contract info
    assert!(rxs[6].success);
    // Checks on the config
    let config: Value = rmp_deserialize(&rxs[6].returns).unwrap();
    let status = config.get(&value!("status")).unwrap().as_str().unwrap();
    assert_eq!(status, "open");
    // 7. Luigi add delegation to pay the bill
    assert!(rxs[7].success);
    // 8. Luigi pays his bill
    assert!(rxs[8].success);
    // 9. Piero tries to pay. This shall fail.
    assert!(!rxs[9].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[9].returns),
        "smart contract fault: not authorized"
    );
    // 10. Piero tries to get contract information. This shall fail.
    assert!(!rxs[10].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[9].returns),
        "smart contract fault: not authorized"
    );
    // 11. Marco tries to close the contract. This shall fail.
    assert!(!rxs[11].success);
    assert_eq!(
        String::from_utf8_lossy(&rxs[9].returns),
        "smart contract fault: not authorized"
    );
    // 12. Mario (the restaurateur) tries to close the contract.
    assert!(rxs[12].success);
    // 13. Bruno add delegation to pay the bill
    assert!(rxs[13].success);
    // 14. Bruno pays his bill
    assert!(rxs[14].success);
    // 15. Marco add delegation to pay the bill
    assert!(rxs[15].success);
    // 16. Marco pays his bill
    assert!(rxs[16].success);
    // 17. Mario (the restaurateur) tries to close the contract.
    assert!(rxs[17].success);
    // 18. Mario get the contract information
    assert!(rxs[18].success);
    // Checks on the config
    let config: Value = rmp_deserialize(&rxs[18].returns).unwrap();
    let status = config.get(&value!("status")).unwrap().as_str().unwrap();
    assert_eq!(status, "close");
}

#[test]
fn pay_meal_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);

    // Blockchain check.
    let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    let contract_info = ACCOUNTS_INFO.get(PAY_ALIAS).unwrap();
    let restaurateur_info = ACCOUNTS_INFO.get(RESTAURATEUR_ALIAS).unwrap();
    let marco_info = ACCOUNTS_INFO.get(MARCO_ALIAS).unwrap();
    let luigi_info = ACCOUNTS_INFO.get(LUIGI_ALIAS).unwrap();
    let bruno_info = ACCOUNTS_INFO.get(BRUNO_ALIAS).unwrap();
    let piero_info = ACCOUNTS_INFO.get(PIERO_ALIAS).unwrap();

    let contract_account = app.account(&contract_info.id).unwrap();
    let contract_asset: Asset =
        serialize::rmp_deserialize(&contract_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(contract_asset.units, 0);

    let restaurateur_account = app.account(&restaurateur_info.id).unwrap();
    let restaurateur_asset: Asset =
        serialize::rmp_deserialize(&restaurateur_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(restaurateur_asset.units, 90);

    let bruno_account = app.account(&bruno_info.id).unwrap();
    let bruno_asset: Asset =
        serialize::rmp_deserialize(&bruno_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(bruno_asset.units, 70);

    let marco_account = app.account(&marco_info.id).unwrap();
    let marco_asset: Asset =
        serialize::rmp_deserialize(&marco_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(marco_asset.units, 70);

    let luigi_account = app.account(&luigi_info.id).unwrap();
    let luigi_asset: Asset =
        serialize::rmp_deserialize(&luigi_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(luigi_asset.units, 70);

    let piero_account = app.account(&piero_info.id).unwrap();
    let piero_asset: Asset =
        serialize::rmp_deserialize(&piero_account.load_asset(&asset_info.id)).unwrap();
    assert_eq!(piero_asset.units, 100);
}
