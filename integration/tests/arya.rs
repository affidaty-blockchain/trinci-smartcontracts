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

use integration::{
    common::{self},
    TestApp,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_value::Value;
use std::collections::{BTreeMap, HashMap};
use trinci_core::crypto::ecdsa::KeyPair as EcdsaKeyPair;
use trinci_core::crypto::sign::KeyPair;
use trinci_core::{
    base::serialize::rmp_serialize,
    crypto::{sign::PublicKey, Hash},
    Receipt, Transaction,
};
use trinci_core::{crypto::ecdsa::CurveId, TransactionData};
use trinci_sdk::{rmp_serialize_named, value};

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
    keypair: &trinci_core::crypto::sign::KeyPair,
    // ecdsa_keypair: ecdsa::KeyPair,
    target: Hash,
    method: &str,
    args: impl Serialize,
) -> Transaction {
    //let keypair = trinci_core::crypto::sign::KeyPair::Ecdsa(ecdsa_keypair);
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

const CRYPTO_PKCS8: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a4020101043072903f41e56843227c479c012994be571d7bf0cf083cb154a369c4008a8709b04f625ee86f281a4b7e4f445b52aaf043a00706052b81040022a16403620004677ab1f2832bfec85ec98b96854db0091b02421d0871ac56f0ff2388e9aad26c486df10a286fda1b2a6955ec283e601580a39cbbad059b778f3af5d4a5fd6b36a2d16bf66fac5630c4778e0cbc3efe025fb492bcb8245e9781c880314fa8aed4";
const ARYA_PKCS8: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a402010104308054283d0eb1af7b1e8349e5f694208342762fcb0c9386cf1fa87f9985bb73a02cfeddccc26ed6fe15d8da019493537ba00706052b81040022a164036200041174cc62f48c0f105a54255291247617ab7983de0ae08c6a26afb28ce04cd76525fa48e75d7cef1e16e165c500e1aba6beb4c2fdbf2e8f8237c1075cb498c32e00d4ab4c37d8bf3ff4a359c8917519015bddf1038e0105fe928f74b31f4f9185";
const CERTIFIER_1_PKCS8: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a40201010430b0394ba13c3f916137a902f1b5e4ef6e1e2b2c8b7c3ef9a39eac68b6c52a3927b5ad0e0ff3ef00a0c551d3b954738e32a00706052b81040022a164036200048cfc38a74cfce0ed3f85d44fbefe14e20238049215573a59f4232098696679d616258041a176ba9f5188ad45c7a6b45b1352a87ac5f2987fc68b96fa63e432536c506dad03088888d70617152bea5107ba4ade8a69d030fd392000eb1a324038";
const CERTIFIER_2_PKCS8: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a40201010430caa9e7c1d5e9a6cd702e3f0c4e20adbf70abf318006c7c2369212b38915f466f5151fde7d8498ae031fd31b36f91c18da00706052b81040022a16403620004f426b3fcd49732a87ea836036d3ef68fb352e5d9c54699a8dd2baee44ccfcc017cfe9e8f4cca12037304d8026ae038124fd10cf339e49d4544dc4985ada714a357e2e199c0c84ba005d421f830018f9903bedf31b4653fd67ca53bdb683e63d9";
const USER_PKCS8: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a402010104308c14091733a6a3d64a0467a4b905775aa9844b03c7d7f7791f11879132445a49e748c6b2aa240dc97aca0c4ab90ee841a00706052b81040022a16403620004d482fa861888be6a0ad580a6e64c459e427de0791d93e472b68cd4b281ebf8c544eff24898124d6618757e494cb75864f0e584e57559fa16ccc75f4aa57c549d2b1a252cad8358ae5e4a73f86a84edc778b1d15dc07acf67f70f6cb2f69c062e";
const DELEGATE_PKCS8: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a40201010430faca9932e9679bbd45723963a88fcad2c39fbf2b53868b899e92bf6315cd725a505cd9aeb5fbbd8e85a99eb8c045bb4ca00706052b81040022a164036200041ef7e46cf1a09e0a761d27c85143ae268cc6fd91ae498629937cfe8b9a15b4c564de296305732017ba5509950d3480af40f00bcece98cc80004251abba818ce72807a94d7af5f385650ec7656b820b900df319aab230435fba3f31dba5a8811f";

const CERTIFICATE_1_HEX: &str ="9395d92e516d646e5452474b7147415165576643475a6770316532375a505a66436146617479787463394b764b6e5458783195a5656d61696ca46e616d65a3736578a77375726e616d65a374656cc4206ca9a15e675d9a989f19c885d3b21af08bebdd77a56f6479c064e53db7846ffcc420018bcb4f53a4861043e476b0ba2d47e9d67f30cb5d05e460307b0d46e059c36493a56563647361a9736563703338347231c461048cfc38a74cfce0ed3f85d44fbefe14e20238049215573a59f4232098696679d616258041a176ba9f5188ad45c7a6b45b1352a87ac5f2987fc68b96fa63e432536c506dad03088888d70617152bea5107ba4ade8a69d030fd392000eb1a324038c46017d23b9e96e8bf081e9b53d1bac649e18fa69ac9e9c9cddd7b24604b20e19c12cf6925effaac6a1536349c00ac5573bdda8ee0c7883f7a94654b0fb35fbf9acc5e07118174baff1b6f49fad05d22366bfae860d6443850507d894417a2221a8c93c4203101428eb3bc57f850f965df436adb2cfadaedb6df61f853d4c9b0c5c872d69ec420fda953ec7d8a64e50eb65890f92614df0673c867e997302e4e7a5864f42b1045c4206c7913d95c21aabfd9c74be95d166f47404b6904a22c5dbecbf0584540127247";
const CERTIFICATE_2_HEX: &str = "9395d92e516d646e5452474b7147415165576643475a6770316532375a505a66436146617479787463394b764b6e5458783195a5656d61696ca46e616d65a3736578a77375726e616d65a374656cc4206126a9c34da0188980f0e94f5e5ccdfe5878fab21614297ab310993bcfbcc263c42036affe4e269782db68bd4df0b8d9431398095276682d6f3780145bbff2d4f03193a56563647361a9736563703338347231c46104f426b3fcd49732a87ea836036d3ef68fb352e5d9c54699a8dd2baee44ccfcc017cfe9e8f4cca12037304d8026ae038124fd10cf339e49d4544dc4985ada714a357e2e199c0c84ba005d421f830018f9903bedf31b4653fd67ca53bdb683e63d9c46049557d652b57f72bb785d4bf1542be7bd3c1012e9e0dee8b4310662bbfd6e06a93e548c2d7fc7af3e46546bed8c108826b51d2f4bfb8cd281ac8987bcffd6d0e934f67ebc851de51bccde2c8f6ceb2c61d062ce761f4899e166e347ef973745593c4204f2915d2137bd6b2bea18bf4151fbc720b03ce5cd4c1b29d6d40e2af1cdfa708c420236c86ffe1a6295e0ab2c683ef76dbedaeeffaff2971c850493fa57e0599471fc420902dd04532ec09aba0a19824941c36e07dccd892a7f1460bea3f70e026073671";

lazy_static! {
    static ref KEYPAIRS_INFO: HashMap<&'static str, KeyPair> = {
        let mut map = HashMap::new();
        map.insert(
            ARYA_ALIAS,
            KeyPair::Ecdsa(
                EcdsaKeyPair::from_pkcs8_bytes(
                    CurveId::Secp384R1,
                    &hex::decode(ARYA_PKCS8).unwrap(),
                )
                .unwrap(),
            ),
        );
        map.insert(
            CRYPTO_ALIAS,
            KeyPair::Ecdsa(
                EcdsaKeyPair::from_pkcs8_bytes(
                    CurveId::Secp384R1,
                    &hex::decode(CRYPTO_PKCS8).unwrap(),
                )
                .unwrap(),
            ),
        );
        map.insert(
            CERTIFIER_1_ALIAS,
            KeyPair::Ecdsa(
                EcdsaKeyPair::from_pkcs8_bytes(
                    CurveId::Secp384R1,
                    &hex::decode(CERTIFIER_1_PKCS8).unwrap(),
                )
                .unwrap(),
            ),
        );
        map.insert(
            CERTIFIER_2_ALIAS,
            KeyPair::Ecdsa(
                EcdsaKeyPair::from_pkcs8_bytes(
                    CurveId::Secp384R1,
                    &hex::decode(CERTIFIER_2_PKCS8).unwrap(),
                )
                .unwrap(),
            ),
        );
        map.insert(
            USER_ALIAS,
            KeyPair::Ecdsa(
                EcdsaKeyPair::from_pkcs8_bytes(
                    CurveId::Secp384R1,
                    &hex::decode(USER_PKCS8).unwrap(),
                )
                .unwrap(),
            ),
        );
        map.insert(
            DELEGATE_ALIAS,
            KeyPair::Ecdsa(
                EcdsaKeyPair::from_pkcs8_bytes(
                    CurveId::Secp384R1,
                    &hex::decode(DELEGATE_PKCS8).unwrap(),
                )
                .unwrap(),
            ),
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
        user_info,
        *CRYPTO_APP_HASH,
        "hash",
        args,
    )
}

fn create_profile_data_bad() -> Value {
    value! ({
            "name": "John",
            "surname": "Dow",
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
        arya_info,
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
        target_account,
        *ARYA_APP_HASH,
        "set_profile_data",
        args,
    )
}

fn arya_remove_profile_data_tx(arya_info: &KeyPair, target_account: &KeyPair) -> Transaction {
    let args = value!(vec!["testfield"]);

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        target_account,
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

    let delegator = delegator_info.public_key();

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
        delegator_info,
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
        delegate_info,
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
        target_info,
        *ARYA_APP_HASH,
        "verify_capability",
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

fn arya_set_certificate_tx(
    arya_info: &KeyPair,
    target_account: &KeyPair,
    certificate_no: u8,
) -> Transaction {
    let (key, certificate) = match certificate_no {
        0 => ("main", hex::decode(CERTIFICATE_1_HEX).unwrap()),
        1 => ("one", hex::decode(CERTIFICATE_2_HEX).unwrap()),
        2 => ("two", hex::decode(CERTIFICATE_2_HEX).unwrap()),
        _ => ("none", vec![1u8, 2, 3]),
    };

    let args = SetCertArgs {
        key,
        certificate: &certificate,
    };

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        target_account,
        *ARYA_APP_HASH,
        "set_certificate",
        args,
    )
}

fn arya_remove_certificate_tx(
    arya_info: &KeyPair,
    certifier_account: &KeyPair,
    target_account: &KeyPair,
) -> Transaction {
    let args = value!({
        "target": target_account.public_key().to_account_id(),
        "certifier" : certifier_account.public_key().to_account_id(),
        "keys": vec!["*"]

    });

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        &target_account,
        *ARYA_APP_HASH,
        "remove_certificate",
        args,
    )
}

fn arya_verify_data_tx(
    arya_info: &KeyPair,
    certifier_account: &KeyPair,
    target_account: &KeyPair,
    good_data: bool,
) -> Transaction {
    let name = match good_data {
        true => "John",
        false => "Mike",
    };

    let args = value!({
        "target": target_account.public_key().to_account_id(),
        "certificate" : format!("{}:main", certifier_account.public_key().to_account_id()),
        "data": value!({
            "name": name,
            "surname": "Doe",
            "sex": "male"
        })
    });

    kp_create_test_tx(
        &arya_info.public_key().to_account_id(),
        &certifier_account,
        *ARYA_APP_HASH,
        "verify_data",
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
        crypto_hash_tx(&crypto_info, &target_info),
        // 1. init arya.
        arya_init_tx(arya_info, crypto_info),
        // 2. set profile data
        arya_set_profile_data_tx(arya_info, target_info, create_profile_data_bad()),
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
        // 10. set certificate `main` from Certifier 1
        arya_set_certificate_tx(arya_info, target_info, 0),
        // 11. set certificate `one` from Certifier 2
        arya_set_certificate_tx(arya_info, target_info, 1),
        // 12. set certificate `two` from Certifier 2
        arya_set_certificate_tx(arya_info, target_info, 2),
        // 13. remove all certificate from Certifier 2
        arya_remove_certificate_tx(arya_info, certifier_2_info, target_info),
        // 14. verify data
        arya_verify_data_tx(arya_info, certifier_1_info, target_info, true),
        // 15. verify bad data. This shall fail
        arya_verify_data_tx(arya_info, certifier_1_info, target_info, false),
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
    // 10. set certificate `main` from Certifier 1
    assert!(rxs[10].success);
    // 11. set certificate `one` from Certifier 2
    assert!(rxs[11].success);
    // 12. set certificate `two` from Certifier 2
    assert!(rxs[12].success);
    // 13. remove all certificate from Certifier 2
    assert!(rxs[13].success);
    // 14. verify data
    assert!(rxs[14].success);
    // 15. verify bad data. This shall fail
    assert!(!rxs[15].success);
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
