use serde::{Deserialize, Serialize};

/// Time Oracle initialization arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct InitArgs<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub update_interval: u64,
    pub initial_time: u64,
}

/// Time Oracle configuration data.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct ConfigData<'a> {
    pub name: &'a str,
    pub owner: &'a str,
    pub description: &'a str,
    pub update_interval: u64,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use trinci_sdk::{rmp_deserialize, rmp_serialize};

    const INIT_ARGS_HEX: &str =
        "94ab54696d65204f7261636c65b55468697320697320612074696d65206f7261636c65cd0e10ce60c38010";
    const CONFIG_DATA_HEX: &str =
        "94ab54696d65204f7261636c65a543726f6e6fb55468697320697320612074696d65206f7261636c65cd0e10";
    const CURRENT_TIME_HEX: &str = "ce60c381c4";

    pub const INIT_TIME_VALUE: u64 = 1623425040;

    pub fn create_init_args() -> InitArgs<'static> {
        InitArgs {
            name: "Time Oracle",
            description: "This is a time oracle",
            update_interval: 3600,
            initial_time: INIT_TIME_VALUE,
        }
    }

    pub fn create_config_data() -> ConfigData<'static> {
        ConfigData {
            name: "Time Oracle",
            owner: "Crono",
            description: "This is a time oracle",
            update_interval: 3600,
        }
    }

    #[test]
    fn current_time_serialize() {
        let time = 1623425476_u64;

        let buf = rmp_serialize(&time).unwrap();

        assert_eq!(hex::encode(&buf), CURRENT_TIME_HEX);
    }

    #[test]
    fn current_time_deserialize() {
        let expected = 1623425476_u64;
        let buf = hex::decode(CURRENT_TIME_HEX).unwrap();

        let time: u64 = rmp_deserialize(&buf).unwrap();

        assert_eq!(time, expected);
    }

    #[test]
    fn init_args_serialize() {
        let data = create_init_args();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), INIT_ARGS_HEX);
    }

    #[test]
    fn init_args_deserialize() {
        let expected = create_init_args();
        let buf = hex::decode(INIT_ARGS_HEX).unwrap();

        let data: InitArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn config_serialize() {
        let data = create_config_data();

        let buf = rmp_serialize(&data).unwrap();

        assert_eq!(hex::encode(&buf), CONFIG_DATA_HEX);
    }

    #[test]
    fn config_data_deserialize() {
        let expected = create_config_data();
        let buf = hex::decode(CONFIG_DATA_HEX).unwrap();

        let data: ConfigData = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, expected);
    }
}
