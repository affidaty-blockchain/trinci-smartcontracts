//! Crypto
//! Crypto Contract Library with methods callable from other contracts
//!
//! ### Methods
//!
//!  - Hash     SHA256, SHA384, SHA512
//!  - Verify   Ecdsa_P384

use sha2::{Digest, Sha256, Sha384, Sha512};

use trinci_sdk::{AppContext, WasmResult};

mod types;

use types::*;

trinci_sdk::app_export!(verify, hash);

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
}
