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

//! Vote integration test

use integration::{
    common::{
        self, AccountInfo, PUB_KEY1, PUB_KEY2, PUB_KEY3, PUB_KEY4, PVT_KEY1, PVT_KEY2, PVT_KEY3,
        PVT_KEY4,
    },
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
    pub static ref VOTE_APP_HASH: Hash = common::app_hash("vote.wasm").unwrap();
}

const ADMIN_ALIAS: &str = "Admin";
const VOTER_ALIAS: &str = "Voter";
const POLL_STATION1_ALIAS: &str = "Station1";
const POLL_STATION2_ALIAS: &str = "Station2";

lazy_static! {
    static ref ACCOUNTS_INFO: HashMap<&'static str, AccountInfo> = {
        let mut map = HashMap::new();
        map.insert(VOTER_ALIAS, AccountInfo::new(PUB_KEY1, PVT_KEY1));
        map.insert(ADMIN_ALIAS, AccountInfo::new(PUB_KEY2, PVT_KEY2));
        map.insert(POLL_STATION1_ALIAS, AccountInfo::new(PUB_KEY3, PVT_KEY3));
        map.insert(POLL_STATION2_ALIAS, AccountInfo::new(PUB_KEY4, PVT_KEY4));
        map
    };
}

// Organization owner.
fn create_vote_data() -> common::SerdeValue {
    let token = value!([[
        0x14, 0x82, 0xD7, 0x4E, 0x08, 0xE3, 0x8A, 0x58, 0x41, 0xA1, 0x16, 0xB8, 0x4E, 0xB0, 0xC5,
        0x19, 0x97, 0xB3, 0xC2, 0x66, 0x64, 0xFD, 0xC7, 0x18, 0x8F, 0x2B, 0xFC, 0x68, 0xA6, 0xFE,
        0x66, 0xAB, 0xB5, 0x0F, 0x88, 0x45, 0xA6, 0x3B, 0xA4, 0x8C, 0x0F, 0xDE, 0xA6, 0xDA, 0xE3,
        0x0B, 0x00, 0xB8, 0x22, 0x11, 0x71, 0x02, 0xFF, 0x08, 0x9B, 0x19, 0x3A, 0x4C, 0x67, 0x5B,
        0x47, 0xF7, 0xFA, 0x6B, 0xB3, 0xDC, 0x4F, 0x57, 0x6C, 0xEE, 0x00, 0x76, 0xE4, 0x10, 0x68,
        0x32, 0x12, 0xFE, 0x2D, 0x41, 0xD4, 0x99, 0x66, 0xDA, 0x93, 0x6B, 0x17, 0xAE, 0xDA, 0x64,
        0x7F, 0xDF, 0x4F, 0x73, 0x73, 0x0D, 0xA4, 0x75, 0xF2, 0x73, 0x59, 0xB9, 0x1C, 0xD9, 0x90,
        0x09, 0x5D, 0x00, 0x24, 0x09, 0xEE, 0x82, 0xBD, 0x1F, 0x19, 0x4F, 0xA6, 0xA5, 0xD5, 0xEB,
        0xE9, 0x4B, 0x30, 0x37, 0x5C, 0xB3, 0x96, 0xEF, 0x7F, 0xBC, 0x02, 0x7E, 0xFB, 0x3E, 0x11,
        0x9A, 0x79, 0xE0, 0x02, 0xBA, 0x67, 0x23, 0x14, 0xB7, 0x22, 0xE1, 0x77, 0x3A, 0x01, 0x3E,
        0x19, 0x13, 0xC2, 0xF1, 0x76, 0x92, 0xBE, 0x9E, 0x00, 0xCF, 0x40, 0x3C, 0xDC, 0x08, 0xC4,
        0x39, 0x5F, 0xC5, 0xF1, 0x22, 0x65, 0x6D, 0x40, 0xAA, 0x70, 0x2A, 0x47, 0x97, 0xE5, 0x5D,
        0x09, 0xA0, 0x0B, 0xC0, 0x3E, 0x0C, 0x82, 0x1A, 0x0E, 0xA7, 0xD6, 0x24, 0x8D, 0xC7, 0xDB,
        0x5F, 0x98, 0x32, 0xB4, 0xD6, 0xE0, 0xEF, 0xA7, 0xBC, 0x96, 0x6A, 0x40, 0x73, 0x55, 0x23,
        0x3F, 0x25, 0xB4, 0xF6, 0xCA, 0xAD, 0x30, 0x3A, 0x14, 0x18, 0xD7, 0xC5, 0xCA, 0x3A, 0x87,
        0x35, 0x58, 0xBA, 0xCF, 0x53, 0x42, 0x20, 0xAF, 0x5D, 0x3D, 0x4D, 0xC0, 0x0E, 0x3F, 0x6C,
        0x3C, 0x48, 0x25, 0x57, 0x41, 0x5E, 0x34, 0x16, 0xFB, 0x7D, 0xE2, 0x54, 0xC8, 0xE8, 0x11,
        0xB4
    ]]);
    value!(
        {
            "token": token,
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

fn create_vote_config(
    admin: &AccountInfo,
    station1: &AccountInfo,
    station2: &AccountInfo,
) -> common::SerdeValue {
    let station1_salt = value!([[
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f
    ]]);
    let station1_e = value!([[0x01, 0x00, 0x01]]);
    let station1_n = value!([[
        0xd0, 0xbf, 0xf1, 0x08, 0x34, 0xc1, 0x5c, 0xad, 0xce, 0xca, 0x81, 0x3f, 0x73, 0x58, 0x87,
        0xe9, 0x6d, 0xc1, 0xf8, 0x5f, 0x2d, 0x4e, 0x3b, 0x6b, 0x2a, 0x21, 0xa0, 0x38, 0x8e, 0xad,
        0x22, 0x98, 0x54, 0x2a, 0x96, 0x60, 0xdd, 0xf3, 0x83, 0xaf, 0x82, 0x6f, 0x21, 0x5d, 0x55,
        0xe7, 0x3f, 0xb5, 0xf7, 0xe4, 0x46, 0x0d, 0xa4, 0xe2, 0x36, 0xb8, 0x87, 0x36, 0x75, 0xcc,
        0x88, 0xe4, 0x8e, 0x4b, 0x48, 0xf6, 0x41, 0xb8, 0xf6, 0x50, 0x13, 0x5a, 0xb5, 0x00, 0x37,
        0x9b, 0x70, 0x5b, 0xc8, 0xe2, 0x85, 0x4c, 0xcc, 0x0b, 0x40, 0xb9, 0x41, 0x24, 0x62, 0x98,
        0x66, 0x8d, 0xaa, 0x59, 0x89, 0xad, 0x8d, 0xc4, 0xb0, 0xde, 0xee, 0xbb, 0x96, 0xa8, 0x4e,
        0x8d, 0x51, 0x44, 0x42, 0xa2, 0xa8, 0x7b, 0x0c, 0x7d, 0x12, 0x83, 0x31, 0x71, 0x97, 0xe5,
        0xc6, 0xe5, 0x29, 0x27, 0x13, 0x36, 0x25, 0x31, 0x48, 0xe1, 0xbf, 0xe2, 0x13, 0x48, 0xf2,
        0x6d, 0xef, 0xee, 0x7a, 0x26, 0x01, 0xbe, 0xd3, 0x20, 0x33, 0xba, 0xfd, 0xe8, 0x3c, 0x09,
        0xc0, 0x4d, 0xb8, 0x14, 0xbd, 0xb3, 0xd3, 0xc0, 0x73, 0x1e, 0x47, 0x98, 0x54, 0xeb, 0xf0,
        0x13, 0x4e, 0xd0, 0xec, 0x2f, 0xcb, 0xb6, 0xd3, 0xf6, 0x01, 0x53, 0x93, 0x8f, 0x6a, 0x45,
        0x58, 0x95, 0xc1, 0x25, 0x00, 0x14, 0xe2, 0xe9, 0x64, 0x61, 0x13, 0x99, 0x35, 0x2c, 0xe2,
        0x31, 0xb4, 0xea, 0x94, 0xe3, 0x6a, 0x82, 0xe7, 0x55, 0xf8, 0x95, 0x9d, 0x75, 0xf9, 0xb0,
        0x5e, 0xc4, 0xf9, 0x36, 0xee, 0x04, 0xf7, 0x2f, 0x6c, 0x4e, 0x63, 0xbb, 0x32, 0x2b, 0xe5,
        0x8b, 0xf8, 0x39, 0xcf, 0x11, 0xf1, 0x2e, 0xda, 0xa5, 0x4d, 0xf2, 0x64, 0xf4, 0x07, 0x7a,
        0x4e, 0x3b, 0x13, 0xd4, 0xb3, 0xb3, 0x30, 0x84, 0xe9, 0x26, 0x6a, 0x86, 0x26, 0x45, 0x2f,
        0x1b
    ]]);

    value!({
        "title": {
          "en": "Best T2 character and spring break lunch"
        },
        "description": {
          "en": "Vote to decide the best character and lunch ..."
        },
        "start": 1619827200,
        "end": 1622505600,
        "status": "OPEN",
        "anonymous": true,
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
        "polling_stations": [
          {
            "id": station1.id,
            "uri": "https://polling.it/submit",
            "salt": station1_salt,
            "pk_rsa": {
                "e": station1_e,
                "n": station1_n,
            }
          },
          {
            "id": station2.id,
            "uri": "https://polling.eu/submit",
            "salt": station1_salt,
            "pk_rsa": {
                "e": station1_e,
                "n": station1_n,
            }
          },
        ]
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
    let station1_info = ACCOUNTS_INFO.get(POLL_STATION1_ALIAS).unwrap();
    let station2_info = ACCOUNTS_INFO.get(POLL_STATION2_ALIAS).unwrap();
    let voter_info = ACCOUNTS_INFO.get(VOTER_ALIAS).unwrap();

    let config = create_vote_config(admin_info, station1_info, station2_info);

    vec![
        // 0. Open polling station 1.
        station_init_tx(admin_info, station1_info, config.clone()),
        // 1. Open polling station 2.
        station_init_tx(admin_info, station2_info, config),
        // 2. Get polling station 1 config from a voter account.
        station_config_tx(station1_info, voter_info),
    ]
}

fn check_open_stations_rxs(rxs: Vec<Receipt>) {
    let admin_info = ACCOUNTS_INFO.get(ADMIN_ALIAS).unwrap();
    let station1_info = ACCOUNTS_INFO.get(POLL_STATION1_ALIAS).unwrap();
    let station2_info = ACCOUNTS_INFO.get(POLL_STATION2_ALIAS).unwrap();

    // 0.
    assert!(rxs[0].success);
    // 1.
    assert!(rxs[1].success);
    // 2.
    assert!(rxs[2].success);
    let config = create_vote_config(admin_info, station1_info, station2_info);
    let expected = rmp_serialize(&config).unwrap();
    let config: common::SerdeValue = rmp_deserialize(&rxs[2].returns).unwrap();
    let actual = rmp_serialize(&config).unwrap();
    assert_eq!(actual, expected);
}

// Insert some votes
fn create_voting_session_txs() -> Vec<Transaction> {
    let voter_info = ACCOUNTS_INFO.get(VOTER_ALIAS).unwrap();
    let station1_info = ACCOUNTS_INFO.get(POLL_STATION1_ALIAS).unwrap();
    let station2_info = ACCOUNTS_INFO.get(POLL_STATION2_ALIAS).unwrap();

    vec![
        // 0. Vote to the wrong station.
        vote_tx(station1_info, voter_info),
        // 1. Vote to the correct station.
        vote_tx(station2_info, voter_info),
        // 2. Duplicated vote. Should fail.
        vote_tx(station2_info, voter_info),
    ]
}

fn check_voting_session_rxs(rxs: Vec<Receipt>) {
    // 0.
    assert!(!rxs[0].success);
    let msg = String::from_utf8_lossy(&rxs[0].returns);
    assert_eq!(msg, "smart contract fault: wrong polling station");
    // 1.
    assert!(rxs[1].success);
    // 2.
    assert!(!rxs[2].success);
    let msg = String::from_utf8_lossy(&rxs[2].returns);
    assert_eq!(msg, "smart contract fault: token already burned");
}

// Close polling stations by getting the results.
fn create_close_stations_txs() -> Vec<Transaction> {
    let admin_info = ACCOUNTS_INFO.get(ADMIN_ALIAS).unwrap();
    let station1_info = ACCOUNTS_INFO.get(POLL_STATION1_ALIAS).unwrap();
    let station2_info = ACCOUNTS_INFO.get(POLL_STATION2_ALIAS).unwrap();

    vec![
        // 0. Get results from station 1.
        get_results_tx(admin_info, station1_info),
        // 1. Get results from station 2.
        get_results_tx(admin_info, station2_info),
    ]
}

fn check_close_stations_rxs(rxs: Vec<Receipt>) {
    // 0.
    assert!(rxs[0].success);
    // 1.
    assert!(rxs[1].success);
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
    assert_eq!(rxs[1].returns, expected);
}

#[test]
fn voting_full_scenario() {
    let mut app = TestApp::default();

    info!("Open polling stations");
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
