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

//! Vote Contract
//!
//! Allows to perform anonymous online voting.
//!
//! ### Rules
//!
//! 1. At the moment anyone can starts a polling station (need to perform an authorization check)
//! 2. Anyone can get the ballot config
//! 3. An `add_vote` transaction si valid only if
//!  - the voter is authorized by an authority that issues a `blindly` signed token
//!  - the token has not been already spent
//!  - the voting is open
//!  - the votes are syntactically right
//! 4. The vote is closed by a `get_result` transaction from the ballot owner

use trinci_sdk::{
    rmp_deserialize, rmp_serialize, value, AppContext, PackedValue, Value, WasmError, WasmResult,
};

mod rsa;
mod types;

use types::*;

trinci_sdk::app_export!(init, get_config, add_vote, get_result);

const INIT_KEY: &str = "init";
const CONFIG_KEY: &str = "config";
const VOTES_KEY: &str = "votes";
const BURNED_KEY: &str = "burn";

/// Load configuration buffer.
fn get_config_buf() -> WasmResult<Vec<u8>> {
    let buf = trinci_sdk::load_data(CONFIG_KEY);
    match buf.is_empty() {
        false => Ok(buf),
        true => Err(WasmError::new("station not initialized")),
    }
}

/// Returns the ballot config data so the user can know the candidates and his voting station.
fn get_config(_ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    let buf = get_config_buf()?;
    let config: VoteConfig = trinci_sdk::rmp_deserialize(&buf)?;
    let buf = trinci_sdk::rmp_serialize_named(&config)?;
    Ok(PackedValue(buf))
}

// Create votes field from questions:
fn pack_votes_struct(config: &VoteConfig) -> WasmResult<Vec<u8>> {
    let mut votes: Vec<QuestionAnswer> = Vec::new();
    for question in &config.questions {
        let mut answer = QuestionAnswer {
            id: question.id,
            result: vec![],
        };
        for option in &question.options {
            let option_ans = QuestionOptionAnswer {
                id: option.id,
                votes: 0,
            };
            answer.result.push(option_ans);
        }
        votes.push(answer);
    }
    rmp_serialize(&votes)
}

/// Polling station initialization.
///
/// Called once if the account field are not yet properly setted check if the
/// content of the account `data` is correct, then move the configuration in the
/// `config` field and generates the votes field builded from the questions
/// field.
fn init(_ctx: AppContext, args: InitArgs) -> WasmResult<Value> {
    // TODO: Perform an authorization check

    if !trinci_sdk::load_data(INIT_KEY).is_empty() {
        return Ok(value!(null));
    }

    // Construct and save votes field.
    let buf = pack_votes_struct(&args)?;
    trinci_sdk::store_data(VOTES_KEY, &buf);

    // Create burned field.
    let burned: HashSet<&str> = HashSet::new();
    trinci_sdk::store_account_data_mp!(BURNED_KEY, &burned)?;

    // Store data as config
    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &args)?;

    // Add init flag.
    trinci_sdk::store_data(INIT_KEY, &[1]);

    Ok(value!(null))
}

fn get_question_rules(questions: &[Question], id: &str) -> MinMaxRule {
    for question in questions {
        if id == question.id {
            return MinMaxRule {
                min: question.rules.min,
                max: question.rules.max,
            };
        }
    }
    MinMaxRule { min: 0, max: 0 }
}

fn check_duplicate_answer_to_question(arr: &[VoteArgsAnswer]) -> WasmResult<()> {
    let uniques: std::collections::BTreeMap<_, _> = arr.iter().map(|c| (c.id, c.id)).collect();

    if uniques.len() != arr.len() {
        return Err(WasmError::new("duplicate preference"));
    }
    Ok(())
}

fn check_duplicate_vote(arr: &[&str]) -> WasmResult<()> {
    let uniques: std::collections::BTreeMap<_, _> = arr.iter().map(|c| (c, c)).collect();

    if uniques.len() != arr.len() {
        return Err(WasmError::new("duplicate preference"));
    }
    Ok(())
}

fn add_votes_values(vote: &mut Vec<QuestionOptionAnswer>, answer: &[&str]) -> WasmResult<()> {
    check_duplicate_vote(answer)?;

    for &answ in answer {
        for entry in &mut *vote {
            if entry.id == answ {
                entry.votes += 1;
            }
        }
    }
    Ok(())
}

/// Checks the voting token validity:
/// - station id.
/// - signature.
fn check_token(
    config: &VoteConfig,
    caller_id: &str,
    station_id: &str,
    token: &[u8],
) -> WasmResult<()> {
    // Get station associated to the caller id.
    let stations_count = config.polling_stations.len();
    let station_index: usize = caller_id
        .as_bytes()
        .iter()
        .fold(0usize, |acc, byte| (acc + *byte as usize) % stations_count);
    let station = match config.polling_stations.get(station_index) {
        Some(station) => station,
        None => return Err(WasmError::new("error retrieving the polling station")),
    };

    if station.id != station_id {
        return Err(WasmError::new("wrong polling station"));
    }
    match rsa::token_verify(
        token,
        caller_id,
        station.salt,
        station.pk_rsa.e,
        station.pk_rsa.n,
    ) {
        true => Ok(()),
        false => Err(WasmError::new("bad token")),
    }
}

