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

use serde::{Deserialize, Serialize};

use std::{
    collections::BTreeMap,
    fmt::{self, Display},
};

/// Escrow resolution status.
#[derive(Serialize, Deserialize, PartialEq)]
pub enum EscrowStatus {
    /// Contract _opened_ and ready to be `updated` by the guarantor.
    Open,
    /// Contract _closed_ with _success_. Funds moved to the merchants.
    Success,
    /// Contract _closed_ with _failure_. Funds returned to the customer.
    Failure,
}

impl Display for EscrowStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            EscrowStatus::Open => "open",
            EscrowStatus::Success => "closed success",
            EscrowStatus::Failure => "closed failure",
        };
        write!(f, "{}", msg)
    }
}

/// Escrow configuration
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default))]
pub struct EscrowConfig<'a> {
    /// The asset involved on escrow contract
    pub asset: &'a str,
    /// Who guarantees the contract
    pub guarantor: &'a str,
    /// Who buy and must pay the merchants
    pub customer: &'a str,
    /// Who receive asset from the finalization with success
    pub merchants: BTreeMap<&'a str, u64>,
}

/// Update (contract finalization method) arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct UpdateArgs<'a> {
    /// Status finalization, shall be "OK" or "KO"
    pub status: &'a str,
}

/// Asset balance arguments.
///
/// Returns the asset amount on the contract
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct BalanceArgs<'a> {
    pub asset: &'a str,
}

/// Initialization arguments.
pub type InitArgs<'a> = EscrowConfig<'a>;

/// Information about the escrow
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Default))]
pub struct EscrowInfo<'a> {
    /// Contract configuration
    pub config: EscrowConfig<'a>,
    /// Contract asset balance
    pub amount: u64,
    /// Contract status: "open", "closed success", "closed failure"
    pub status: &'a str,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use std::collections::BTreeMap;
    use trinci_sdk::{rmp_deserialize, rmp_serialize};

    pub const ASSET_ID: &str = "QmXEuDQTpccaEETHWMqXPHcrVRUqaD9JxRHLdSVsCFgxj5";
    pub const GUARANTOR_ID: &str = "QmSCRCPFznxEX6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq";
    pub const CUSTOMER_ID: &str = "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";
    pub const MERCHANT1_ID: &str = "QmQH99ydqr7Ci1Hsj5Eb5DnbR1hDZRFpRrQVeQZHkibuEp";
    pub const MERCHANT2_ID: &str = "QmTuhaS8rBRjBSxPYHGVGtZmmkN3fVtHJTuTbwwUSdnB8a";

    const ESCROW_CONFIG_HEX: &str = "94d92e516d5845754451547063636145455448574d715850486372565255716144394a7852484c64535673434667786a35d92e516d5343524350467a6e78455836533331364d3479566d786478504236584e36336f62324c6a46596b50364d4c71d92e516d59486e45514c64663568374b59626a4650754853526b325350676458724a574668355736393648506671376982d92e516d51483939796471723743693148736a35456235446e62523168445a5246705272515665515a486b6962754570ccc3d92e516d5475686153387242526a425378505948475647745a6d6d6b4e33665674484a5475546277775553646e42386105";
    const BALANCE_ARGS_HEX: &str = "91a346434b";
    const UPDATE_ARGS_HEX: &str = "91a24f4b";
    const ESCROW_INFO_HEX: &str = "9394d92e516d5845754451547063636145455448574d715850486372565255716144394a7852484c64535673434667786a35d92e516d5343524350467a6e78455836533331364d3479566d786478504236584e36336f62324c6a46596b50364d4c71d92e516d59486e45514c64663568374b59626a4650754853526b325350676458724a574668355736393648506671376982d92e516d51483939796471723743693148736a35456235446e62523168445a5246705272515665515a486b6962754570ccc3d92e516d5475686153387242526a425378505948475647745a6d6d6b4e33665674484a5475546277775553646e423861052aae636c6f736564206661696c757265";

    /// Unfortunatelly looks like the `Derive` macro doesn't work here.
    /// This is after that we've made the `EscrowData` generic over the `Hasher`.
    impl PartialEq for EscrowConfig<'_> {
        fn eq(&self, other: &EscrowConfig<'_>) -> bool {
            self.merchants == other.merchants
                && self.asset == other.asset
                && self.customer == other.customer
                && self.guarantor == other.guarantor
        }
    }

    pub fn create_escrow_config() -> EscrowConfig<'static> {
        let map = BTreeMap::new();
        let mut data = EscrowConfig {
            asset: ASSET_ID,
            customer: CUSTOMER_ID,
            guarantor: GUARANTOR_ID,
            merchants: map,
        };
        data.merchants.insert(MERCHANT1_ID, 195);
        data.merchants.insert(MERCHANT2_ID, 5);
        data
    }

    #[test]
    fn escrow_config_serialize() {
        let data = create_escrow_config();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ESCROW_CONFIG_HEX);
    }

    #[test]
    fn escrow_config_deserialize() {
        let expected = create_escrow_config();
        let buf = hex::decode(ESCROW_CONFIG_HEX).unwrap();

        let data: EscrowConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn balance_args_serialize() {
        let args = BalanceArgs { asset: "FCK" };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), BALANCE_ARGS_HEX);
    }

    #[test]
    fn balance_args_deserialize() {
        let expected = BalanceArgs { asset: "FCK" };
        let buf = hex::decode(BALANCE_ARGS_HEX).unwrap();

        let args: BalanceArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn update_args_serialize() {
        let args = UpdateArgs { status: "OK" };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), UPDATE_ARGS_HEX);
    }

    #[test]
    fn update_args_deserialize() {
        let expected = UpdateArgs { status: "OK" };
        let buf = hex::decode(UPDATE_ARGS_HEX).unwrap();

        let args: UpdateArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn escrow_info_serialize() {
        let config = create_escrow_config();
        let args = EscrowInfo {
            config,
            amount: 42,
            status: &EscrowStatus::Failure.to_string(),
        };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), ESCROW_INFO_HEX);
    }

    #[test]
    fn escrow_info_deserialize() {
        let expected = EscrowInfo {
            config: create_escrow_config(),
            amount: 42,
            status: &EscrowStatus::Failure.to_string(),
        };
        let buf = hex::decode(ESCROW_INFO_HEX).unwrap();

        let args: EscrowInfo = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }
}
