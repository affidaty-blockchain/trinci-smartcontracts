use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

// Init Args
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct InitArgs<'a> {
    pub restaurateur: &'a str, // is the merchant account
    pub asset: &'a str,        // is the asset account
    pub part: u64,             // is the the share for each diner
    #[cfg_attr(test, serde(serialize_with = "tests::ser_ordered_map"))]
    pub customers: HashMap<&'a str, bool>, // the diners list
    pub status: &'a str,       // status of the contract: "open", "closed"
}

#[cfg(test)]
pub(crate) mod tests {

    use std::collections::BTreeMap;

    use serde::{Serialize, Serializer};

    use super::*;

    const INIT_ARGS_HEX: &str = "95d92e516d5265737461757261746575725438696a5737524564334b714e316b474267597846577378616a6a67754c6b79d92e516d41737365745438696a73646657736437524566333564334b714e316b474267597846577378616a6a67754c6b1e83d92e516d437573746f6d6572312d5438696a7364665773333564334b714e316b474267597846577378616a6a67754c6bc2d92e516d437573746f6d6572322d5438696a7364665773333564334b714e316b474267597846577378616a6a67754c6bc2d92e516d437573746f6d6572332d5438696a7364665773333564334b714e316b474267597846577378616a6a67754c6bc2a46f70656e";

    pub(crate) const PAY_ID: &str = "QmContractd7RqEzdV3gKjqXN1kGBgYxFWsxajjguLkyy7";
    pub(crate) const RESTAURATEUR_ID: &str = "QmRestaurateurT8ijW7REd3KqN1kGBgYxFWsxajjguLky";
    pub(crate) const ASSET_ID: &str = "QmAssetT8ijsdfWsd7REf35d3KqN1kGBgYxFWsxajjguLk";
    pub(crate) const CUSTOMER1_ID: &str = "QmCustomer1-T8ijsdfWs35d3KqN1kGBgYxFWsxajjguLk";
    pub(crate) const CUSTOMER2_ID: &str = "QmCustomer2-T8ijsdfWs35d3KqN1kGBgYxFWsxajjguLk";
    pub(crate) const CUSTOMER3_ID: &str = "QmCustomer3-T8ijsdfWs35d3KqN1kGBgYxFWsxajjguLk";

    pub(crate) fn ser_ordered_map<S>(
        value: &HashMap<&str, bool>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ordered: BTreeMap<_, _> = value.iter().collect();
        ordered.serialize(serializer)
    }

    pub(crate) fn create_init_args() -> InitArgs<'static> {
        let mut customers = HashMap::new();
        customers.insert(CUSTOMER3_ID, false);
        customers.insert(CUSTOMER2_ID, false);
        customers.insert(CUSTOMER1_ID, false);

        InitArgs {
            restaurateur: RESTAURATEUR_ID,
            asset: ASSET_ID,
            part: 30,
            customers,
            status: "open",
        }
    }

    #[test]
    fn init_args_serialize() {
        let args = create_init_args();

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), INIT_ARGS_HEX);
    }

    #[test]
    fn init_args_deserialize() {
        let expected = create_init_args();

        let buf = hex::decode(INIT_ARGS_HEX).unwrap();

        let args: InitArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }
}
