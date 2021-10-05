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

/// Alias registration arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct AliasRegistrationArgs<'a> {
    pub alias: &'a str,
}

/// Alias deletion arguments.
pub type AliasDeletionArgs<'a> = AliasRegistrationArgs<'a>;

/// Alias lookup arguments.
pub type AliasLookupArgs<'a> = AliasRegistrationArgs<'a>;

/// Oracle registration arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct OracleRegistrationArgs<'a> {
    /// Oracle account id
    pub id: &'a str,
    /// Oracle name
    pub name: &'a str,
    /// Oracle description
    pub description: &'a str,
    /// Oracle contract web site
    pub url: &'a str,
    /// Oracle contract hash
    #[serde(with = "serde_bytes")]
    pub contract: &'a [u8],
}

/// Oracle registration internal data.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct OracleRegistrationData<'a> {
    /// Oracle name
    pub name: &'a str,
    /// Oracle creator (is the oracle_registration caller)
    pub creator: &'a str,
    /// Oracle description
    pub description: &'a str,
    /// Oracle contract web site
    pub url: &'a str,
    /// Oracle contract hash
    #[serde(with = "serde_bytes")]
    pub contract: &'a [u8],
}

/// Contract registration arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct ContractRegistrationArgs<'a> {
    /// Contract name
    pub name: &'a str,
    /// Contract version
    pub version: &'a str,
    /// Contract description
    pub description: &'a str,
    /// Contract web site
    pub url: &'a str,
    /// Contract binary content
    #[serde(with = "serde_bytes")]
    pub bin: &'a [u8],
}

/// Contract registration internal data.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct ContractRegistrationData<'a> {
    /// Contract name
    pub name: &'a str,
    /// Contract version
    pub version: &'a str,
    /// Contract creator (is the contract_registration caller)
    pub creator: &'a str,
    /// Contract description
    pub description: &'a str,
    /// Contract web site
    pub url: &'a str,
}

/// Asset registration arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct AssetRegistrationArgs<'a> {
    /// Asset account id
    pub id: &'a str,
    /// Asset name
    pub name: &'a str,
    /// Asset web site
    pub url: &'a str,
    /// Asset contract hash
    #[serde(with = "serde_bytes")]
    pub contract: &'a [u8],
}

