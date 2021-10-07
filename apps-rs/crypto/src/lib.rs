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

//! Crypto
//! Crypto Contract Library with methods callable from other contracts
//!
//! ### Methods
//!
//!  - hash                 SHA256, SHA384, SHA512
//!  - verify               Ecdsa_P384
//!  - merkle_tree_verify   Verify multiproof merkle tree leaves

use sha2::{Digest, Sha256, Sha384, Sha512};
use trinci_sdk::{AppContext, WasmResult};

mod merkle_tree;
mod types;

use merkle_tree::verify_merkle_tree_multiproof;
use types::*;

trinci_sdk::app_export!(verify, hash, merkle_tree_verify);

/// Verify multiproof merkle tree
fn merkle_tree_verify(_ctx: AppContext, args: MerkleTreeVerifyArgs) -> WasmResult<()> {
    verify_merkle_tree_multiproof(
        args.root,
        &args.indices,
        &args.leaves,
        args.depth,
        &args.proofs,
    )
}

/// Calculate hash.
fn hash(_ctx: AppContext, args: HashArgs) -> WasmResult<Vec<u8>> {
    // let mut hasher;
    let hash = match args.algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(args.data);
            hasher.finalize().as_slice().to_vec()
        }
        HashAlgorithm::Sha384 => {
            let mut hasher = Sha384::new();
            hasher.update(args.data);
            hasher.finalize().as_slice().to_vec()
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(args.data);
            hasher.finalize().as_slice().to_vec()
        }
    };

    Ok(hash)
}

/// Transaction data signature verification.
pub fn verify(_ctx: AppContext, args: VerifyArgs) -> WasmResult<bool> {
    Ok(trinci_sdk::verify(&args.pk, args.data, args.sign))
}

#[cfg(test)]
mod tests {

    use crate::types::tests::create_merkle_tree_verify_args;

    use super::*;
    use trinci_sdk::ecdsa::{CurveId, PublicKey};
    use trinci_sdk::not_wasm;

    const SHA256_HASH_HEX: &str =
        "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81";
    const SHA384_HASH_HEX: &str = "86229dc6d2ffbeac7380744154aa700291c064352a0dbdc77b9ed3f2c8e1dac4dc325867d39ddff1d2629b7a393d47f6";
    const CALLER_ID: &str = "QmT48ijWd7RqEzdV3gKjqXN1kGBgYxFWsxajjguLkyTjy7";

    const PUBLIC_KEY: &str = "045936d631b849bb5760bcf62e0d1261b6b6e227dc0a3892cbeec91be069aaa25996f276b271c2c53cba4be96d67edcadd66b793456290609102d5401f413cd1b5f4130b9cfaa68d30d0d25c3704cb72734cd32064365ff7042f5a3eee09b06cc1";

    #[test]
    fn test_verify_ecdsa_p384() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);

        let public_key = hex::decode(PUBLIC_KEY).unwrap();

        let pk = PublicKey {
            curve: CurveId::Secp384R1,
            value: public_key,
        };

        let args = VerifyArgs {
            pk: trinci_sdk::core::PublicKey::Ecdsa(pk),
            data: &[1, 2, 3],
            sign: &[1, 2, 3], // The first byte to one make hf_verify returns true
        };

        let res = not_wasm::call_wrap(verify, ctx, args).unwrap();

        assert!(res);
    }

    #[test]
    fn test_verify_ecdsa_p384_bad_sign() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);

        let public_key = hex::decode(PUBLIC_KEY).unwrap();

        let pk = PublicKey {
            curve: CurveId::Secp384R1,
            value: public_key,
        };

        let args = VerifyArgs {
            pk: trinci_sdk::core::PublicKey::Ecdsa(pk),
            data: &[1, 2, 3],
            sign: &[0, 1, 2], // The first byte to zero make hf_verify fails
        };

        let res = not_wasm::call_wrap(verify, ctx, args).unwrap();

        assert!(!res);
    }

    #[test]
    fn test_hash_sha256() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        let args = HashArgs {
            algorithm: HashAlgorithm::Sha256,
            data: &[1u8, 2, 3],
        };

        let res = not_wasm::call_wrap(hash, ctx, args.clone()).unwrap();

        assert_eq!(res, hex::decode(SHA256_HASH_HEX).unwrap());
    }

    #[test]
    fn test_hash_sha384() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);
        let args = HashArgs {
            algorithm: HashAlgorithm::Sha384,
            data: &[1u8, 2, 3],
        };

        let res = not_wasm::call_wrap(hash, ctx, args.clone()).unwrap();

        assert_eq!(res, hex::decode(SHA384_HASH_HEX).unwrap());
    }

    #[test]
    fn test_merkle_tree_verify() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);

        let args = create_merkle_tree_verify_args();

        not_wasm::call_wrap(merkle_tree_verify, ctx, args.clone()).unwrap();
    }

    #[test]
    fn test_merkle_tree_verify_not_valid_leaf() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);

        let mut args = create_merkle_tree_verify_args();
        args.leaves[0] = "BAD8a1d5f33d18548fb35e16323a22cb3cf81f6007c1c9810ffdb39c02ba6340"; // <-- Not valid

        let err = not_wasm::call_wrap(merkle_tree_verify, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "invalid leaves")
    }

    #[test]
    fn test_merkle_tree_verify_bad_hex() {
        let ctx = not_wasm::create_app_context(CALLER_ID, CALLER_ID);

        let mut args = create_merkle_tree_verify_args();
        args.root = "BAD_d35867fb16f0399a60046a0d9f93a111adc50ab6c09c57c4885e0cba25a7"; // <- Bad HEX

        let err = not_wasm::call_wrap(merkle_tree_verify, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "invalid `root` hex");
    }
}