fn check_voting_open(config: &VoteConfig) -> WasmResult<()> {
    if config.status != "OPEN" {
        return Err(WasmError::new("the status is not OPEN"));
    }
    Ok(())
}

fn set_question_answers(
    questions: &[Question],
    answers: &[VoteArgsAnswer],
    votes: &mut Vec<QuestionAnswer>,
) -> WasmResult<()> {
    for answer in answers {
        let answer_len = answer.values.len();
        let MinMaxRule { min, max } = get_question_rules(questions, answer.id);
        if answer_len < min as usize || answer_len > max as usize {
            return Err(WasmError::new("inner answer number inconsistency"));
        }

        let vote = votes
            .iter_mut()
            .find(|ans| ans.id == answer.id)
            .ok_or_else(|| WasmError::new("field not found"))?;
        add_votes_values(&mut vote.result, &answer.values)?;
    }
    Ok(())
}

use std::{collections::HashSet, u8};

/// Add the vote
fn add_vote(ctx: AppContext, args: VoteArgs) -> WasmResult<Value> {
    let buf = get_config_buf()?;
    let config: VoteConfig = trinci_sdk::rmp_deserialize(&buf)?;
    // Checks if the voting is open.
    check_voting_open(&config)?;
    // Checks the token signature and polling station.
    check_token(&config, ctx.caller, ctx.owner, args.token)?;
    // Checks if the token has been already spent.
    // TODO: maybe we can use a lighter hash map containing the crc32 of the caller.
    // And check the complete "burned" list only on duplicate detection.
    let buf = trinci_sdk::load_data(BURNED_KEY);
    let mut burned: HashSet<&str> = trinci_sdk::rmp_deserialize(&buf)?;
    if burned.contains(ctx.caller) {
        return Err(WasmError::new("token already burned"));
    }

    let answer_len = args.answers.len();
    if answer_len < config.rules.min as usize || answer_len > config.rules.max as usize {
        return Err(WasmError::new("answers number inconsistency"));
    }

    check_duplicate_answer_to_question(&args.answers)?;

    let buf = trinci_sdk::load_data(VOTES_KEY);
    let mut votes: Vec<QuestionAnswer> = rmp_deserialize(&buf)?;

    set_question_answers(&config.questions, &args.answers, &mut votes)?;

    trinci_sdk::store_account_data_mp!(VOTES_KEY, &votes)?;

    // Add the token to the burned list.
    burned.insert(ctx.caller);
    trinci_sdk::store_account_data_mp!(BURNED_KEY, &burned)?;

    Ok(value!(null))
}