/// Asset registration internal data.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct AssetRegistrationData<'a> {
    /// Asset name
    pub name: &'a str,
    /// Asset creator (is the asset_registration caller)
    pub creator: &'a str,
    /// Asset web site
    pub url: &'a str,
    /// Asset contract hash
    #[serde(with = "serde_bytes")]
    pub contract: &'a [u8],
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct GetAssetArgs<'a> {
    pub asset_id: &'a str,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct GetOracleArgs<'a> {
    pub oracle_id: &'a str,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct GetContractArgs<'a> {
    /// Contract hash as string
    pub contract: &'a str,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use trinci_sdk::{rmp_deserialize, rmp_serialize};

    const ASSET_REGISTRATION_ARGS_HEX: &str = "94d92e516d41737365744c64663568374b59626a4650754853526b325350676458724a5746683557363936485066713769a64d79436f696eb7687474703a2f2f7777772e6d79636f696e2e6d6f6e6579c403010203";
    const ASSET_REGISTRATION_DATA_HEX: &str = "94a64d79436f696ed92e516d43616c6c657250467a6e78455836533331364d3479566d786478504236584e36336f626a46596b50364d4c71b7687474703a2f2f7777772e6d79636f696e2e6d6f6e6579c403010203";
    const GET_ASSET_ARGS_HEX: &str = "91d92e516d41737365744c64663568374b59626a4650754853526b325350676458724a5746683557363936485066713769";
    const CONTRACT_REGISTRATION_ARGS_HEX: &str = "95aa6d79636f6e7472616374a5302e312e30bc54686973206973206d7920706572736f6e616c20636f6e7472616374b9687474703a2f2f7777772e6d79636f6e74726163742e6f7267c403010203";
    const CONTRACT_REGISTRATION_DATA_HEX: &str = "95aa6d79636f6e7472616374a5302e312e30d92e516d43616c6c657250467a6e78455836533331364d3479566d786478504236584e36336f626a46596b50364d4c71bc54686973206973206d7920706572736f6e616c20636f6e7472616374b9687474703a2f2f7777772e6d79636f6e74726163742e6f7267";
    const GET_CONTRACT_ARGS_HEX: &str = "91d9443132323030333930353863366632633063623439326335333362306134643134656637376363306637386162636363656435323837643834613161323031316366623831";
    const ORACLE_REGISTRATION_ARGS_HEX: &str = "95d92e516d4f7261636c6564663568374b59626a4650754853526b325350676458724a5746683557363936485066313233ab54696d65204f7261636c65d928546869732077696c6c20736179207468652074696d6520696e2074686520626c6f636b636861696eb5687474703a2f2f54696d654f7261636c652e6f7267c403010203";
    const ORACLE_REGISTRATION_DATA_HEX: &str = "95ab54696d65204f7261636c65d92e516d43616c6c657250467a6e78455836533331364d3479566d786478504236584e36336f626a46596b50364d4c71d928546869732077696c6c20736179207468652074696d6520696e2074686520626c6f636b636861696eb5687474703a2f2f54696d654f7261636c652e6f7267c403010203";
    const GET_ORACLE_ARGS_HEX: &str = "91d92e516d4f7261636c6564663568374b59626a4650754853526b325350676458724a5746683557363936485066313233";
    const ALIAS_REGISTRATION_ARGS_HEX: &str = "91ab4d79436f6f6c416c696173";

    pub const CALLER_ID: &str = "QmCallerPFznxEX6S316M4yVmxdxPB6XN63objFYkP6MLq";
    pub const ASSET_ID: &str = "QmAssetLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";
    pub const ORACLE_ID: &str = "QmOracledf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPf123";
    pub const SERVICE_ID: &str = "QmServicef5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPf123";
    pub const USER_ID: &str = "QmUserfqwe5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPf123";

    pub const CONTRACT_MULTIHASH: &[u8] = &[
        18, 32, 3, 144, 88, 198, 242, 192, 203, 73, 44, 83, 59, 10, 77, 20, 239, 119, 204, 15, 120,
        171, 204, 206, 213, 40, 125, 132, 161, 162, 1, 28, 251, 129,
    ];

    pub fn create_asset_registration_args() -> AssetRegistrationArgs<'static> {
        AssetRegistrationArgs {
            id: ASSET_ID,
            name: "MyCoin",
            url: "http://www.mycoin.money",
            contract: &[1, 2, 3],
        }
    }

    pub fn create_asset_registration_data() -> AssetRegistrationData<'static> {
        AssetRegistrationData {
            creator: CALLER_ID,
            name: "MyCoin",
            url: "http://www.mycoin.money",
            contract: &[1, 2, 3],
        }
    }

    pub fn create_get_asset_information_args(asset_id: &str) -> GetAssetArgs {
        GetAssetArgs { asset_id }
    }

    pub fn create_get_oracle_information_args(oracle_id: &str) -> GetOracleArgs {
        GetOracleArgs { oracle_id }
    }

    pub fn create_get_contract_information_args(contract: &str) -> GetContractArgs {
        GetContractArgs { contract }
    }

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

    pub fn create_oracle_registration_args() -> OracleRegistrationArgs<'static> {
        OracleRegistrationArgs {
            id: ORACLE_ID,
            name: "Time Oracle",
            description: "This will say the time in the blockchain",
            url: "http://TimeOracle.org",
            contract: &[1, 2, 3],
        }
    }

    pub fn create_oracle_registration_data() -> OracleRegistrationData<'static> {
        OracleRegistrationData {
            name: "Time Oracle",
            creator: CALLER_ID,
            description: "This will say the time in the blockchain",
            url: "http://TimeOracle.org",
            contract: &[1, 2, 3],
        }
    }

    pub fn create_alias_registration_args(alias: &'static str) -> AliasRegistrationArgs<'static> {
        AliasRegistrationArgs { alias }
    }
    pub fn create_alias_deletion_args(alias: &'static str) -> AliasDeletionArgs<'static> {
        AliasDeletionArgs { alias }
    }
    pub fn create_alias_lookup_args(alias: &'static str) -> AliasLookupArgs<'static> {
        AliasLookupArgs { alias }
    }

    #[test]
    fn asset_registration_args_serialize() {
        let data = create_asset_registration_args();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ASSET_REGISTRATION_ARGS_HEX);
    }

    #[test]
    fn asset_registration_args_deserialize() {
        let expected = create_asset_registration_args();
        let buf = hex::decode(ASSET_REGISTRATION_ARGS_HEX).unwrap();

        let data: AssetRegistrationArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn asset_registration_data_serialize() {
        let data = create_asset_registration_data();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ASSET_REGISTRATION_DATA_HEX);
    }

    #[test]
    fn asset_registration_data_deserialize() {
        let expected = create_asset_registration_data();
        let buf = hex::decode(ASSET_REGISTRATION_DATA_HEX).unwrap();

        let data: AssetRegistrationData = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
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

    #[test]
    fn oracle_registration_args_serialize() {
        let data = create_oracle_registration_args();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ORACLE_REGISTRATION_ARGS_HEX);
    }

    #[test]
    fn oracle_registration_args_deserialize() {
        let expected = create_oracle_registration_args();
        let buf = hex::decode(ORACLE_REGISTRATION_ARGS_HEX).unwrap();

        let data: OracleRegistrationArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn oracle_registration_data_serialize() {
        let data = create_oracle_registration_data();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ORACLE_REGISTRATION_DATA_HEX);
    }

    #[test]
    fn oracle_registration_data_deserialize() {
        let expected = create_oracle_registration_data();
        let buf = hex::decode(ORACLE_REGISTRATION_DATA_HEX).unwrap();

        let data: OracleRegistrationData = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn get_asset_args_serialize() {
        let data = create_get_asset_information_args(ASSET_ID);

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), GET_ASSET_ARGS_HEX);
    }

    #[test]
    fn get_asset_args_deserialize() {
        let expected = create_get_asset_information_args(ASSET_ID);
        let buf = hex::decode(GET_ASSET_ARGS_HEX).unwrap();

        let data: GetAssetArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn get_oracle_args_serialize() {
        let data = create_get_oracle_information_args(ORACLE_ID);

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), GET_ORACLE_ARGS_HEX);
    }

    #[test]
    fn get_oracle_args_deserialize() {
        let expected = create_get_oracle_information_args(ORACLE_ID);
        let buf = hex::decode(GET_ORACLE_ARGS_HEX).unwrap();

        let data: GetOracleArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn get_contract_args_serialize() {
        let contract_hash = hex::encode(CONTRACT_MULTIHASH);
        let data = create_get_contract_information_args(&contract_hash);

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), GET_CONTRACT_ARGS_HEX);
    }

    #[test]
    fn get_contract_args_deserialize() {
        let contract_hash = hex::encode(CONTRACT_MULTIHASH);
        let expected = create_get_contract_information_args(&contract_hash);
        let buf = hex::decode(GET_CONTRACT_ARGS_HEX).unwrap();

        let data: GetContractArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn alias_registration_args_serialize() {
        let data = create_alias_registration_args("MyCoolAlias");

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), ALIAS_REGISTRATION_ARGS_HEX);
    }

    #[test]
    fn alias_registration_args_deserialize() {
        let expected = create_alias_registration_args("MyCoolAlias");
        let buf = hex::decode(ALIAS_REGISTRATION_ARGS_HEX).unwrap();

        let data: AliasRegistrationArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }
}
