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

use lazy_static::lazy_static;
use serde::Serialize;
pub use serde_value::{value, Value as SerdeValue};
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use trinci_core::{
    base::{
        schema::{
            BulkTransaction, BulkTransactions, SignedTransaction, TransactionData,
            TransactionDataBulkNodeV1, TransactionDataBulkV1, UnsignedTransaction,
        },
        serialize::rmp_serialize,
    },
    crypto::{ecdsa, Hash, HashAlgorithm, KeyPair, PublicKey},
    wm::WasmLoader,
    Account, Error, ErrorKind, Transaction, TransactionDataV1,
};
use trinci_sdk::rmp_serialize_named;
pub use trinci_sdk::tai::{Asset, AssetLockArgs, LockPrivilege, LockType};

// Various keypairs used for testing

pub const PUB_KEY1: &str = "045936d631b849bb5760bcf62e0d1261b6b6e227dc0a3892cbeec91be069aaa25996f276b271c2c53cba4be96d67edcadd66b793456290609102d5401f413cd1b5f4130b9cfaa68d30d0d25c3704cb72734cd32064365ff7042f5a3eee09b06cc1";
pub const PVT_KEY1: &str = "818f1a16382f219b9284442687420caa12a60d8945c93dca6d28e81f1597e6d8abcec81a2dca0fe6eae838891c1b7157";

pub const PUB_KEY2: &str = "04755974cec8051cd19adb9f6a5daea99c768418a84f6a8e1a3c17e20b863b5e3372af75fdb1164288bcc6a85f54a781f0ad533dd722cf28437dfe763cf4d5e9ff2a862518609a0b41ba46dd6f3b9f03e4815047b5ffe2a03d1f4e6f42b2dbcca1";
pub const PVT_KEY2: &str = "4007db25c582d39d9912ef6095d9064bcb6b84211cf570b5dd95a10545dde27707ff4042708eae0f357b5c8bcbbfbddb";

pub const PUB_KEY3: &str = "0415cf93b220a8baca938323e0977db3b5a3ccfc1e02d41d92a00394776cfb03409946b22b29b1103bfe82ff9bd946f16d422045bf8ee6a3fc03e80deb10b8b163b13c521aebd943c799b67f26974932f8c3c9f836e069d354642ed9216beff000";
pub const PVT_KEY3: &str = "e5e86a167ddad2d28baa5b1792b3bb83ff366f57dada85d7f268f750a70bbb20d0c463ee7c71e669250efb44375735d1";

pub const PUB_KEY4: &str = "04cc560593ad5919f84458a9d37fa0c1856c40d9d8700825bd15aefac4edaceb66562f4590330c5e0ccffb1527752b4a014f6b1cc30d7248107e9578563ced6abe13dcd6b02af730c65525b10781cbf6c1dc19e78b919aa491ca65aae1f642f3f0";
pub const PVT_KEY4: &str = "f27a1ae0bb73af43e6a03426fb416ad75bb76e08de6b487b0761fd0578240603bae18d74213fac72b6e5cd12a9653a08";

pub const PUB_KEY5: &str = "04f709bc338386339551ea7a45b9ce73005fb27b49286ff4aa2e64c516a23c810e5e66e8c9a0d15f44e1cf79fcaf52903545f40f27573fe5328604e9f27c9760bcaf9778d3117a1dbcfd3b69bc6d5c94101e68a44f3efe4f4aee138a96f5d0db5d";
pub const PVT_KEY5: &str = "d8647351464ae5ad80d91a02537c015708f292711e1f76614e94958077bcbee3793799daa6cfcb7f5d6383e82a8a7429";

pub const PUB_KEY6: &str = "048cec345edb04d9cc78d24a100f637a9bf8ab26e1880a824d49b0bf03f2622ad570a62101c48184c7d998a8e91a9d129ff80ae225862ff28e620c77088452eea7166e2b4c4de36dae7c7881a0ef73bee8be39cd4cdc80e2363583e002dc5f23c2";
pub const PVT_KEY6: &str = "365059c2c2295a6323216a63bc811c57b2c88086e17445e83112d489bbaecf4050828ef0ce346722dbdfbecb0f0b79d6";

