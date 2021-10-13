use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Display},
};

/// Dynamic Exchange Apply arguments
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct ApplyArgs<'a> {
    pub asset: &'a str,
    pub amount: u64,
}

/// Dynamic Exchange resolution status.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum DynamicExchangeStatus {
    /// Open and ready to be "applied" from a buyer.
    Open,
    /// Exchange fully emptied.
    Exhausted,
    /// Exchange aborted by the seller.
    Aborted,
}

impl Display for DynamicExchangeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            DynamicExchangeStatus::Open => "open",
            DynamicExchangeStatus::Exhausted => "exhausted",
            DynamicExchangeStatus::Aborted => "aborted",
        };
        write!(f, "{}", msg)
    }
}

/// Dynamic Exchange Configuration.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct DynamicExchangeConfig<'a> {
    pub seller: &'a str,
    pub asset: &'a str,
    pub guarantor: &'a str,
    pub guarantor_fee: u64,
    pub penalty_fee: u64, // This is a percentage if the penalty_asset is the same as the source asset, is an amount instead
    pub penalty_asset: &'a str, // Currently is the same of the src asset
    pub assets: BTreeMap<&'a str, u64>,
}

/// Dynamic Exchange information returned by `get_info` method
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct DynamicExchangeInfo<'a> {
    pub config: DynamicExchangeConfig<'a>,
    pub amount: u64,
    pub status: &'a str,
}

/// Initialization arguments.
pub type DynamicExchangeInitArgs<'a> = DynamicExchangeConfig<'a>;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use trinci_sdk::{rmp_deserialize, rmp_serialize};

    pub const DYNAMIC_EXCHANGE_ID: &str = "QmExchange_qEzdV3gKjqXN1kGBgYxFWsxajjguLkyTjy7";
    pub const ASSET_ID: &str = "QmAsset_pccaEETHWMqXPHcrVRUqaD9JxRHLdSVsCFgxj5";
    pub const PENALTY_ASSET_ID: &str = "QmPenalty_Asset_WMqXPHcrVRUqaD9JxRHLdSVsCFgxj5";
    pub const GUARANTOR_ID: &str = "QmGuarantor_X6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq";
    pub const BUYER_ID: &str = "QmBuyer_znxEX6S316M4yVmxdxPB6XN63ob2LjFYkP6MXX";
    pub const SELLER_ID: &str = "QmSeller_f5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";
    pub const ASSET1_ID: &str = "QmAsset_1_7Ci1Hsj5Eb5DnbR1hDZRFpRrQVeQZHkibuEp";
    pub const ASSET2_ID: &str = "QmAsset_2_RjBSxPYHGVGtZmmkN3fVtHJTuTbwwUSdnB8a";

    const DYNAMIC_EXCHANGE_CONFIG_HEX: &str = "97d92e516d53656c6c65725f663568374b59626a4650754853526b325350676458724a5746683557363936485066713769d92e516d41737365745f7063636145455448574d715850486372565255716144394a7852484c64535673434667786a35d92e516d47756172616e746f725f5836533331364d3479566d786478504236584e36336f62324c6a46596b50364d4c713264d92e516d41737365745f7063636145455448574d715850486372565255716144394a7852484c64535673434667786a3582d92e516d41737365745f315f3743693148736a35456235446e62523168445a5246705272515665515a486b6962754570ccfad92e516d41737365745f325f526a425378505948475647745a6d6d6b4e33665674484a5475546277775553646e423861cd0136";
    const DYNAMIC_EXCHANGE_APPLY_ARGS_HEX: &str = "92d92e516d41737365745f7063636145455448574d715850486372565255716144394a7852484c64535673434667786a352a";
    const DYNAMIC_EXCHANGE_INFO_HEX: &str = "9397d92e516d53656c6c65725f663568374b59626a4650754853526b325350676458724a5746683557363936485066713769d92e516d41737365745f7063636145455448574d715850486372565255716144394a7852484c64535673434667786a35d92e516d47756172616e746f725f5836533331364d3479566d786478504236584e36336f62324c6a46596b50364d4c713264d92e516d41737365745f7063636145455448574d715850486372565255716144394a7852484c64535673434667786a3582d92e516d41737365745f315f3743693148736a35456235446e62523168445a5246705272515665515a486b6962754570ccfad92e516d41737365745f325f526a425378505948475647745a6d6d6b4e33665674484a5475546277775553646e423861cd013664a46f70656e";

    pub fn create_dynamic_exchange_info() -> DynamicExchangeInfo<'static> {
        let config = create_dynamic_exchange_config(ASSET_ID);
        DynamicExchangeInfo {
            config,
            amount: 100,
            status: "open",
        }
    }

    pub fn create_dynamic_exchange_config(
        penalty_asset: &'static str,
    ) -> DynamicExchangeConfig<'static> {
        let map = BTreeMap::new();
        let mut data = DynamicExchangeConfig {
            asset: ASSET_ID,
            seller: SELLER_ID,
            guarantor: GUARANTOR_ID,
            assets: map,
            guarantor_fee: 50, // guarantor_fee:  50/1000*100 =>  5%
            penalty_fee: 100, // penalty_fee:   100/1000*100 => 10% (because penalty_asset == asset)
            penalty_asset: penalty_asset,
        };
        data.assets.insert(ASSET1_ID, 250); //  1 ASSET1 = 250/100 = 2.5 ASSET_SRC
        data.assets.insert(ASSET2_ID, 310); //  1 ASSET2 = 310/100 = 3.1 ASSET_SRC
        data
    }

    #[test]
    fn dynamic_exchange_config_serialize() {
        let data = create_dynamic_exchange_config(ASSET_ID);

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), DYNAMIC_EXCHANGE_CONFIG_HEX);
    }

    #[test]
    fn dynamic_exchange_config_deserialize() {
        let expected = create_dynamic_exchange_config(ASSET_ID);
        let buf = hex::decode(DYNAMIC_EXCHANGE_CONFIG_HEX).unwrap();

        let data: DynamicExchangeConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn dynamic_exchange_apply_args_serialize() {
        let data = ApplyArgs {
            asset: ASSET_ID,
            amount: 42u64,
        };

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), DYNAMIC_EXCHANGE_APPLY_ARGS_HEX);
    }

    #[test]
    fn dynamic_exchange_apply_args_deserialize() {
        let expected = ApplyArgs {
            asset: ASSET_ID,
            amount: 42u64,
        };

        let buf = hex::decode(DYNAMIC_EXCHANGE_APPLY_ARGS_HEX).unwrap();

        let data: ApplyArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn dynamic_exchange_info_serialize() {
        let data = create_dynamic_exchange_info();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), DYNAMIC_EXCHANGE_INFO_HEX);
    }

    #[test]
    fn dynamic_exchange_info_deserialize() {
        let expected = create_dynamic_exchange_info();
        let buf = hex::decode(DYNAMIC_EXCHANGE_INFO_HEX).unwrap();

        let data: DynamicExchangeInfo = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }
}
