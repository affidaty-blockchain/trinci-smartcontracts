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

//! 4rya contract integration tests
//!
//! // FIXME // TODO  Add more tests with failure situations
//! // FIXME // TODO  Add certificate set (trough a signed HEX tx) and verify
//!

use integration::{
    common::{self},
    TestApp,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_value::Value;
use std::collections::{BTreeMap, HashMap};
use trinci_core::crypto::ecdsa::{self, KeyPair};
use trinci_core::{
    base::serialize::rmp_serialize,
    crypto::{ecdsa::PublicKey as EcdsaPublicKey, sign::PublicKey, Hash},
    Receipt, Transaction,
};
use trinci_core::{crypto::ecdsa::CurveId, TransactionData};
use trinci_sdk::{rmp_serialize_named, value};

// TMP Using keypair instead pub/pvt keys for account
fn kp_create_test_tx_data(
    id: &str,
    public_key: PublicKey,
    contract_hash: Hash,
    method: &str,
    args: impl Serialize,
) -> TransactionData {
    static mut MYNONCE: u64 = 0;
    let args = rmp_serialize_named(&args).unwrap();

    let nonce = unsafe {
        MYNONCE += 1;
        MYNONCE.to_be_bytes().to_vec()
    };
    TransactionData {
        account: id.to_string(),
        nonce,
        network: "skynet".to_string(),
        contract: Some(contract_hash),
        method: method.to_string(),
        caller: public_key,
        args,
    }
}
fn kp_create_test_tx(
    id: &str,
    ecdsa_keypair: ecdsa::KeyPair,
    target: Hash,
    method: &str,
    args: impl Serialize,
) -> Transaction {
    let keypair = trinci_core::crypto::sign::KeyPair::Ecdsa(ecdsa_keypair);
    let public_key = keypair.public_key();
    let data = kp_create_test_tx_data(id, public_key, target, method, args);

    let signature = data.sign(&keypair).unwrap();
    Transaction { data, signature }
}

// Hash Algorithms available
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
}

lazy_static! {
    pub static ref ARYA_APP_HASH: Hash = common::app_hash("arya.wasm").unwrap();
    pub static ref CRYPTO_APP_HASH: Hash = common::app_hash("crypto.wasm").unwrap();
}

const ARYA_ALIAS: &str = "4Rya";
const CRYPTO_ALIAS: &str = "Crypto";
const CERTIFIER_1_ALIAS: &str = "Certifier_1";
const CERTIFIER_2_ALIAS: &str = "Certifier_2";
const USER_ALIAS: &str = "User";
const DELEGATE_ALIAS: &str = "Delegate";

const ARYA_PKCS8: &str = "";
const CRYPTO_PKCS8: &str = "";
const CERTIFIER_1_PKCS8: &str = "";
const CERTIFIER_2_PKCS8: &str = "";
const USER_PKCS8: &str = "";
const DELEGATE_PKCS8: &str = "";

lazy_static! {
    static ref KEYPAIRS_INFO: HashMap<&'static str, KeyPair> = {
        let mut map = HashMap::new();
        map.insert(
            ARYA_ALIAS,
            KeyPair::from_pkcs8_bytes(CurveId::Secp384R1, &hex::decode(ARYA_PKCS8).unwrap())
                .unwrap(),
        );
        map.insert(
            CRYPTO_ALIAS,
            KeyPair::from_pkcs8_bytes(CurveId::Secp384R1, &hex::decode(CRYPTO_PKCS8).unwrap())
                .unwrap(),
        );
        map.insert(
            CERTIFIER_1_ALIAS,
            KeyPair::from_pkcs8_bytes(CurveId::Secp384R1, &hex::decode(CERTIFIER_1_PKCS8).unwrap())
                .unwrap(),
        );
        map.insert(
            CERTIFIER_2_ALIAS,
            KeyPair::from_pkcs8_bytes(CurveId::Secp384R1, &hex::decode(CERTIFIER_2_PKCS8).unwrap())
                .unwrap(),
        );
        map.insert(
            USER_ALIAS,
            KeyPair::from_pkcs8_bytes(CurveId::Secp384R1, &hex::decode(USER_PKCS8).unwrap())
                .unwrap(),
        );
        map.insert(
            DELEGATE_ALIAS,
            KeyPair::from_pkcs8_bytes(CurveId::Secp384R1, &hex::decode(DELEGATE_PKCS8).unwrap())
                .unwrap(),
        );
        map
    };
}

// Hash arguments
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct HashArgs<'a> {
    pub algorithm: HashAlgorithm,
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
}

fn crypto_hash_tx(owner_info: &KeyPair, user_info: &KeyPair) -> Transaction {
    let args = HashArgs {
        algorithm: HashAlgorithm::Sha256,
        data: &[1, 2, 3],
    };

    kp_create_test_tx(
        &owner_info.public_key().to_account_id(),
        *user_info,
        *CRYPTO_APP_HASH,
        "hash",
        args,
    )
}

fn create_profile_data() -> Value {
    value! ({
            "name": "John",
            "surname": "Doe",
            "sex": "male",
            "tel": "1634829548",
            "email": "john.doe@mail.net",
    })
}

fn create_update_profile_data() -> Value {
    value!({
        "surname": "Doe",
        "testField": "testString",
    })
}

fn arya_init_tx(arya_info: &KeyPair, crypto_info: &KeyPair) -> Transaction {
    let args = value! ({"crypto": crypto_info.public_key().to_account_id()});

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        *arya_info,
        *ARYA_APP_HASH,
        "init",
        args,
    )
}