pub const PUB_KEY7: &str = "044d469f3a0d6b50ab570ae4724446a5730fd7311d399dcf3cb3542eb6773b0e571f63197255ae29c4f154c89962ca1e36b78a90295fa6a242973743a14685fe60b644c7923530edf0967f4f639a35fb5ed081e660b2606e5f1f394eaa75f2630c";
pub const PVT_KEY7: &str = "fa646fa1f6d3b876b0f57700d0134d11fd1913073092e23f3df753289db64a73cc7b8920a39136e697a01e677f4834b5";

pub const PUB_KEY8: &str = "044717583406373a9b47f564e6af4c28d9bc45b11da5de0fdfcd9928dab12eaacaedfabc7357565f2ecfa222f4b4e654a727397c3cad00a2af4c21defe5a0b403d3e62390b71633b203c268fd35ffe2e83fc7c602c2ae19274707a96f579e5439e";
pub const PVT_KEY8: &str = "f9a2619f076ca99870bb90b4faf63a9ddedc031b07a1f2ea82305b71dc43d040b64ff56af043c887a24f5c4148b15dad";

pub const PKCS8_01: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a4020101043072903f41e56843227c479c012994be571d7bf0cf083cb154a369c4008a8709b04f625ee86f281a4b7e4f445b52aaf043a00706052b81040022a16403620004677ab1f2832bfec85ec98b96854db0091b02421d0871ac56f0ff2388e9aad26c486df10a286fda1b2a6955ec283e601580a39cbbad059b778f3af5d4a5fd6b36a2d16bf66fac5630c4778e0cbc3efe025fb492bcb8245e9781c880314fa8aed4";
pub const PKCS8_02: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a402010104308054283d0eb1af7b1e8349e5f694208342762fcb0c9386cf1fa87f9985bb73a02cfeddccc26ed6fe15d8da019493537ba00706052b81040022a164036200041174cc62f48c0f105a54255291247617ab7983de0ae08c6a26afb28ce04cd76525fa48e75d7cef1e16e165c500e1aba6beb4c2fdbf2e8f8237c1075cb498c32e00d4ab4c37d8bf3ff4a359c8917519015bddf1038e0105fe928f74b31f4f9185";
pub const PKCS8_03: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a40201010430b0394ba13c3f916137a902f1b5e4ef6e1e2b2c8b7c3ef9a39eac68b6c52a3927b5ad0e0ff3ef00a0c551d3b954738e32a00706052b81040022a164036200048cfc38a74cfce0ed3f85d44fbefe14e20238049215573a59f4232098696679d616258041a176ba9f5188ad45c7a6b45b1352a87ac5f2987fc68b96fa63e432536c506dad03088888d70617152bea5107ba4ade8a69d030fd392000eb1a324038";
pub const PKCS8_04: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a40201010430caa9e7c1d5e9a6cd702e3f0c4e20adbf70abf318006c7c2369212b38915f466f5151fde7d8498ae031fd31b36f91c18da00706052b81040022a16403620004f426b3fcd49732a87ea836036d3ef68fb352e5d9c54699a8dd2baee44ccfcc017cfe9e8f4cca12037304d8026ae038124fd10cf339e49d4544dc4985ada714a357e2e199c0c84ba005d421f830018f9903bedf31b4653fd67ca53bdb683e63d9";
pub const PKCS8_05: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a402010104308c14091733a6a3d64a0467a4b905775aa9844b03c7d7f7791f11879132445a49e748c6b2aa240dc97aca0c4ab90ee841a00706052b81040022a16403620004d482fa861888be6a0ad580a6e64c459e427de0791d93e472b68cd4b281ebf8c544eff24898124d6618757e494cb75864f0e584e57559fa16ccc75f4aa57c549d2b1a252cad8358ae5e4a73f86a84edc778b1d15dc07acf67f70f6cb2f69c062e";
pub const PKCS8_06: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a40201010430faca9932e9679bbd45723963a88fcad2c39fbf2b53868b899e92bf6315cd725a505cd9aeb5fbbd8e85a99eb8c045bb4ca00706052b81040022a164036200041ef7e46cf1a09e0a761d27c85143ae268cc6fd91ae498629937cfe8b9a15b4c564de296305732017ba5509950d3480af40f00bcece98cc80004251abba818ce72807a94d7af5f385650ec7656b820b900df319aab230435fba3f31dba5a8811f";
pub const PKCS8_07: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a402010104300f7d3838b4e8743e72df2c6df7139ee06a8218c0c29a028391c344470e8905bf4b6b283c432702ca53bcc842bbe9c4faa00706052b81040022a1640362000413cf363d9c7d9cbc6a72878c1293f979a443520663ea4d9dd2fa5a442e9693958b36ea4d09c78391a595ef4997ce7bfe8d4a2fa46a850078033e2e1ebc05c9d0670577f15ce35eb7b9ecbbc27aac47f8e87181471b566188a060236123826364";
pub const PKCS8_08: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a402010104305e9e56c0d18bd775806d0561bbd1a4a72c587f3110d87e6645a44a0a71bbc7e77ae5bf381e6df0026406b46b08ee8a75a00706052b81040022a16403620004a4ec78e0fd5bf731028dd339567d782b200c38426af7cc7d06e4e89bbfc5d47109af0871b51d818ec47aa818e09cb377ede2d261bc8ac61bce4d2884a4635ad8db142d8865032230930ab53caf2f13290b4baca69dc8b1d3595a4e213826ea83";
pub const PKCS8_09: &str = "3081bf020100301006072a8648ce3d020106052b810400220481a73081a402010104308f9d5bb2af6c8e6418c3099d209c112db9d7dcc1f946cafa7c072077402bc60fe9d37336f7a9a59f89f3b21c022e65a7a00706052b81040022a16403620004aa70ba3a9ae56a5032fd88040c6db8bb02d37f308c399b972a60caf7e4c14f01fff240fee1fe64498145d903e9f02c505aa8e188a29fd78b7a2769a0ea9aa7cb7016465d9c5d27a0f7d669187aec1205ba9f605c7473745a9e1bab3d6c72a57f";

