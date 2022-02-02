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

/// Load data arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct LoadDataArgs<'a> {
    /// Location to retrieve the data to
    pub key: &'a str,
}

/// Remove data arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct RemoveDataArgs<'a> {
    /// Location to delete the data
    pub key: &'a str,
}

/// Store data arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct StoreDataArgs<'a> {
    /// Location to save the data to
    pub key: &'a str,
    /// Data to save
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
}

/// TAI transfer arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct TransferArgs<'a> {
    /// Destination account-id
    pub to: &'a str,
    /// Asset to transfer
    pub asset: &'a str,
    /// Amount to transfer
    pub units: u64,
}

/// TAI balance arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct BalanceArgs<'a> {
    pub asset: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    const BALANCE_ARGS_HEX: &str = "91a346434b";
    const LOAD_DATA_ARGS_HEX: &str = "91a464617461";
    const STORE_DATA_ARGS_HEX: &str = "92a464617461c403010203";
    const REMOVE_DATA_ARGS_HEX: &str = "91a66d7964617461";

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
    fn load_data_args_serialize() {
        let args = LoadDataArgs { key: "data" };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), LOAD_DATA_ARGS_HEX);
    }

    #[test]
    fn load_data_args_deserialize() {
        let expected = LoadDataArgs { key: "data" };
        let buf = hex::decode(LOAD_DATA_ARGS_HEX).unwrap();

        let args: LoadDataArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn store_data_args_serialize() {
        let args = StoreDataArgs {
            key: "data",
            data: &[1u8, 2, 3],
        };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), STORE_DATA_ARGS_HEX);
    }

    #[test]
    fn store_data_args_deserialize() {
        let expected = StoreDataArgs {
            key: "data",
            data: &[1u8, 2, 3],
        };
        let buf = hex::decode(STORE_DATA_ARGS_HEX).unwrap();

        let args: StoreDataArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }

    #[test]
    fn remove_data_args_serialize() {
        let args = RemoveDataArgs { key: "mydata" };

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), REMOVE_DATA_ARGS_HEX);
    }

    #[test]
    fn remove_data_deserialize() {
        let expected = RemoveDataArgs { key: "mydata" };
        let buf = hex::decode(REMOVE_DATA_ARGS_HEX).unwrap();

        let args: RemoveDataArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }
}