fn arya_set_profile_data_tx(
    arya_info: &KeyPair,
    target_account: &KeyPair,
    args: Value,
) -> Transaction {
    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        *target_account,
        *ARYA_APP_HASH,
        "set_profile_data",
        args,
    )
}

fn arya_remove_profile_data_tx(arya_info: &KeyPair, target_account: &KeyPair) -> Transaction {
    let args = value!(vec!["testfield"]);

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        *target_account,
        *ARYA_APP_HASH,
        "remove_profile_data",
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
    arya_info: &KeyPair,
    delegate_info: &KeyPair,
    delegator_info: &KeyPair,
    target_account: &KeyPair,
) -> Transaction {
    let mut capabilities = BTreeMap::<&str, bool>::new();
    capabilities.insert("*", true);
    capabilities.insert("method1", false);

    let delegator = PublicKey::Ecdsa(EcdsaPublicKey {
        curve_id: CurveId::Secp384R1,
        value: delegator_info.public_key().value,
    });

    let data = DelegationData {
        delegate: &delegate_info.public_key().to_account_id(),
        delegator: delegator.clone(),
        network: "skynet",
        target: &target_account.public_key().to_account_id(),
        expiration: 123u64,
        capabilities: capabilities.clone(),
    };

    let data_to_sign = rmp_serialize(&data).unwrap();

    let sign = delegator_info.sign(&data_to_sign).unwrap();

    let delegation = Delegation {
        data,
        signature: &sign,
    };

    let delegation = rmp_serialize(&delegation).unwrap();

    let args = SetDelegationArgs {
        key: "",
        delegation: &delegation,
    };

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        *delegator_info,
        *ARYA_APP_HASH,
        "set_delegation",
        args,
    )
}

fn arya_remove_delegation_tx(
    arya_info: &KeyPair,
    delegate_info: &KeyPair,
    delegator_info: &KeyPair,
) -> Transaction {
    let args = value!({ "delegate": delegate_info.public_key().to_account_id(),
        "delegator": delegator_info.public_key().to_account_id(),
        "targets": vec!["*"],
    });

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        *delegate_info,
        *ARYA_APP_HASH,
        "remove_delegation",
        args,
    )
}

fn arya_verify_capabilities_tx(
    arya_info: &KeyPair,
    delegate_info: &KeyPair,
    delegator_info: &KeyPair,
    target_info: &KeyPair,
    method: &str,
) -> Transaction {
    let args = value!({
        "delegate": delegate_info.public_key().to_account_id(),
        "delegator": delegator_info.public_key().to_account_id(),
        "target": target_info.public_key().to_account_id(),
        "method": method,
    });

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        *target_info,
        *ARYA_APP_HASH,
        "verify_capability",
        args,
    )
}

fn create_txs() -> Vec<Transaction> {
    let arya_info = KEYPAIRS_INFO.get(ARYA_ALIAS).unwrap();
    let crypto_info = KEYPAIRS_INFO.get(CRYPTO_ALIAS).unwrap();
    let target_info = KEYPAIRS_INFO.get(USER_ALIAS).unwrap();
    let certifier_1_info = KEYPAIRS_INFO.get(CERTIFIER_1_ALIAS).unwrap();
    let certifier_2_info = KEYPAIRS_INFO.get(CERTIFIER_2_ALIAS).unwrap();
    let delegate_info = KEYPAIRS_INFO.get(DELEGATE_ALIAS).unwrap();

    vec![
        // 0. Crypto Hash to initializate crypto contract in account
        crypto_hash_tx(crypto_info, target_info),
        // 1. init arya.
        arya_init_tx(arya_info, crypto_info),
        // 2. set profile data
        arya_set_profile_data_tx(arya_info, target_info, create_profile_data()),
        // 3. update profile data
        arya_set_profile_data_tx(arya_info, target_info, create_update_profile_data()),
        // 4. remove profile data
        arya_remove_profile_data_tx(arya_info, target_info),
        // 5. set delegation 1
        arya_set_delegation_tx(arya_info, delegate_info, certifier_1_info, target_info),
        // 6. set delegation 2
        arya_set_delegation_tx(arya_info, delegate_info, certifier_2_info, target_info),
        // 7. remove delegation 2
        arya_remove_delegation_tx(arya_info, delegate_info, certifier_2_info),
        // 8. verify delegation 1
        arya_verify_capabilities_tx(
            arya_info,
            delegate_info,
            certifier_1_info,
            target_info,
            "other_method",
        ),
        // 9. verify delegation 1 on `method1` that is forbidden. This shall fail
        arya_verify_capabilities_tx(
            arya_info,
            delegate_info,
            certifier_1_info,
            target_info,
            "method1",
        ),
    ]
}

fn check_rxs(rxs: Vec<Receipt>) {
    // 0. Crypto Hash to initializate crypto contract in account
    assert!(rxs[0].success);
    // 1. init arya.
    assert!(rxs[1].success);
    // 2. set profile data
    assert!(rxs[2].success);
    // 3. update profile data
    assert!(rxs[3].success);
    // 4. remove profile data
    assert!(rxs[4].success);
    // 5. set delegation 1
    assert!(rxs[5].success);
    // 6. set delegation 2
    assert!(rxs[6].success);
    // 7. remove delegation 2
    assert!(rxs[7].success);
    // 8. verify delegation 1
    assert!(rxs[8].success);
    // 9. verify delegation 1 on method1 that is forbidden. This shall fail
    assert!(!rxs[9].success);
}

#[test]
fn arya_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);
}
