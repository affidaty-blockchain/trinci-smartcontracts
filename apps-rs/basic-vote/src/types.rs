use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct MinMaxRule {
    pub min: u8,
    pub max: u8,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct QuestionOption<'a> {
    /// Numeric Identifier.
    pub id: &'a str,
    /// Question format (not used by the contract app).
    pub question: &'a str,
    /// Answer value.
    pub value: &'a str,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct Question<'a> {
    /// Numeric Identifier
    pub id: &'a str,
    /// Question text.
    pub question: &'a str,
    /// Min and max number of allowed answeres.
    pub rules: MinMaxRule,
    /// Question answer options.
    pub options: Vec<QuestionOption<'a>>,
}

/// Vote configuration data.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct VoteConfig<'a> {
    /// Short ballot information. Key is the language id (e.g. "en").
    pub title: BTreeMap<&'a str, &'a str>,
    /// Longer ballot information. Key is the language id (e.g. "en").
    pub description: BTreeMap<&'a str, &'a str>,
    /// Status: OPEN or CLOSE
    pub status: &'a str,
    /// If the polling is anonymous.
    pub owner: &'a str,
    /// Minumum and maximum questions where an answer is required.
    pub rules: MinMaxRule,
    /// Questions list.
    pub questions: Vec<Question<'a>>,
}

/// Vote initialization data.
pub type InitArgs<'a> = VoteConfig<'a>;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct QuestionOptionAnswer<'a> {
    pub id: &'a str,
    pub votes: u32,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct QuestionAnswer<'a> {
    pub id: &'a str,
    pub result: Vec<QuestionOptionAnswer<'a>>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct VoteArgsAnswer<'a> {
    pub id: &'a str,
    pub values: Vec<&'a str>,
}

/// Add vote arguments.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone, Default))]
pub struct VoteArgs<'a> {
    /// Answers list.
    #[serde(borrow)]
    pub answers: Vec<VoteArgsAnswer<'a>>,
}

#[cfg(test)]
pub(crate) mod tests {
    use lazy_static::lazy_static;

    use super::*;

    pub const OWNER_ID: &str = "QmWXBHFErKYgvByBKzoKwkYX6sUAGsXzmdz5uP3eeKfUjt";
    pub const POLL_STATION_ID: &str = "QmSCRCPFznxEX6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq";

    const VOTE_CONFIG_HEX: &str = "9681a2656ed928426573742054322063686172616374657220616e6420737072696e6720627265616b206c756e636881a2656ed92f566f746520746f206465636964652074686520626573742063686172616374657220616e64206c756e6368202e2e2ea44f50454ed92e516d575842484645724b5967764279424b7a6f4b776b5958367355414773587a6d647a3575503365654b66556a749202029294a131d9314578707265737320796f757220707265666572656e636520666f72207468652062657374205432206368617261637465729201019293a131a468746d6cab4a6f686e20436f6e6e6f7293a132a468746d6cae546865205465726d696e61746f7294a132d92e4578707265737320796f757220707265666572656e636520666f7220737072696e6720627265616b206c756e63689201029393a131a468746d6ca5537573686993a132a468746d6ca553616c616493a133a468746d6ca550697a7a61";

    const QUESTION_ANS_HEX: &str = "9292a1319292a1311492a1326492a1329392a1310392a132cd029a92a1332a";
    const VOTE_ARGS_HEX: &str = "919292a13191a13292a13292a131a133";

    pub fn create_test_vote_config() -> VoteConfig<'static> {
        lazy_static! {
            static ref E: Vec<u8> = hex::decode("010001").unwrap();
            static ref N512: Vec<u8> = hex::decode("db651a5584bc01af06507acac6fe2730ad135cd567ed92963123f64e951ee953d45b64e4eba2e88b0a3902f343bbf32983d43571aecc89b954996feb5260e11f").unwrap();
            static ref N2048: Vec<u8> = hex::decode("d0bff10834c15cadceca813f735887e96dc1f85f2d4e3b6b2a21a0388ead2298542a9660ddf383af826f215d55e73fb5f7e4460da4e236b8873675cc88e48e4b48f641b8f650135ab500379b705bc8e2854ccc0b40b941246298668daa5989ad8dc4b0deeebb96a84e8d514442a2a87b0c7d1283317197e5c6e529271336253148e1bfe21348f26defee7a2601bed32033bafde83c09c04db814bdb3d3c0731e479854ebf0134ed0ec2fcbb6d3f60153938f6a455895c1250014e2e964611399352ce231b4ea94e36a82e755f8959d75f9b05ec4f936ee04f72f6c4e63bb322be58bf839cf11f12edaa54df264f4077a4e3b13d4b3b33084e9266a8626452f1b").unwrap();
            static ref SALT: Vec<u8> = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
        }
        let mut title = BTreeMap::new();
        title.insert("en", "Best T2 character and spring break lunch");

