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
        self, AccountInfo, SerdeValue, PUB_KEY1, PUB_KEY2, PUB_KEY3, PVT_KEY1, PVT_KEY2, PVT_KEY3,
    },
    TestApp,
};
use lazy_static::lazy_static;
use serde_value::Value;

use std::collections::HashMap;
use trinci_core::{
    crypto::{Hash, HashAlgorithm},
    db::{Db, DbFork},
    Account, Receipt, Transaction,
};

use trinci_sdk::{rmp_deserialize, value};

lazy_static! {
    pub static ref SERVICE_APP_HASH: Hash = common::app_hash("service.wasm").unwrap();
}

const SERVICE_ALIAS: &str = "Service";
const SUBMITTER_ALIAS: &str = "Submitter";
const ASSET_ALIAS: &str = "FCK";

const SERVICE_ID: &str = "TRINCI";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(SERVICE_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1, false));
        map.insert(SUBMITTER_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2, false));
        map.insert(ASSET_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3, false));
        map
    };
    static ref CRYPTO_APP_HASH: Hash = common::app_hash("crypto.wasm").unwrap();
    static ref CRYPTO_APP_HASH_HEX: String = hex::encode(&CRYPTO_APP_HASH.as_bytes());
    static ref CRYPTO_APP_BIN: Vec<u8> = common::app_read("crypto.wasm").unwrap();
    static ref SERVICE_APP_BIN: Vec<u8> = common::app_read("service.wasm").unwrap();
    static ref SERVICE_APP_HASH_HEX: String = hex::encode(&SERVICE_APP_HASH.as_bytes());
    static ref CONTRACTS_DATA: Value = {
        value!(
        {
            "name": "mycontract",
            "version": "0.1.0",
            "publisher": ACCOUNTS_INFO.get(SERVICE_ALIAS).unwrap().id,
            "description": "This is my personal contract",
            "url": "http://www.mycontract.org",
        })
    };
}

fn create_service_account(app: &mut TestApp) {
    let db_arc = app.block_svc.db_arc();
    let mut db = db_arc.write();
    // Create an empty account.
    let account = Account::new(SERVICE_ID, Some(*SERVICE_APP_HASH));
    // Update account within the DB.
    let mut fork = db.fork_create();
    fork.store_account(account);
    // Store the SERVICE contract binary in the SERVICE account data
    // (for service wasm loader lookup resolution)
    // TODO: probably we shall store service contract registration data as well.
    let mut code_key = String::from("contracts:code:");
    code_key.push_str(&SERVICE_APP_HASH_HEX);

    fork.store_account_data(SERVICE_ID, &code_key, SERVICE_APP_BIN.clone());
    db.fork_merge(fork).unwrap();
}

fn create_contract_registration_tx(caller_info: &AccountInfo) -> Transaction {
    let args = value!({
        "name": "mycontract",
        "version": "0.1.0",
        "description": "This is my personal contract",
        "url": "http://www.mycontract.org",
        "bin": SerdeValue::Bytes(CRYPTO_APP_BIN.clone()),
    });

    common::create_test_tx(
        SERVICE_ID,
        &caller_info.pub_key,
        &caller_info.pvt_key,
        *SERVICE_APP_HASH,
        "contract_registration",
        args,
    )
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
        *CRYPTO_APP_HASH,
        "init",
        args,
    )
}

fn service_init_tx(service_info: &AccountInfo) -> Transaction {
    common::create_test_tx(
        "TRINCI",
        &service_info.pub_key,
        &service_info.pvt_key,
        *SERVICE_APP_HASH,
        "init",
        SERVICE_APP_BIN.clone(),
    )
}

fn create_contract_registration_txs() -> Vec<Transaction> {
    let submitter_info = ACCOUNTS_INFO.get(SUBMITTER_ALIAS).unwrap();
    let service_info = ACCOUNTS_INFO.get(SERVICE_ALIAS).unwrap();

    vec![
        // 0. Call to an unregistered contract. Expected to fail.
        asset_init_tx(submitter_info),
        // 1. Initialize the service
        service_init_tx(service_info),
        // 2. Register the contract.
        create_contract_registration_tx(service_info),
    ]
}
fn check_contract_registration_rxs_first(rxs: Vec<Receipt>) {
    // 0. Call to an unregistered contract. Expected to fail.
    assert!(!rxs[0].success);
    let error = String::from_utf8_lossy(&rxs[0].returns);
    assert_eq!(error, "invalid contract hash: cannot bind the contract to the account");
    // 1. Initialize the service
    assert!(rxs[1].success);
    // 2. Register the contract.
    assert!(rxs[2].success);
    let contract_hash: String = rmp_deserialize(&rxs[2].returns).unwrap();
    assert_eq!(*CRYPTO_APP_HASH_HEX, contract_hash);
}

fn check_contract_registration_rxs_second(rxs: Vec<Receipt>) {
    // 0. Call to an unregistered contract. Expected to fail.
    assert!(rxs[0].success);
    // 1. Initialize the service
    assert!(!rxs[1].success);
    let msg = String::from_utf8_lossy(&rxs[1].returns);
    assert_eq!(msg, "smart contract fault: Already initialized.");
    // 2. Register the contract.
    assert!(!rxs[2].success);
    let msg = String::from_utf8_lossy(&rxs[2].returns);
    assert_eq!(
        msg,
        "smart contract fault: Smart contract with the same name and version has already been published."
    );
}

#[test]
fn test_contract_registration() {
    // Instance the application without contracts pre-registration.
    let mut app = TestApp::new("");

    // Create and store the service account.
    create_service_account(&mut app);

    // Create and execute transactions.
    let txs = create_contract_registration_txs();
    let rxs = app.exec_txs(txs);
    check_contract_registration_rxs_first(rxs);

    // Check registered contract availability.
    let txs = create_contract_registration_txs();
    let rxs = app.exec_txs(txs);
    check_contract_registration_rxs_second(rxs);

    // Blockchain check.

    let mut code_key = String::from("contracts:metadata:");
    code_key.push_str(&CRYPTO_APP_HASH_HEX);

    let contract_data = app.account_data(SERVICE_ID, &code_key).unwrap();
    let contract_data: Value = rmp_deserialize(&contract_data).unwrap();
    assert_eq!(contract_data, *CONTRACTS_DATA);

    let contract_bin_exp = &*CRYPTO_APP_BIN;
    let contract_id = hex::encode(Hash::from_data(HashAlgorithm::Sha256, contract_bin_exp));
    let mut code_key = String::from("contracts:code:");
    code_key.push_str(&contract_id);

    let contract_bin = app.account_data(SERVICE_ID, &code_key).unwrap();
    assert_eq!(&contract_bin, contract_bin_exp);
}
