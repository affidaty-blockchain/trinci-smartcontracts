use serde::{Deserialize, Serialize};

// NFT Config Data
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct NFTConfig<'a> {
    pub creator: &'a str,
    pub owner: &'a str,
    #[serde(with = "serde_bytes")]
    pub identifier: &'a [u8],
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
    pub sellable: bool,
    pub asset: &'a str,
    pub price: u64,
    pub minimum_price: u64,
    pub creator_fee: u16, // thousandth: 5 => 5/1000*100 => 0.5%
    pub intermediary: &'a str,
    pub intermediary_fee: u16, // thousandth: 5 => 5/1000*100 => 0.5%
}

/// NFT Initialization arguments.
pub type NFTInitArgs<'a> = NFTConfig<'a>;

// NFT set_sellable args
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct SetSellableArgs {
    pub sellable: bool,
}

// NFT set_price args
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct SetPriceArgs {
    pub price: u64,
}

// NFT set_intermediary args
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct SetIntermediaryArgs<'a> {
    pub intermediary: &'a str,
    pub intermediary_fee: u16, // thousandth: 5 => 5/1000*100 => 0.5%
}

// NFT buy args
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct BuyArgs {
    pub sellable: bool,
    pub new_price: u64,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use trinci_sdk::{rmp_deserialize, rmp_serialize};

    pub const NFT_ID: &str = "QmNFTDQTpccaEETHWMqXPHcrVRUqaD9JxRHLdSVsCFgxj5";
    pub const CREATOR_ID: &str = "QmXEuDQTpccaEETHWMqXPHcrVRUqaD9JxRHLdSVsCFgxj5";
    pub const OWNER_1_ID: &str = "QmSCRCPFznxEX6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq";
    pub const ASSET_ID: &str = "QmQH99ydqr7Ci1Hsj5Eb5DnbR1hDZRFpRrQVeQZHkibuEp";
    pub const UNKNOWN_ID: &str = "QmUNKNOWNi1Hsj5Eb5ertDnbR1hDZRFpRrQVeQZHkibuEp";
    pub const INTERMEDIARY_ID: &str = "QmINTERMEDIARYi1Hsj5Eb5ertDnbR1hDZRFpRrQVeQZEp";

    const NFT_CONFIG_HEX: &str = "9bd92e516d5845754451547063636145455448574d715850486372565255716144394a7852484c64535673434667786a35d92e516d5343524350467a6e78455836533331364d3479566d786478504236584e36336f62324c6a46596b50364d4c71c403010203c400c2d92e516d51483939796471723743693148736a35456235446e62523168445a5246705272515665515a486b6962754570cdc350cdc35023a000";
    const SELLABLE_ARGS_HEX: &str = "91c3";
    const PRICE_ARGS_HEX: &str = "912a";
    const INTERMEDIARY_ARGS_HEX: &str = "92b14e6577496e7465726d6564696172794964cd01a4";
    const BUY_ARGS_HEX: &str = "92c3cc96";

    pub fn create_nft_init_config_data() -> NFTConfig<'static> {
        NFTConfig {
            creator: CREATOR_ID,
            owner: OWNER_1_ID,
            identifier: &[1, 2, 3],
            data: &[],
            sellable: false,
            asset: ASSET_ID,
            price: 50000,
            minimum_price: 50000,
            creator_fee: 35,
            intermediary: "",
            intermediary_fee: 0,
        }
    }

    #[test]
    fn nft_config_serialize() {
        let data = create_nft_init_config_data();
        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), NFT_CONFIG_HEX);
    }
    #[test]
    fn nft_config_deserialize() {
        let expected = create_nft_init_config_data();
        let buf = hex::decode(NFT_CONFIG_HEX).unwrap();

        let data: NFTConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn nft_set_sellable_args_serialize() {
        let data = SetSellableArgs { sellable: true };
        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), SELLABLE_ARGS_HEX);
    }

    #[test]
    fn nft_set_sellable_args_deserialize() {
        let expected = SetSellableArgs { sellable: true };
        let buf = hex::decode(SELLABLE_ARGS_HEX).unwrap();

        let data: SetSellableArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn nft_set_price_args_serialize() {
        let data = SetPriceArgs { price: 42 };
        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), PRICE_ARGS_HEX);
    }

    #[test]
    fn nft_set_price_args_deserialize() {
        let expected = SetPriceArgs { price: 42 };
        let buf = hex::decode(PRICE_ARGS_HEX).unwrap();

        let data: SetPriceArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn nft_set_intermediary_args_serialize() {
        let data = SetIntermediaryArgs {
            intermediary: "NewIntermediaryId",
            intermediary_fee: 420,
        };
        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), INTERMEDIARY_ARGS_HEX);
    }

    #[test]
    fn nft_set_intermediary_args_deserialize() {
        let expected = SetIntermediaryArgs {
            intermediary: "NewIntermediaryId",
            intermediary_fee: 420,
        };
        let buf = hex::decode(INTERMEDIARY_ARGS_HEX).unwrap();

        let data: SetIntermediaryArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn nft_buy_args_serialize() {
        let data = BuyArgs {
            sellable: true,
            new_price: 150,
        };
        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), BUY_ARGS_HEX);
    }

    #[test]
    fn nft_buy_args_deserialize() {
        let expected = BuyArgs {
            sellable: true,
            new_price: 150,
        };
        let buf = hex::decode(BUY_ARGS_HEX).unwrap();

        let data: BuyArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }
}
