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

//! Crypto integration tests
use integration::{
    common::{self, AccountInfo, PUB_KEY1, PUB_KEY2, PVT_KEY1, PVT_KEY2},
    TestApp,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use trinci_core::{
    base::serialize::rmp_deserialize,
    crypto::{
        ecdsa::{CurveId, PublicKey as EcdsaPublicKey},
        ed25519::{KeyPair, PublicKey as Ed25519PublicKey},
        Hash,
    },
    PublicKey, Receipt, Transaction,
};

// Verify Algorithms available
// Verify arguments
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct VerifyArgs<'a> {
    pub pk: PublicKey,
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
    #[serde(with = "serde_bytes")]
    pub sign: &'a [u8],
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

lazy_static! {
    pub static ref CRYPTO_APP_HASH: Hash = common::app_hash("crypto.wasm").unwrap();
}

const OWNER_ALIAS: &str = "Owner";
const USER_ALIAS: &str = "User";

const HASH_HEX: &str = "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81"; // [1,2,3] sha256
const ECDSA_PK_HEX: &str = "045936d631b849bb5760bcf62e0d1261b6b6e227dc0a3892cbeec91be069aaa25996f276b271c2c53cba4be96d67edcadd66b793456290609102d5401f413cd1b5f4130b9cfaa68d30d0d25c3704cb72734cd32064365ff7042f5a3eee09b06cc1";
const DATA_HEX : &str = "93d92e516d59486e45514c64663568374b59626a4650754853526b325350676458724a5746683557363936485066713769d92e516d51483939796471723743693148736a35456235446e62523168445a5246705272515665515a486b69627545701e";
const ECDSA_SIGN_HEX: &str = "88c4ba6d4ce8661787104408d42e8c59c7ed3efd548732c670d69e47e15bbe59dd915c6c1f7e532a112af7b6e2189ab0f22efdf6846048ec2cdb056ce1085cc2d5cdee157c6f70d930962958bd27089b54c1dcbf7f4c85d3df579e69cd63a395";
const ED25519_BYTES_HEX: &str = "5fe6fc0f9274651d278798a4d86d9395ffdf4eff7361876f72201a130befb2c9587b8d516e9605a6ee57a19e2734f1ab3bb8b45e6062801dff3e6408d8594063";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(OWNER_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(USER_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map
    };
}

fn hash_tx(owner_info: &AccountInfo, user_info: &AccountInfo) -> Transaction {
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

pub fn ed25519_test_keypair() -> KeyPair {
    let bytes = hex::decode(ED25519_BYTES_HEX).unwrap();
    KeyPair::from_bytes(&bytes).unwrap()
}

pub fn ed25519_test_public_key() -> Ed25519PublicKey {
    ed25519_test_keypair().public_key()
}

fn verify_ecdsa_tx(owner_info: &AccountInfo, user_info: &AccountInfo, valid: bool) -> Transaction {
    let pk = PublicKey::Ecdsa(EcdsaPublicKey {
        curve: CurveId::Secp384R1,
        value: hex::decode(&ECDSA_PK_HEX).unwrap(),
    });

    let mut args = VerifyArgs {
        pk,
        data: &hex::decode(&DATA_HEX).unwrap(),
        sign: &hex::decode(&ECDSA_SIGN_HEX).unwrap(),
    };

    if !valid {
        args.sign = &[5, 6, 7];
    }

    common::create_test_tx(
        &owner_info.id,
        &user_info.pub_key,
        &user_info.pvt_key,
        *CRYPTO_APP_HASH,
        "verify",
        args,
    )
}

fn verify_ed25519_tx(
    owner_info: &AccountInfo,
    user_info: &AccountInfo,
    valid: bool,
) -> Transaction {
    let kp = ed25519_test_keypair();
    let pk: PublicKey = PublicKey::Ed25519 {
        pb: kp.public_key(),
    };

    let data = &hex::decode(&DATA_HEX).unwrap();

    let sign = &kp.sign(data).unwrap();

    let mut args = VerifyArgs { pk, data, sign };

    if !valid {
        args.sign = &[5, 6, 7];
    }

    common::create_test_tx(
        &owner_info.id,
        &user_info.pub_key,
        &user_info.pvt_key,
        *CRYPTO_APP_HASH,
        "verify",
        args,
    )
}

fn create_txs() -> Vec<Transaction> {
    let owner_info = ACCOUNTS_INFO.get(OWNER_ALIAS).unwrap();
    let user_info = ACCOUNTS_INFO.get(USER_ALIAS).unwrap();

    vec![
        // 0. Calculate hash.
        hash_tx(owner_info, user_info),
        // 1. Verify ECDSA signature.
        verify_ecdsa_tx(owner_info, user_info, true),
        // 2. Verify ECDSA bad signature. This shall fail
        verify_ecdsa_tx(owner_info, user_info, false),
        // 3. Verify ED25519 signature.
        verify_ed25519_tx(owner_info, user_info, true),
        // 3. Verify ED25519 bad signature. This shall fail.
        verify_ed25519_tx(owner_info, user_info, false),
    ]
}

fn check_rxs(rxs: Vec<Receipt>) {
    // 0. Calculate hash.
    assert!(rxs[0].success);
    let res: Vec<u8> = rmp_deserialize(&rxs[0].returns).unwrap();
    assert_eq!(HASH_HEX, hex::encode(&res));
    // 1. Verify ECDSA signature.
    assert!(rxs[1].success);
    let res: bool = rmp_deserialize(&rxs[1].returns).unwrap();
    assert_eq!(res, true);
    // 2. Verify ECDSA bad signature. This shall fail.
    assert!(rxs[2].success);
    let res: bool = rmp_deserialize(&rxs[2].returns).unwrap();
    assert_eq!(res, false);
    // 3. Verify ED25519 signature.
    assert!(rxs[1].success);
    let res: bool = rmp_deserialize(&rxs[1].returns).unwrap();
    assert_eq!(res, true);
    // 2. Verify ED25519 bad signature. This shall fail.
    assert!(rxs[2].success);
    let res: bool = rmp_deserialize(&rxs[2].returns).unwrap();
    assert_eq!(res, false);
}

#[test]
fn crypto_test() {
    // Instance the application.
    let mut app = TestApp::default();

    // Create and execute transactions.
    let txs = create_txs();
    let rxs = app.exec_txs(txs);
    check_rxs(rxs);
}