/// Get voting result and close the polling station.
/// returns the data `votes` field only if the voting is ended.
fn get_result(ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    let buf = get_config_buf()?;
    let mut config: VoteConfig = trinci_sdk::rmp_deserialize(&buf)?;

    if !ctx.caller.eq(config.owner) {
        return Err(WasmError::new("not authorized"));
    }

    let buf = trinci_sdk::load_data(VOTES_KEY);
    let results: Vec<QuestionAnswer> = trinci_sdk::rmp_deserialize(&buf)?;
    let buf = trinci_sdk::rmp_serialize_named(&results)?;

    config.status = "CLOSED";
    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;

    Ok(PackedValue(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use trinci_sdk::not_wasm;

    const OWNER_ID: &str = types::tests::OWNER_ID;
    const POLL_STATION_ID: &str = types::tests::POLL_STATION2_ID;
    const VOTER_ID: &str = "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";

    fn prepare_full_env(caller: &'static str) -> AppContext<'static> {
        let config = types::tests::create_test_vote_config();
        let buf = trinci_sdk::rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(POLL_STATION_ID, CONFIG_KEY, &buf);

        let burned: HashSet<&str> = HashSet::new();
        let buf = trinci_sdk::rmp_serialize(&burned).unwrap();
        not_wasm::set_account_data(POLL_STATION_ID, BURNED_KEY, &buf);

        let buf = pack_votes_struct(&config).unwrap();
        not_wasm::set_account_data(POLL_STATION_ID, VOTES_KEY, &buf);

        not_wasm::create_app_context(POLL_STATION_ID, caller)
    }

    #[test]
    fn initializing_data() {
        let ctx = not_wasm::create_app_context(POLL_STATION_ID, OWNER_ID);
        let expected = types::tests::create_test_vote_config();

        not_wasm::call_wrap(init, ctx, expected.clone()).unwrap();

        let buf = not_wasm::get_account_data(POLL_STATION_ID, INIT_KEY);
        assert_eq!(buf[0], 1);
        let buf = not_wasm::get_account_data(POLL_STATION_ID, CONFIG_KEY);
        let config: VoteConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(expected, config);
        let buf = not_wasm::get_account_data(POLL_STATION_ID, VOTES_KEY);
        let votes: Vec<QuestionAnswer> = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(votes.len(), 2);
        let buf = not_wasm::get_account_data(POLL_STATION_ID, BURNED_KEY);
        let burned: HashSet<&str> = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(burned.len(), 0);
    }

    #[test]
    fn getting_config() {
        let ctx = prepare_full_env(OWNER_ID);
        let config = types::tests::create_test_vote_config();
        let expected = trinci_sdk::rmp_serialize_named(&config).unwrap();

        let pkt: PackedValue =
            not_wasm::call_wrap(get_config, ctx, PackedValue::default()).unwrap();

        assert_eq!(expected, pkt.0);
    }

    #[test]
    fn adding_vote() {
        let ctx = prepare_full_env(VOTER_ID);
        let args = types::tests::create_test_vote_args();

        not_wasm::call_wrap(add_vote, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(POLL_STATION_ID, VOTES_KEY);
        let votes: Vec<QuestionAnswer> = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(votes[0].result[0].votes, 0);
        assert_eq!(votes[0].result[1].votes, 1);
        assert_eq!(votes[1].result[0].votes, 1);
        assert_eq!(votes[1].result[1].votes, 0);
        assert_eq!(votes[1].result[2].votes, 1);
    }

    #[test]
    fn adding_vote_bad_token() {
        let ctx = prepare_full_env(VOTER_ID);
        let mut args = types::tests::create_test_vote_args();
        args.token = &[1, 2, 3, 4];

        let err = not_wasm::call_wrap(add_vote, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "bad token");
    }

    #[test]
    fn adding_vote_duplicate() {
        let ctx = prepare_full_env(VOTER_ID);
        let args = types::tests::create_test_vote_args();
        not_wasm::call_wrap(add_vote, ctx, args.clone()).unwrap();
        let ctx = not_wasm::create_app_context(POLL_STATION_ID, VOTER_ID);

        let err = not_wasm::call_wrap(add_vote, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "token already burned");
    }

    #[test]
    fn unauthorized_getting_vote_result() {
        let ctx = prepare_full_env("DummyAccount");

        let err = not_wasm::call_wrap(get_result, ctx, PackedValue::default()).unwrap_err();

        assert_eq!("not authorized", err.to_string());
    }

    #[test]
    fn authorized_getting_vote_result() {
        let ctx = prepare_full_env(VOTER_ID);
        let args = types::tests::create_test_vote_args();
        not_wasm::call_wrap(add_vote, ctx, args).unwrap();

        let ctx = not_wasm::create_app_context(POLL_STATION_ID, OWNER_ID);
        let pkt = not_wasm::call_wrap(get_result, ctx, PackedValue::default()).unwrap();

        let votes: Vec<QuestionAnswer> = trinci_sdk::rmp_deserialize(&pkt).unwrap();
        assert_eq!(votes[0].result[0].votes, 0);
        assert_eq!(votes[0].result[1].votes, 1);
        assert_eq!(votes[1].result[0].votes, 1);
        assert_eq!(votes[1].result[1].votes, 0);
        assert_eq!(votes[1].result[2].votes, 1);

        let buf = not_wasm::get_account_data(POLL_STATION_ID, CONFIG_KEY);
        let config: VoteConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!("CLOSED", config.status);
    }

    #[test]
    fn testing_check_duplicate_answer_to_question() {
        let array = vec![
            VoteArgsAnswer {
                id: "1",
                values: ["1", "2"].to_vec(),
            },
            VoteArgsAnswer {
                id: "2",
                values: ["3", "5"].to_vec(),
            },
        ];

        let res = check_duplicate_answer_to_question(&array);

        assert!(res.is_ok());
    }

    #[test]
    fn testing_failing_check_duplicate_answer_to_question() {
        let array = vec![
            VoteArgsAnswer {
                id: "5",
                values: ["1", "2"].to_vec(),
            },
            VoteArgsAnswer {
                id: "5",
                values: ["3", "5"].to_vec(),
            },
        ];

        let err = check_duplicate_answer_to_question(&array).unwrap_err();

        assert_eq!("duplicate preference", err.to_string());
    }

    #[test]
    fn testing_check_duplicate_vote() {
        let array = vec!["1", "2"];

        let res = check_duplicate_vote(&array);

        assert!(res.is_ok());
    }

    #[test]
    fn testing_failing_check_duplicate_vote() {
        let array = ["5", "5"];

        let err = check_duplicate_vote(&array).unwrap_err();

        assert_eq!("duplicate preference", err.to_string());
    }

    #[test]
    fn burned_ser_des() {
        let mut burned: HashSet<&str> = HashSet::new();
        burned.insert("Hello");
        burned.insert("World");
        let buf = trinci_sdk::rmp_serialize(&burned).unwrap();
        println!("{}", hex::encode(&buf));

        let des: HashSet<&str> = rmp_deserialize(&buf).unwrap();
        println!("{:?}", des);
    }
}