        let mut description = BTreeMap::new();
        description.insert("en", "Vote to decide the best character and lunch ...");

        VoteConfig {
            title,
            description,
            status: "OPEN",
            owner: OWNER_ID,
            rules: MinMaxRule { min: 2, max: 2 },
            questions: vec![
                Question {
                    id: "1",
                    question: "Express your preference for the best T2 character",
                    rules: MinMaxRule { min: 1, max: 1 },
                    options: vec![
                        QuestionOption {
                            id: "1",
                            question: "html",
                            value: "John Connor",
                        },
                        QuestionOption {
                            id: "2",
                            question: "html",
                            value: "The Terminator",
                        },
                    ],
                },
                Question {
                    id: "2",
                    question: "Express your preference for spring break lunch",
                    rules: MinMaxRule { min: 1, max: 2 },
                    options: vec![
                        QuestionOption {
                            id: "1",
                            question: "html",
                            value: "Sushi",
                        },
                        QuestionOption {
                            id: "2",
                            question: "html",
                            value: "Salad",
                        },
                        QuestionOption {
                            id: "3",
                            question: "html",
                            value: "Pizza",
                        },
                    ],
                },
            ],
        }
    }

    pub fn create_test_answers() -> Vec<QuestionAnswer<'static>> {
        vec![
            QuestionAnswer {
                id: "1",
                result: vec![
                    QuestionOptionAnswer { id: "1", votes: 20 },
                    QuestionOptionAnswer {
                        id: "2",
                        votes: 100,
                    },
                ],
            },
            QuestionAnswer {
                id: "2",
                result: vec![
                    QuestionOptionAnswer { id: "1", votes: 3 },
                    QuestionOptionAnswer {
                        id: "2",
                        votes: 666,
                    },
                    QuestionOptionAnswer { id: "3", votes: 42 },
                ],
            },
        ]
    }

    pub fn create_test_vote_args() -> VoteArgs<'static> {
        VoteArgs {
            answers: vec![
                VoteArgsAnswer {
                    id: "1",
                    values: vec!["2"],
                },
                VoteArgsAnswer {
                    id: "2",
                    values: vec!["1", "3"],
                },
            ],
        }
    }

    #[test]
    fn vote_config_serialize() {
        let info = create_test_vote_config();

        let buf = trinci_sdk::rmp_serialize(&info).unwrap();

        assert_eq!(hex::encode(&buf), VOTE_CONFIG_HEX);
    }

    #[test]
    fn vote_config_deserialize() {
        let expected = create_test_vote_config();
        let buf = hex::decode(VOTE_CONFIG_HEX).unwrap();

        let info: VoteConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(info, expected);
    }

    #[test]
    fn question_answers_serialize() {
        let ans = create_test_answers();

        let buf = trinci_sdk::rmp_serialize(&ans).unwrap();

        assert_eq!(hex::encode(&buf), QUESTION_ANS_HEX);
    }

    #[test]
    fn question_answers_deserialize() {
        let expected = create_test_answers();
        let buf = hex::decode(QUESTION_ANS_HEX).unwrap();

        let info: Vec<QuestionAnswer> = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(info, expected);
    }

    #[test]
    fn vote_args_serialize() {
        let args = create_test_vote_args();

        let buf = trinci_sdk::rmp_serialize(&args).unwrap();

        assert_eq!(hex::encode(&buf), VOTE_ARGS_HEX);
    }

    #[test]
    fn vote_args_deserialize() {
        let expected = create_test_vote_args();
        let buf = hex::decode(VOTE_ARGS_HEX).unwrap();

        let args: VoteArgs = trinci_sdk::rmp_deserialize(&buf).unwrap();

        assert_eq!(args, expected);
    }
}
