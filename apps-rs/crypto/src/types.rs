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

use trinci_sdk::core::PublicKey;

use serde_derive::{Deserialize, Serialize};

// Hash Algorithms available
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::Sha256
    }
}

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

// Hash arguments
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct HashArgs<'a> {
    pub algorithm: HashAlgorithm,
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
}

// Verify arguments
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct MerkleTreeVerifyArgs<'a> {
    pub root: &'a str,
    pub indices: Vec<i32>,
    pub leaves: Vec<&'a str>,
    pub depth: i32,
    pub proofs: Vec<&'a str>,
}

#[cfg(test)]
pub(crate) mod tests {
    use trinci_sdk::ecdsa::{CurveId, PublicKey};

    use super::*;

    const HASH_ARGS_HEX: &str = "928100c0c403010203";
    const VERIFY_ARGS_HEX: &str =
        "9393a56563647361a9736563703338347231c403010203c403040506c403060708";
    const MERKLE_TREE_VERIFY_ARGS_HEX: &str = "95d9406634643764333538363766623136663033393961363030343661306439663933613131316164633530616236633039633537633438383565306362613235613792010392d94034383738613164356633336431383534386662333565313633323361323263623363663831663630303763316339383130666664623339633032626136333430d940346237366262663036633332616234646334356331336362316335306139386439346438666465653661363238346238663532333032613539396463616630650393d94066316166663461333430333432623037343831636238373432336131363763363935353032323636383831356630383031643462653337643865353033393766d94033386334616432636230623663666662613362383432646163346261326537346536363835613635656439343630643234626561386663366161316463616634d94066393633373935316631306264396533353534313962316638346664396566626130633861626366356664386538333831653436666538353634383265653665";

    pub fn create_merkle_tree_verify_args() -> MerkleTreeVerifyArgs<'static> {
        let root = "f4d7d35867fb16f0399a60046a0d9f93a111adc50ab6c09c57c4885e0cba25a7";
        let indices = vec![1, 3];
        let leaves = vec![
            "4878a1d5f33d18548fb35e16323a22cb3cf81f6007c1c9810ffdb39c02ba6340",
            "4b76bbf06c32ab4dc45c13cb1c50a98d94d8fdee6a6284b8f52302a599dcaf0e",
        ];
        let depth = 3;

        let proofs = vec![
            "f1aff4a340342b07481cb87423a167c6955022668815f0801d4be37d8e50397f",
            "38c4ad2cb0b6cffba3b842dac4ba2e74e6685a65ed9460d24bea8fc6aa1dcaf4",
            "f9637951f10bd9e355419b1f84fd9efba0c8abcf5fd8e8381e46fe856482ee6e",
        ];

        MerkleTreeVerifyArgs {
            root,
            indices,
            leaves,
            depth,
            proofs,
        }
    }

    #[test]
    fn hash_args_serialize() {
        let args = HashArgs {
            algorithm: HashAlgorithm::Sha256,
            data: &[1u8, 2, 3],
        };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), HASH_ARGS_HEX);
    }

    #[test]
    fn hash_args_deserialize() {
        let expected = HashArgs {
            algorithm: HashAlgorithm::Sha256,
            data: &[1u8, 2, 3],
        };

        let buf = hex::decode(HASH_ARGS_HEX).unwrap();

        let args: HashArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn verify_args_serialize() {
        let pk = PublicKey {
            curve: CurveId::Secp384R1,
            value: vec![1, 2, 3],
        };

        let args = VerifyArgs {
            pk: trinci_sdk::core::PublicKey::Ecdsa(pk),
            data: &[4, 5, 6],
            sign: &[6, 7, 8],
        };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), VERIFY_ARGS_HEX);
    }

    #[test]
    fn verify_args_deserialize() {
        let pk = PublicKey {
            curve: CurveId::Secp384R1,
            value: vec![1, 2, 3],
        };

        let expected = VerifyArgs {
            pk: trinci_sdk::core::PublicKey::Ecdsa(pk),
            data: &[4, 5, 6],
            sign: &[6, 7, 8],
        };

        let buf = hex::decode(VERIFY_ARGS_HEX).unwrap();

        let args: VerifyArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn merkle_tree_verify_args_serialize() {
        let args = create_merkle_tree_verify_args();

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), MERKLE_TREE_VERIFY_ARGS_HEX);
    }

    #[test]
    fn merkle_tree_verify_args_deserialize() {
        let expected = create_merkle_tree_verify_args();

        let buf = hex::decode(MERKLE_TREE_VERIFY_ARGS_HEX).unwrap();

        let args: MerkleTreeVerifyArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }
}
