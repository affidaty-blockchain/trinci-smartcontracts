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

use serde_derive::{Deserialize, Serialize};

/// Contract registration arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct ContractRegistrationArgs<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub description: &'a str,
    pub url: &'a str,
    #[serde(with = "serde_bytes")]
    pub bin: &'a [u8],
}

/// Contract registration internal data.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct ContractRegistrationData<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub creator: &'a str,
    pub description: &'a str,
    pub url: &'a str,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use trinci_sdk::{rmp_deserialize, rmp_serialize};

    const CONTRACT_REGISTRATION_ARGS_HEX: &str = "95aa6d79636f6e7472616374a5302e312e30bc54686973206973206d7920706572736f6e616c20636f6e7472616374b9687474703a2f2f7777772e6d79636f6e74726163742e6f7267c403010203";
    const CONTRACT_REGISTRATION_DATA_HEX: &str = "95aa6d79636f6e7472616374a5302e312e30d92e516d43616c6c657250467a6e78455836533331364d3479566d786478504236584e36336f626a46596b50364d4c71bc54686973206973206d7920706572736f6e616c20636f6e7472616374b9687474703a2f2f7777772e6d79636f6e74726163742e6f7267";

    pub const CALLER_ID: &str = "QmCallerPFznxEX6S316M4yVmxdxPB6XN63objFYkP6MLq";
    pub const SERVICE_ID: &str = "QmServicef5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPf123";

    pub const CONTRACT_MULTIHASH: &[u8] = &[
        18, 32, 3, 144, 88, 198, 242, 192, 203, 73, 44, 83, 59, 10, 77, 20, 239, 119, 204, 15, 120,
        171, 204, 206, 213, 40, 125, 132, 161, 162, 1, 28, 251, 129,
    ];

    pub fn create_contract_registration_args() -> ContractRegistrationArgs<'static> {
        ContractRegistrationArgs {
            name: "mycontract",
            version: "0.1.0",
            description: "This is my personal contract",
            url: "http://www.mycontract.org",
            bin: &[1u8, 2, 3],
        }
    }

    pub fn create_contract_registration_data() -> ContractRegistrationData<'static> {
        ContractRegistrationData {
            name: "mycontract",
            version: "0.1.0",
            creator: CALLER_ID,
            description: "This is my personal contract",
            url: "http://www.mycontract.org",
        }
    }

    #[test]
    fn contract_registration_args_serialize() {
        let data = create_contract_registration_args();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), CONTRACT_REGISTRATION_ARGS_HEX);
    }

    #[test]
    fn contract_registration_args_deserialize() {
        let expected = create_contract_registration_args();
        let buf = hex::decode(CONTRACT_REGISTRATION_ARGS_HEX).unwrap();

        let data: ContractRegistrationArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn contract_registration_data_serialize() {
        let data = create_contract_registration_data();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), CONTRACT_REGISTRATION_DATA_HEX);
    }

    #[test]
    fn contract_registration_data_deserialize() {
        let expected = create_contract_registration_data();
        let buf = hex::decode(CONTRACT_REGISTRATION_DATA_HEX).unwrap();

        let data: ContractRegistrationData = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }
}