lazy_static! {
    pub static ref ASSET_APP_HASH: Hash = app_hash("asset.wasm").unwrap();
}

pub struct AccountInfo {
    pub id: String,
    pub pub_key: String,
    pub pvt_key: String,
}

impl AccountInfo {
    pub fn new(pub_key: &str, pvt_key: &str) -> Self {
        AccountInfo {
            id: p384_hex_key_to_account_id(pub_key),
            pub_key: pub_key.to_owned(),
            pvt_key: pvt_key.to_owned(),
        }
    }
}

fn build_registry_map(path: &str) -> HashMap<Hash, PathBuf> {
    let mut map: HashMap<Hash, PathBuf> = HashMap::new();

    let entries = std::fs::read_dir(path)
        .expect("read 'apps' registry")
        .map(|res| res.map(|e| e.path()))
        .collect::<std::result::Result<Vec<_>, std::io::Error>>()
        .expect("reading 'apps' registry");

    for filename in entries {
        if let Some("wasm") = filename.extension().and_then(|ext| ext.to_str()) {
            if let Some(hash) = file_hash(&filename) {
                map.insert(hash, filename);
            }
        }
    }
    map
}

pub fn wasm_fs_loader(path: &str) -> impl WasmLoader {
    let map = build_registry_map(path);
    move |hash| {
        let filename = map
            .get(&hash)
            .ok_or_else(|| Error::new_ext(ErrorKind::ResourceNotFound, "wasm not found"))?;
        std::fs::read(filename).map_err(|err| Error::new_ext(ErrorKind::Other, err))
    }
}

