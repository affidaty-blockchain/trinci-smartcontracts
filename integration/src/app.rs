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

use crate::common;
use std::{
    path::PathBuf,
    sync::{Arc, Once},
    time::Duration,
};
use tempfile::TempDir;
use trinci_core::{
    base::serialize::{rmp_deserialize, rmp_serialize},
    blockchain::{BlockConfig, BlockService, IsValidator, Message},
    crypto::{
        drand::SeedSource,
        ecdsa::{CurveId as EcdsaCurveId, KeyPair as EcdsaKeyPair},
        Hash,
    },
    db::RocksDb,
    wm::WmLocal,
    Account, ErrorKind, KeyPair, Receipt, Transaction,
};

const PRIVATE_KEY_BYTES: &str = "d7d90e5214f69e1297ca555815eefb5a540353561cc45e3be0db6b605f9f337140fae80101bb04e627cecb74556dead3";
const PUBLIC_KEY_BYTES: &str = "045be4d28cc245b289885f7912f8ae1ef98463e5846646bcb211b445de2022cdcf22f08147fdd7f6276059c21af5b9b64d08339cd013b28d7cf898fd32e3d2fccfae8e421075f24d25d857dbc919ebfd09ccbc371f2ba77c5d7a7f6d62d856a24e";

pub fn create_keypair() -> KeyPair {
    let private_bytes = hex::decode(PRIVATE_KEY_BYTES).unwrap();
    let public_bytes = hex::decode(PUBLIC_KEY_BYTES).unwrap();
    KeyPair::Ecdsa(
        EcdsaKeyPair::new(EcdsaCurveId::Secp384R1, &private_bytes, &public_bytes).unwrap(),
    )
}

pub struct TestApp {
    pub block_svc: BlockService<RocksDb, WmLocal>,
    pub path: PathBuf,
}

impl Default for TestApp {
    fn default() -> Self {
        TestApp::new(&common::apps_path())
    }
}

static INIT: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
fn logger_setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

// All nodes are validator for the first block
fn is_validator_function_temporary() -> impl IsValidator {
    move |_account_id| Ok(true)
}

impl TestApp {
    pub fn new(apps_path: &str) -> Self {
        logger_setup();

        let path = TempDir::new().unwrap().into_path();
        let db = RocksDb::new(&path);

        let wasm_loader = common::wasm_fs_loader(apps_path);
        let wm = WmLocal::new(wasm_loader, 3);

        let keypair = create_keypair();
        let account_id = keypair.public_key().to_account_id();

        let config = BlockConfig {
            timeout: 1,
            threshold: 42,
            network: "skynet".to_owned(),
            keypair: Arc::new(keypair),
        };

        let is_validator = is_validator_function_temporary();

        let seed = Arc::new(SeedSource::new(
            "skynet".to_string(),
            vec![1, 2, 3],
            Hash::default(),
            Hash::default(),
            Hash::default(),
        ));

        let mut block_svc = BlockService::new(&account_id, is_validator, config, db, wm, seed);

        block_svc.start();

        TestApp { block_svc, path }
    }

    fn send_recv_packed(&self, requests: Vec<Message>) -> Vec<Message> {
        const RECV_TIMEOUT: Duration = Duration::from_secs(3600);
        let req_chan = self.block_svc.request_channel();
        let buf = rmp_serialize(&requests).unwrap();
        let res_chan = req_chan.send_sync(Message::Packed { buf }).unwrap();
        match res_chan.recv_timeout_sync(RECV_TIMEOUT) {
            Ok(Message::Packed { buf }) => rmp_deserialize(&buf).unwrap(),
            Ok(res) => panic!("Unexpected block service response: {:?}", res),
            Err(err) => panic!("Put transaction error: {}", err),
        }
    }

    fn put_transactions(&self, txs: Vec<Transaction>) -> Vec<Hash> {
        let requests: Vec<Message> = txs
            .into_iter()
            .map(|tx| Message::PutTransactionRequest { confirm: true, tx })
            .collect();

        let responses = self.send_recv_packed(requests);

        responses
            .into_iter()
            .map(|msg| match msg {
                Message::PutTransactionResponse { hash } => hash,
                _ => panic!("Unexpected response: {:?}", msg),
            })
            .collect()
    }

    fn get_receipts(&self, hashes: Vec<Hash>) -> Option<Vec<Receipt>> {
        let requests: Vec<Message> = hashes
            .into_iter()
            .map(|hash| Message::GetReceiptRequest { hash })
            .collect();

        let responses = self.send_recv_packed(requests);

        let not_ready = responses.iter().any(
            |msg| matches!(msg, Message::Exception(err) if err.kind == ErrorKind::ResourceNotFound),
        );
        if not_ready {
            return None;
        }

        let responses = responses
            .into_iter()
            .map(|msg| match msg {
                Message::GetReceiptResponse { rx } => rx,
                _ => panic!("Unexpected response: {:?}", msg),
            })
            .collect();
        Some(responses)
    }

    // Execute transactions set and wait for receipts.
    pub fn exec_txs(&mut self, txs: Vec<Transaction>) -> Vec<Receipt> {
        const MAX_TRIALS: usize = 32;
        const RETRY_PERIOD: Duration = Duration::from_secs(3);

        let hashes = self.put_transactions(txs);

        let mut max_trials = MAX_TRIALS;
        loop {
            match self.get_receipts(hashes.clone()) {
                Some(receipts) => return receipts,
                None => {
                    if max_trials >= 1 {
                        std::thread::sleep(RETRY_PERIOD);
                        max_trials -= 1;
                    } else {
                        panic!("Timeout waiting for receipts");
                    }
                }
            }
        }
    }

    pub fn account_data(&self, id: &str, key: &str) -> Option<Vec<u8>> {
        let chan = self.block_svc.request_channel();
        let req = Message::GetAccountRequest {
            id: id.to_owned(),
            data: vec![key.to_owned()],
        };
        let res = chan.send_sync(req).unwrap().recv_sync().unwrap();
        match res {
            Message::GetAccountResponse { acc: _, mut data } if !data.is_empty() => data[0].take(),
            _ => None,
        }
    }

    pub fn account(&self, id: &str) -> Option<Account> {
        let chan = self.block_svc.request_channel();
        let req = Message::GetAccountRequest {
            id: id.to_owned(),
            data: vec![],
        };
        let res = chan.send_sync(req).unwrap().recv_sync().unwrap();
        match res {
            Message::GetAccountResponse { acc, data: _ } => Some(acc),
            _ => None,
        }
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.block_svc.stop();
        std::fs::remove_dir_all(&self.path).unwrap_or_else(|err| {
            println!(
                "failed to remove temporary db folder '{:?}' ({})",
                self.path, err
            );
        });
    }
}
