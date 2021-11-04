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
//!

use integration::{
    common::{
        self, AccountInfo, PUB_KEY1, PUB_KEY2, PUB_KEY3, PUB_KEY4, PUB_KEY5, PUB_KEY6, PVT_KEY1,
        PVT_KEY2, PVT_KEY3, PVT_KEY4, PVT_KEY5, PVT_KEY6,
    },
    TestApp,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_value::Value;
use trinci_sdk::value;

use std::collections::{BTreeMap, HashMap};
use trinci_core::crypto::ecdsa::KeyPair;
use trinci_core::{
    base::serialize::rmp_serialize,
    crypto::{
        ecdsa::{CurveId, PublicKey as EcdsaPublicKey},
        sign::PublicKey,
        Hash,
    },
    Receipt, Transaction,
};

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

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(ARYA_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(CRYPTO_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(CERTIFIER_1_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(CERTIFIER_2_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map.insert(USER_ALIAS, AccountInfo::new(PUB_KEY5, PVT_KEY5));
        map.insert(DELEGATE_ALIAS, AccountInfo::new(PUB_KEY6, PVT_KEY6));
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

fn arya_set_profile_data_tx(
    arya_info: &AccountInfo,
    target_account: &AccountInfo,
    args: Value,
) -> Transaction {
    common::create_test_tx(
        &arya_info.id,
        &target_account.pub_key,
        &target_account.pvt_key,
        *ARYA_APP_HASH,
        "set_profile_data",
        args,
    )
}

fn arya_remove_profile_data_tx(
    arya_info: &AccountInfo,
    target_account: &AccountInfo,
) -> Transaction {
    let args = value!(vec!["testfield"]);

    common::create_test_tx(
        &arya_info.id,
        &target_account.pub_key,
        &target_account.pvt_key,
        *ARYA_APP_HASH,
        "remove_profile_data",
        args,
    )
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct SetCertArgs<'a> {
    pub key: &'a str,
    #[serde(with = "serde_bytes")]
    pub certificate: &'a [u8],
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct Certificate<'a> {
    data: CertificateData<'a>,
    #[serde(with = "serde_bytes")]
    pub signature: &'a [u8],
    // #[serde(with = "serde_bytes")]
    pub multiproof: Vec<&'a [u8]>,
}
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct CertificateData<'a> {
    target: &'a str,
    fields: Vec<&'a str>,
    #[serde(with = "serde_bytes")]
    pub salt: &'a [u8],
    #[serde(with = "serde_bytes")]
    pub root: &'a [u8],
    certifier: PublicKey,
}

fn arya_set_certificate_tx(
    arya_info: &AccountInfo,
    certifier_kp: &KeyPair,
    target_account: &AccountInfo,
) -> Transaction {
    let certifier = PublicKey::Ecdsa(EcdsaPublicKey {
        curve_id: CurveId::Secp384R1,
        value: certifier_kp.public_key().value,
    });

    let certificated_data = CertificateData {
        target: &target_account.id,
        fields: vec!["name", "surname", "sex", "tel", "email"],
        salt: &[
            205u8, 105, 241, 125, 32, 198, 247, 56, 76, 220, 110, 242, 251, 48, 145, 9, 15, 211,
            208, 205, 200, 42, 32, 83, 87, 7, 234, 109, 58, 193, 224, 239,
        ],
        root: &[
            238, 42, 127, 230, 79, 120, 133, 124, 200, 183, 0, 166, 120, 83, 197, 100, 101, 228,
            65, 44, 69, 160, 237, 78, 139, 20, 67, 252, 243, 224, 98, 3,
        ],
        certifier,
    };

    let certified_data_buf = rmp_serialize(&certificated_data).unwrap();
    let sign = certifier_kp.sign(&certified_data_buf).unwrap();

    let certificate = Certificate {
        data: certificated_data,
        signature: &sign,
        multiproof: vec![],
        // multiproof: vec![&vec![], &vec![]],
    };

    let certificate = rmp_serialize(&certificate).unwrap();

    let args = SetCertArgs {
        key: "main",
        certificate: &certificate,
    };

    common::create_test_tx(
        &arya_info.id,
        &target_account.pub_key,
        &target_account.pvt_key,
        *ARYA_APP_HASH,
        "set_certificate",
        args,
    )
}

fn arya_remove_certificate_tx(
    arya_info: &AccountInfo,
    certifier_account: &AccountInfo,
    target_account: &AccountInfo,
) -> Transaction {
    let args = value!({
        "target": target_account.id,
        "certifier" : certifier_account.id,
        "keys": vec!["*"]

    });

    common::create_test_tx(
        &arya_info.id,
        &target_account.pub_key,
        &target_account.pvt_key,
        *ARYA_APP_HASH,
        "remove_certificate",
        args,
    )
}

fn arya_verify_data_tx(
    arya_info: &AccountInfo,
    certifier_account: &AccountInfo,
    target_account: &AccountInfo,
) -> Transaction {
    let args = value!({
        "target": target_account.id,
        "certificate" : format!("{}:main", certifier_account.id),
        "data": create_profile_data()
    });

    common::create_test_tx(
        &arya_info.id,
        &target_account.pub_key,
        &target_account.pvt_key,
        *ARYA_APP_HASH,
        "verify_data",
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
    delegator_kp: KeyPair,
    target_account: &AccountInfo,
) -> Transaction {
    let mut capabilities = BTreeMap::<&str, bool>::new();
    capabilities.insert("*", true);
    capabilities.insert("method1", false);

    let delegator = PublicKey::Ecdsa(EcdsaPublicKey {
        curve_id: CurveId::Secp384R1,
        value: delegator_kp.public_key().value,
    });

    let data = DelegationData {
        delegate: &delegate_info.id,
        delegator: delegator.clone(),
        network: "skynet",
        target: &target_account.id,
        expiration: 123u64,
        capabilities: capabilities.clone(),
    };

    let data_to_sign = rmp_serialize(&data).unwrap();

    let sign = delegator_kp.sign(&data_to_sign).unwrap();

    let delegation = Delegation {
        data,
        signature: &sign,
    };

    let delegation = rmp_serialize(&delegation).unwrap();

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

fn arya_remove_delegation_tx(
    arya_info: &AccountInfo,
    delegate_info: &AccountInfo,
    delegator_info: &AccountInfo,
) -> Transaction {
    let args = value!({ "delegate": delegate_info.id,
        "delegator": delegator_info.id,
        "targets": vec!["*"],
    });

    common::create_test_tx(
        &arya_info.id,
        &delegate_info.pub_key,
        &delegate_info.pvt_key,
        *ARYA_APP_HASH,
        "remove_delegation",
        args,
    )
}

fn arya_verify_capabilities_tx(
    arya_info: &AccountInfo,
    delegate_info: &AccountInfo,
    delegator_info: &AccountInfo,
    target_info: &AccountInfo,
    method: &str,
) -> Transaction {
    let args = value!({
        "delegate": delegate_info.id,
        "delegator": delegator_info.id,
        "target": target_info.id,
        "method": method,
    });

    common::create_test_tx(
        &arya_info.id,
        &target_info.pub_key,
        &target_info.pvt_key,
        *ARYA_APP_HASH,
        "verify_capability",
        args,
    )
}

fn create_txs() -> Vec<Transaction> {
    let arya_info = ACCOUNTS_INFO.get(ARYA_ALIAS).unwrap();
    let crypto_info = ACCOUNTS_INFO.get(CRYPTO_ALIAS).unwrap();
    let target_info = ACCOUNTS_INFO.get(USER_ALIAS).unwrap();
    let certifier_1_info = ACCOUNTS_INFO.get(CERTIFIER_1_ALIAS).unwrap();
    let certifier_2_info = ACCOUNTS_INFO.get(CERTIFIER_2_ALIAS).unwrap();
    let delegate_info = ACCOUNTS_INFO.get(DELEGATE_ALIAS).unwrap();

    let certifier_1_kp = KeyPair::new(
        CurveId::Secp384R1,
        &hex::decode(&PVT_KEY3).unwrap(),
        &hex::decode(&PUB_KEY3).unwrap(),
    )
    .unwrap();
    let certifier_2_kp = KeyPair::new(
        CurveId::Secp384R1,
        &hex::decode(&PVT_KEY4).unwrap(),
        &hex::decode(&PUB_KEY4).unwrap(),
    )
    .unwrap();

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
        // 5. set certificate // FIXME
        arya_set_certificate_tx(arya_info, &certifier_1_kp, target_info),
        // 6. remove certificate // FIXME
        arya_remove_certificate_tx(arya_info, certifier_1_info, target_info),
        // 7. set certificate from certifier 2 // FIXME
        arya_set_certificate_tx(arya_info, &certifier_2_kp, target_info),
        // 8. update certificate. This really does nothing  // FIXME
        arya_set_certificate_tx(arya_info, &certifier_2_kp, target_info),
        // 9. verify data // FIXME
        arya_verify_data_tx(arya_info, certifier_2_info, target_info),
        // 10. set delegation 1
        arya_set_delegation_tx(
            arya_info,
            delegate_info,
            certifier_1_info,
            certifier_1_kp,
            target_info,
        ),
        // 11. set delegation 2
        arya_set_delegation_tx(
            arya_info,
            delegate_info,
            certifier_2_info,
            certifier_2_kp,
            target_info,
        ),
        // 12. remove delegation 2
        arya_remove_delegation_tx(arya_info, delegate_info, certifier_2_info),
        // 13. verify delegation 1
        arya_verify_capabilities_tx(
            arya_info,
            delegate_info,
            certifier_1_info,
            target_info,
            "other_method",
        ),
        // 14. verify delegation 1 on `method1` that is forbidden. This shall fail
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

    // 5. set certificate
    assert!(rxs[5].success);

    // 6. remove certificate
    assert!(rxs[6].success);

    // 7. set certificate from certifier 2
    assert!(rxs[7].success);
    // 8. update certificate
    assert!(rxs[8].success);
    // 9. verify data
    println!("{}", String::from_utf8_lossy(&rxs[9].returns));
    assert!(rxs[9].success);
    // 10. set delegation 1
    assert!(rxs[10].success);
    // 11. set delegation 2
    assert!(rxs[11].success);
    // 12. remove delegation 2
    assert!(rxs[12].success);
    // 13. verify delegation 1
    assert!(rxs[13].success);
    // 14. verify delegation 1 on method1 that is forbidden. This shall fail
    assert!(!rxs[14].success);
}

#[test]
fn arya_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);

    // Checks
    // let asset_info = ACCOUNTS_INFO.get(ASSET_ALIAS).unwrap();
    // let escrow_info = ACCOUNTS_INFO.get(ESCROW_ALIAS).unwrap();
    // let customer_info = ACCOUNTS_INFO.get(CUSTOMER_ALIAS).unwrap();

    // let account = app.account(&escrow_info.id).unwrap();
    // let asset: Asset = serialize::rmp_deserialize(&account.load_asset(&asset_info.id)).unwrap();
    // assert_eq!(asset.units, 0);
}