pub fn create_test_tx_data(
    id: &str,
    public_key: &str,
    contract_hash: Hash,
    method: &str,
    args: impl Serialize,
) -> TransactionData {
    static mut MYNONCE: u64 = 0;

    let args = rmp_serialize_named(&args).unwrap();

    let nonce = unsafe {
        MYNONCE += 1;
        MYNONCE.to_be_bytes().to_vec()
    };
    TransactionData::V1(TransactionDataV1 {
        account: id.to_string(),
        nonce,
        network: "skynet".to_string(),
        contract: Some(contract_hash),
        method: method.to_string(),
        caller: p384_hex_key_to_public_key(public_key),
        args,
        fuel_limit: 0,
    })
}

pub fn create_test_tx(
    id: &str,
    public_key: &str,
    private_key: &str,
    target: Hash,
    method: &str,
    args: impl Serialize,
) -> Transaction {
    let data = create_test_tx_data(id, public_key, target, method, args);
    let keypair = p384_hex_keypair_to_keypair(public_key, private_key);
    let buf = trinci_core::base::serialize::rmp_serialize(&data).unwrap();
    let signature = keypair.sign(&buf).unwrap();

    Transaction::UnitTransaction(SignedTransaction { data, signature })
}

pub fn create_bulk_root_tx(
    id: &str,
    public_key: &str,
    target: Hash,
    method: &str,
    args: impl Serialize,
) -> UnsignedTransaction {
    static mut MYNONCE: u64 = 0;

    let args = rmp_serialize_named(&args).unwrap();

    let nonce = unsafe {
        MYNONCE += 1;
        MYNONCE.to_be_bytes().to_vec()
    };

    let root_data = TransactionData::BulkRootV1(TransactionDataV1 {
        account: id.to_string(),
        fuel_limit: 0,
        nonce,
        network: "skynet".to_string(),
        contract: Some(target),
        method: method.to_string(),
        caller: p384_hex_key_to_public_key(public_key),
        args,
    });

    UnsignedTransaction { data: root_data }
}

fn create_transaction_data_bulk_node_v1(
    id: &str,
    public_key: PublicKey,
    target: Hash,
    method: &str,
    args: impl Serialize,
    depends_on: Hash,
) -> TransactionData {
    static mut MYNONCE: u64 = 0;

    let args = rmp_serialize_named(&args).unwrap();

    let nonce = unsafe {
        MYNONCE += 1;
        MYNONCE.to_be_bytes().to_vec()
    };

    TransactionData::BulkNodeV1(TransactionDataBulkNodeV1 {
        account: id.to_string(),
        fuel_limit: 0,
        nonce,
        network: "skynet".to_string(),
        contract: Some(target),
        method: method.to_string(),
        caller: public_key,
        args,
        depends_on,
    })
}

pub fn create_bulk_node_transaction(
    id: &str,
    public_key: &str,
    private_key: &str,
    target: Hash,
    method: &str,
    args: impl Serialize,
    depends_on: Hash,
) -> SignedTransaction {
    let keypair = p384_hex_keypair_to_keypair(public_key, private_key);

    let data = create_transaction_data_bulk_node_v1(
        id,
        keypair.public_key(),
        target,
        method,
        args,
        depends_on,
    );

    let buf = rmp_serialize(&data).unwrap();
    let signature = keypair.sign(&buf).unwrap();

    SignedTransaction { data, signature }
}

fn create_test_data_bulk(
    root: UnsignedTransaction,
    nodes: Vec<SignedTransaction>,
) -> TransactionData {
    TransactionData::BulkV1(TransactionDataBulkV1 {
        txs: BulkTransactions {
            root: Box::new(root),
            nodes: Some(nodes),
        },
    })
}

