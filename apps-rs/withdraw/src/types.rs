use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

/// Withdraw status.
#[derive(Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Debug, Clone))]
pub enum WithdrawStatus {
    /// Open and ready to be "updated" by the exchange.
    Open,
    /// Closed with success. Funds burned.
    Success,
    /// Closed with failure. Funds returned to the customer and the exchange.
    Failure,
}

impl Display for WithdrawStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            WithdrawStatus::Open => "open",
            WithdrawStatus::Success => "closed success",
            WithdrawStatus::Failure => "closed failure",
        };
        write!(f, "{}", msg)
    }
}

// Inner asset struct with name and units
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, PartialEq, Default))]
pub struct InnerAsset<'a> {
    pub id: &'a str,
    pub units: u64,
}

// Configuration data
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default, PartialEq))]
pub struct WithdrawConfig<'a> {
    pub customer: &'a str,
    pub exchange: &'a str,
    pub currency_asset: InnerAsset<'a>,
    pub withdrawn_asset: InnerAsset<'a>,
}

/// Initialization arguments.
pub type InitArgs<'a> = WithdrawConfig<'a>;

// Update arguments
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, PartialEq))]
pub struct UpdateArgs<'a> {
    pub status: &'a str,
}

/// Asset Burn method arguments.
/// MUST BE THE SAME AS IN THE TRINCI ASSET CONTRACT
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct BurnArgs<'a> {
    /// Source account.
    pub from: &'a str,
    /// Number of units.
    pub units: u64,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default))]
pub struct WithdrawInfo<'a> {
    pub config: WithdrawConfig<'a>,
    pub currency_asset_amount: u64,
    pub withdrawn_asset_amount: u64,
    pub status: &'a str,
}

#[cfg(test)]
pub(crate) mod tests {

    use super::*;

    pub const WITHDRAW_ID: &str = "QmWithdraw_pccaEETHWRUqaD9JxRHLdSVsCFgxjwerwv5";
    pub const CURRENCY_ASSET_ID: &str = "QmCurrencyAsset_pccaEETHWRUqaD9JxRHLdSVsCFgxj5";
    pub const WITHDRAWN_ASSET_ID: &str = "QmwithdrawnAsset_pccaEETHWMqXPqaD9JxRHLdSVsCFgxqe";
    pub const EXCHANGE_ID: &str = "QmExchange_EX6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq";
    pub const CUSTOMER_ID: &str = "QmCustomer_h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";

    const INNER_ASSET_HEX: &str = "92a74d7941737365742a";
    const WITHDRAW_CONFIG_HEX: &str =
        "94d92e516d437573746f6d65725f68374b59626a4650754853526b325350676458724a5746683557363936485066713769d92e516d45786368616e67655f455836533331364d3479566d786478504236584e36336f62324c6a46596b50364d4c7192d92e516d43757272656e637941737365745f7063636145455448575255716144394a7852484c64535673434667786a352a92d931516d77697468647261776e41737365745f7063636145455448574d715850716144394a7852484c6453567343466778716564";
    const UPDATE_ARGS_HEX: &str = "91a26f6b";
    const WITHDRAW_INFO_HEX: &str = "9494d92e516d437573746f6d65725f68374b59626a4650754853526b325350676458724a5746683557363936485066713769d92e516d45786368616e67655f455836533331364d3479566d786478504236584e36336f62324c6a46596b50364d4c7192d92e516d43757272656e637941737365745f7063636145455448575255716144394a7852484c64535673434667786a356492d931516d77697468647261776e41737365745f7063636145455448574d715850716144394a7852484c645356734346677871652a642aa46f70656e";

    pub(crate) fn create_withdraw_config(
        currency_units: u64,
        asset_units: u64,
    ) -> WithdrawConfig<'static> {
        WithdrawConfig {
            customer: CUSTOMER_ID,
            exchange: EXCHANGE_ID,
            currency_asset: InnerAsset {
                id: CURRENCY_ASSET_ID,
                units: currency_units,
            },
            withdrawn_asset: InnerAsset {
                id: WITHDRAWN_ASSET_ID,
                units: asset_units,
            },
        }
    }

    pub(crate) fn create_withdraw_info(
        currency_units: u64,
        asset_units: u64,
        status: &'static str,
    ) -> WithdrawInfo<'static> {
        WithdrawInfo {
            config: create_withdraw_config(currency_units, asset_units),
            currency_asset_amount: currency_units,

            withdrawn_asset_amount: asset_units,
            status,
        }
    }

    impl PartialEq for WithdrawInfo<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.config == other.config
                && self.currency_asset_amount == other.currency_asset_amount
                && self.withdrawn_asset_amount == other.withdrawn_asset_amount
                && self.status == other.status
        }

        fn ne(&self, other: &Self) -> bool {
            self.config == other.config
                && self.currency_asset_amount == other.currency_asset_amount
                && self.withdrawn_asset_amount == other.withdrawn_asset_amount
                && self.status == other.status
        }
    }

    #[test]
    fn inner_asset_serialize() {
        let args = InnerAsset {
            id: "MyAsset",
            units: 42,
        };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), INNER_ASSET_HEX);
    }

    #[test]
    fn inner_asset_deserialize() {
        let expected = InnerAsset {
            id: "MyAsset",
            units: 42,
        };

        let buf = hex::decode(INNER_ASSET_HEX).unwrap();

        let args: InnerAsset = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn withdraw_config_serialize() {
        let args = create_withdraw_config(42, 100);

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), WITHDRAW_CONFIG_HEX);
    }

    #[test]
    fn withdraw_config_deserialize() {
        let expected = create_withdraw_config(42, 100);

        let buf = hex::decode(WITHDRAW_CONFIG_HEX).unwrap();

        let args: WithdrawConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn update_args_serialize() {
        let args = UpdateArgs { status: "ok" };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), UPDATE_ARGS_HEX);
    }

    #[test]
    fn update_args_deserialize() {
        let expected = UpdateArgs { status: "ok" };

        let buf = hex::decode(UPDATE_ARGS_HEX).unwrap();

        let args: UpdateArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn withdraw_info_serialize() {
        let args = create_withdraw_info(100, 42, "open");

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), WITHDRAW_INFO_HEX);
    }

    #[test]
    fn withdraw_info_deserialize() {
        let expected = create_withdraw_info(100, 42, "open");

        let buf = hex::decode(WITHDRAW_INFO_HEX).unwrap();

        let args: WithdrawInfo = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }
}
