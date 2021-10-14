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

//! Basic Vote integration test

use integration::{
    common::{self, AccountInfo, PUB_KEY1, PUB_KEY2, PUB_KEY3, PVT_KEY1, PVT_KEY2, PVT_KEY3},
    TestApp,
};
use lazy_static::lazy_static;
use log::info;
use std::collections::HashMap;
use trinci_core::{
    base::serialize::{rmp_deserialize, rmp_serialize},
    crypto::Hash,
    Receipt, Transaction,
};
use trinci_sdk::value;

lazy_static! {
    pub static ref VOTE_APP_HASH: Hash = common::app_hash("basic_vote.wasm").unwrap();
}

const ADMIN_ALIAS: &str = "Admin";
const VOTER_ALIAS: &str = "Voter";
const POLL_STATION1_ALIAS: &str = "Station1";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(VOTER_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(ADMIN_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(POLL_STATION1_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map
    };
}

// Organization owner.
fn create_vote_data() -> common::SerdeValue {
    value!(
        {
            "answers": [
              {
                "id": "1",
                "values": [
                  "2"
                ]
              },
              {
                "id": "2",
                "values": [
                  "1",
                  "3"
                ]
              }
            ]
          }
    )
}

fn create_vote_config(admin: &AccountInfo) -> common::SerdeValue {
    value!({
        "title": {
          "en": "Best T2 character and spring break lunch"
        },
        "description": {
          "en": "Vote to decide the best character and lunch ..."
        },
        "status": "OPEN",
        "owner": admin.id,
        "rules": {
          "min": 2,
          "max": 2
        },
        "questions": [
          {
            "id": "1",
            "question": "Express your preference for the best T2 character",
            "rules": {
              "min": 1,
              "max": 1
            },
            "options": [
              {
                "id": "1",
                "question": "html",
                "value": "John Connor"
              },
              {
                "id": "2",
                "question": "html",
                "value": "The Terminator"
              }
            ]
          },
          {
            "id": "2",
            "question": "Express your preference for spring break lunch",
            "rules": {
              "min": 1,
              "max": 2
            },
            "options": [
              {
                "id": "1",
                "question": "html",
                "value": "Sushi"
              },
              {
                "id": "2",
                "question": "html",
                "value": "Salad"
              },
              {
                "id": "3",
                "question": "html",
                "value": "Pizza"
              }
            ]
          }
        ],
      }
    )
}

fn station_init_tx(
    admin: &AccountInfo,
    station: &AccountInfo,
    vote_config: common::SerdeValue,
) -> Transaction {
    common::create_test_tx(
        &station.id,
        &admin.pub_key,
        &admin.pvt_key,
        *VOTE_APP_HASH,
        "init",
        vote_config,
    )
}

fn station_config_tx(station: &AccountInfo, caller: &AccountInfo) -> Transaction {
    common::create_test_tx(
        &station.id,
        &caller.pub_key,
        &caller.pvt_key,
        *VOTE_APP_HASH,
        "get_config",
        value!(null),
    )
}

fn vote_tx(station_info: &AccountInfo, voter_info: &AccountInfo) -> Transaction {
    let args = create_vote_data();

    common::create_test_tx(
        &station_info.id,
        &voter_info.pub_key,
        &voter_info.pvt_key,
        *VOTE_APP_HASH,
        "add_vote",
        args,
    )
}

fn get_results_tx(admin: &AccountInfo, station: &AccountInfo) -> Transaction {
    let args = value!(null);

    common::create_test_tx(
        &station.id,
        &admin.pub_key,
        &admin.pvt_key,
        *VOTE_APP_HASH,
        "get_result",
        args,
    )
}

/// Open polling stations
fn create_open_stations_txs() -> Vec<Transaction> {
    let admin_info = ACCOUNTS_INFO.get(ADMIN_ALIAS).unwrap();
    let station_info = ACCOUNTS_INFO.get(POLL_STATION1_ALIAS).unwrap();
    let voter_info = ACCOUNTS_INFO.get(VOTER_ALIAS).unwrap();

    let config = create_vote_config(admin_info);

    vec![
        // 0. Open polling station
        station_init_tx(admin_info, station_info, config.clone()),
        // 1. Get polling station config from a voter account.
        station_config_tx(station_info, voter_info),
    ]
}

fn check_open_stations_rxs(rxs: Vec<Receipt>) {
    let admin_info = ACCOUNTS_INFO.get(ADMIN_ALIAS).unwrap();

    // 0.
    assert!(rxs[0].success);
    // 1.
    assert!(rxs[1].success);
    let config = create_vote_config(admin_info);
    let expected = rmp_serialize(&config).unwrap();
    let config: common::SerdeValue = rmp_deserialize(&rxs[1].returns).unwrap();
    let actual = rmp_serialize(&config).unwrap();
    assert_eq!(actual, expected);
}

// Insert some votes
fn create_voting_session_txs() -> Vec<Transaction> {
    let voter_info = ACCOUNTS_INFO.get(VOTER_ALIAS).unwrap();
    let station_info = ACCOUNTS_INFO.get(POLL_STATION1_ALIAS).unwrap();

    vec![
        // 0. Vote to the wrong station.
        vote_tx(station_info, voter_info),
        // 2. Duplicated vote. Should fail.
        vote_tx(station_info, voter_info),
    ]
}

fn check_voting_session_rxs(rxs: Vec<Receipt>) {
    // 0.
    assert!(rxs[0].success);
    // 1.
    assert!(!rxs[1].success);
    let msg = String::from_utf8_lossy(&rxs[1].returns);
    assert_eq!(msg, "smart contract fault: the caller has already voted");
}

// Close polling stations by getting the results.
fn create_close_stations_txs() -> Vec<Transaction> {
    let admin_info = ACCOUNTS_INFO.get(ADMIN_ALIAS).unwrap();
    let station_info = ACCOUNTS_INFO.get(POLL_STATION1_ALIAS).unwrap();

    vec![
        // 0. Get results from station.
        get_results_tx(admin_info, station_info),
    ]
}

fn check_close_stations_rxs(rxs: Vec<Receipt>) {
    // 0.
    assert!(rxs[0].success);
    let expected = rmp_serialize(&value!([
      {
        "id": "1",
        "result": [ { "id": "1", "votes": 0 }, { "id": "2", "votes": 1 } ]
      },
      {
        "id": "2",
        "result": [ { "id": "1", "votes": 1 }, { "id": "2", "votes": 0 }, { "id": "3", "votes": 1 } ]
      }
    ]))
    .unwrap();
    assert_eq!(rxs[0].returns, expected);
}

#[test]
fn voting_full_scenario() {
    let mut app = TestApp::default();

    info!("Open polling station");
    let txs = create_open_stations_txs();
    let rxs = app.exec_txs(txs);
    check_open_stations_rxs(rxs);

    info!("Voting session");
    let txs = create_voting_session_txs();
    let rxs = app.exec_txs(txs);
    check_voting_session_rxs(rxs);

    info!("Close polling stations");
    let txs = create_close_stations_txs();
    let rxs = app.exec_txs(txs);
    check_close_stations_rxs(rxs);
}