pub fn create_test_bulk_tx(
    public_key: &str,
    private_key: &str,
    root: UnsignedTransaction,
    nodes: Vec<SignedTransaction>,
) -> Transaction {
    let keypair = p384_hex_keypair_to_keypair(public_key, private_key);
    let data = create_test_data_bulk(root, nodes);
    let buf = rmp_serialize(&data).unwrap();
    let signature = keypair.sign(&buf).unwrap();

    Transaction::BulkTransaction(BulkTransaction { data, signature })
}

pub fn kp_create_test_tx_data(
    id: &str,
    public_key: PublicKey,
    contract_hash: Hash,
    method: &str,
    args: impl Serialize,
) -> TransactionData {
    static mut MYNONCE: u64 = 0;
    let args = rmp_serialize_named(&args).unwrap();

    let nonce = unsafe {
        MYNONCE += 1;
        MYNONCE.to_be_bytes().to_vec()
    };
    TransactionData::V1(TransactionDataV1 {
        account: id.to_string(),
        nonce,
        network: "skynet".to_string(),
        contract: Some(contract_hash),
        method: method.to_string(),
        caller: public_key,
        args,
        fuel_limit: 0,
    })
}

pub fn kp_create_test_tx(
    id: &str,
    keypair: &trinci_core::crypto::sign::KeyPair,
    target: Hash,
    method: &str,
    args: impl Serialize,
) -> Transaction {
    let public_key = keypair.public_key();
    let data = kp_create_test_tx_data(id, public_key, target, method, args);

    let buf = rmp_serialize(&data).unwrap();
    let signature = keypair.sign(&buf).unwrap();

    Transaction::UnitTransaction(SignedTransaction { data, signature })
}

pub fn create_default_account(id: &str) -> Account {
    Account::new(id, None)
}

/// Utility function to get a public key from hex bytes.
/// Key bytes are not checked by the implementation and are taken "as-is".
///
/// # Panics
///
/// Panics if the `key` string is not a valid hex string.
pub fn p384_hex_key_to_account_id(key: &str) -> String {
    let public_key_ecdsa = ecdsa::PublicKey {
        curve_id: ecdsa::CurveId::Secp384R1,
        value: hex::decode(key).unwrap(), // TODO: eventually remove the overall function.
    };
    public_key_ecdsa.to_account_id()
}

pub fn p384_hex_key_to_public_key(public_key: &str) -> PublicKey {
    let ecdsa_public_key = ecdsa::PublicKey {
        value: hex::decode(public_key).unwrap(),
        curve_id: ecdsa::CurveId::Secp384R1,
    };
    PublicKey::Ecdsa(ecdsa_public_key)
}

fn p384_hex_keypair_to_keypair(public_key: &str, private_key: &str) -> KeyPair {
    let public_bytes = hex::decode(public_key).unwrap();
    let private_bytes = hex::decode(private_key).unwrap();
    let ecdsa_keypair =
        ecdsa::KeyPair::new(ecdsa::CurveId::Secp384R1, &private_bytes, &public_bytes).unwrap();
    KeyPair::Ecdsa(ecdsa_keypair)
}

pub fn file_read(filename: &Path) -> Option<Vec<u8>> {
    let mut file = File::open(filename).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    Some(buf)
}

pub fn file_hash(filename: &Path) -> Option<Hash> {
    let buf = file_read(filename)?;
    Some(Hash::from_data(HashAlgorithm::Sha256, &buf))
}

pub fn apps_path() -> String {
    let mut path = std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string();
    if let Some("integration") = path.split('/').last() {
        path.push_str("/..");
    }
    path.push_str("/registry");
    path
}

pub fn app_path(name: &str) -> String {
    apps_path() + "/" + name
}

pub fn app_hash(name: &str) -> Option<Hash> {
    let filename = app_path(name);
    let path = Path::new(&filename);
    file_hash(path)
}

pub fn app_read(name: &str) -> Option<Vec<u8>> {
    let filename = app_path(name);
    let path = Path::new(&filename);
    file_read(path)
}
