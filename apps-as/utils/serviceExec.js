#!/bin/env node

const fs = require('fs');
const path = require('path');
const msgpack = require('msgpack-lite');
const { Uint64BE } = require("int64-buffer");
const t2lib = require('@affidaty/t2-lib');
const HashList = require('./include/hashlist').HashList;

// CONFIGS START
const nodeUrl = 'http://localhost:8000/';
// const nodeUrl = 'http://testnet.trinci.net/';
const network = 'QmY5BKcMiaUaS8CRN7et77hFHMXjUeTfF2aMWyXTp8jhpD';
// const network = 'QmYGc7fe885jCAdnNBJ24kniLvnp9WnqV3JHEjoMENyRRJ';
const client = new t2lib.Client(nodeUrl, network);

const adminPrivKeyFile = '/home/alex/git-projects/t2cli-js-sample-scripts/local_data/keys/priv_key_admin.txt';
// CONFIGS END

const adminAcc = new t2lib.Account();
const codec = msgpack.createCodec({int64: true});
const hashList = new HashList();
const scHash = hashList.load('service');

async function init() {
    const adminPrivKeyB58Bytes = fs.readFileSync(path.resolve(__dirname, adminPrivKeyFile));
    const adminPrivKey = new t2lib.ECDSAKey('private');
    await adminPrivKey.setPKCS8(new Uint8Array(t2lib.binConversions.base58ToArrayBuffer(adminPrivKeyB58Bytes.toString())))
    await adminAcc.setPrivateKey(adminPrivKey);
    return;
}

// async function initSc() {

//     let wasmFilePath = path.resolve(__dirname, '../service/build/service.wasm');
//     let wasmFileBin = fs.readFileSync(wasmFilePath);

//     const tx = new t2lib.UnitaryTransaction();
//     tx.data.accountId = 'TRINCI';
//     tx.data.maxFuel = 1000000;
//     tx.data.genNonce();
//     tx.data.networkName = client.network;
//     tx.data.smartContractHashHex = '';
//     tx.data.smartContractMethod = 'init';
//     tx.data.smartContractMethodArgsBytes = new Uint8Array(wasmFileBin);
//     await tx.sign(adminAcc.keyPair.privateKey);

//     const ticket = await client.submitTx(tx);
//     console.log(`init ticket: ${ticket}`);
//     // const rec = await client.waitForTicket(ticket);
//     // console.log(rec);
//     // if (!rec.success) {
//     //     console.log(`Error: ${Buffer.from(rec.result).toString()}`);
//     // }
// }

async function accBalance(accountId, assetName) {
    const accData = await client.accountData(accountId);
    const accBalance = msgpack.decode(accData.assets[assetName], {codec}).toString();
    console.log(`account balance : ${accBalance}`);
}

async function getScByName(scName) {
    const scList = await client.registeredContractsList();
    const hashes = Object.keys(scList);
    for (let i = 0; i < hashes.length ; i++) {
        if (scList[hashes[i]].name === scName) {
            return hashes[i];
        }
    }
    throw new Error(`Smart contract [${scName}] not found.`);
}

// async function mint() {
//     const newScHash = await getScByName('service');
//     const tx = new t2lib.UnitaryTransaction();
//     tx.data.accountId = 'TRINCI';
//     tx.data.maxFuel = 1000;
//     tx.data.genNonce();
//     tx.data.networkName = client.network;
//     tx.data.smartContractHashHex = '';
//     tx.data.smartContractMethod = 'contract_updatable';
//     tx.data.smartContractMethodArgs = [
//         randAcc.accountId,
//         Buffer.from('', 'hex'),
//         Buffer.from(newScHash, 'hex'),
//     ];
//     await tx.sign(randAcc.keyPair.privateKey);
//     console.log((await tx.toBytes()).byteLength);
//     const ticket = await client.submitTx(tx);
//     console.log(`ticket: ${ticket}`);
//     const rec = await client.waitForTicket(ticket, 100, 1000);
//     rec.resultObj = t2lib.Utils.bytesToObject(rec.result);
//     console.log(rec);
//     if (!rec.success) {
//         console.log(`Error: ${Buffer.from(rec.result).toString()}`);
//     }
// }

async function test() {

    const randAcc = new t2lib.Account();
    await randAcc.generate();

    const serviceScHash = await getScByName('service');
    const newScHash = await getScByName('4rya');
    console
    const tx = new t2lib.UnitaryTransaction();
    tx.data.accountId = 'TRINCI';
    tx.data.maxFuel = 1000;
    tx.data.genNonce();
    tx.data.networkName = client.network;
    tx.data.smartContractHashHex = serviceScHash;
    tx.data.smartContractMethod = 'contract_updatable';
    tx.data.smartContractMethodArgs = [
        randAcc.accountId,
        Buffer.from('ff', 'hex'),
        Buffer.from(newScHash, 'hex'),
    ];
    await tx.sign(adminAcc.keyPair.privateKey);
    // console.log(await tx.toUnnamedObject())
    // console.log(Buffer.from(await tx.toBytes()).toString('hex'));
    const ticket = await client.submitTx(tx);
    console.log(`ticket: ${ticket}`);
    const rec = await client.waitForTicket(ticket, 100, 1000);
    rec.resultObj = t2lib.Utils.bytesToObject(rec.result);
    console.log(rec);
    if (!rec.success) {
        console.log(`Error: ${Buffer.from(rec.result).toString()}`);
    }
}

async function main() {
    await init();
    await accBalance(adminAcc.accountId, 'TRINCI');
    // await mint();
    await test();
};

main();
