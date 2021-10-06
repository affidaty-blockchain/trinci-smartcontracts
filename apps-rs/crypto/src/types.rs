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

#[cfg(test)]
pub(crate) mod tests {
    use trinci_sdk::ecdsa::{CurveId, PublicKey};

    use super::*;

    const HASH_ARGS_HEX: &str = "928100c0c403010203";
    const VERIFY_ARGS_HEX: &str = "9393a56563647361a9736563703338347231c403010203c403040506c403060708";

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
}
