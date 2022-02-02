#!/bin/env node

const fs = require('fs');
const t2lib = require('@affidaty/t2-lib');
const fileFromPath = require('./include/hashlist').fileFromPath;
const HashList = require('./include/hashlist').HashList;
const { argv } = require('process');
const hashList = new HashList();

// CONFIGS START
// const nodeUrl = 'http://t2.dev.trinci.net/0.2.3rc1/';
const nodeUrl = 'http://localhost:8000';
// const network = 'breakingnet';
const network = 'QmVkyfEaxPvEJVJLDK93VBq9GSda8oSXvLkey8oY6DdBNR';
const publishPrivKeyFilePath = '/home/alex/Scrivania/t2/local_data/keys/priv_key_admin.txt';

// List of files to publish. You can also pass a list of paths
// as args. they will be added to this list
const fileList = [
    '../../registry/crypto.wasm',
    '../arya/build/arya.wasm',
    '../bart/build/bart.wasm',
    // '../../registry/service.wasm',
]
// CONFIGS END

// appending passed paths to fileList array
if (argv.length > 2) {
    for (let i = 2; i < argv.length; i++) {
        fileList.push(argv[i]);
    }
}

const scriptDir = argv[1].substring(0, argv[1].lastIndexOf('/'));

let c = new t2lib.Client(nodeUrl, network);
const publisher = new t2lib.Account();

async function main() {
    const publisherPrivKeyB58 = fs.readFileSync(publishPrivKeyFilePath, {encoding: 'utf-8'}).trimEnd();
    const privKey = new t2lib.ECDSAKey('private');
    await privKey.setPKCS8(new Uint8Array(t2lib.binConversions.base58ToArrayBuffer(publisherPrivKeyB58)));
    await publisher.setPrivateKey(privKey);
    console.log(`PUBLISHER: ${publisher.accountId}`);
    for (let fileIdx = 0; fileIdx < fileList.length; fileIdx++) {
        if (fileList[fileIdx][0] !== '/') {
            fileList[fileIdx] = `${scriptDir}/${fileList[fileIdx]}`;
        }
        const scVersion = `${(new Date().toUTCString()).replace(/ /g, '_')}`;
        const scName = `${fileFromPath(fileList[fileIdx], true)}-${scVersion}`;
        const scDescription = `${fileFromPath(fileList[fileIdx], true)} smart contract`;
        const scUrl = 'https://affidaty.io/';
        const scBin = new Uint8Array(fs.readFileSync(fileList[fileIdx]));
        const tx = new t2lib.UnitaryTransaction();
        tx.data.accountId = c.serviceAccount;
        tx.data.maxFuel = 0;
        tx.data.genNonce();
        tx.data.networkName = c.network;
        tx.data.smartContractMethod = 'contract_registration';
        tx.data.smartContractMethodArgs = {
            name: scName,
            version: scVersion,
            description: scDescription,
            url: scUrl,
            bin: Buffer.from(scBin),
        };
        await tx.sign(publisher.keyPair.privateKey);
        // console.log(Buffer.from(await tx.toBytes()).toString('hex'));
        const ticket = await c.submitTx(tx);
        let receipt = await c.waitForTicket(ticket);
        let hashString;
        if (receipt.success) {
            hashString = t2lib.Utils.bytesToObject(receipt.result);
            hashList.save(fileFromPath(fileList[fileIdx], true), hashString);
        } else {
            hashString = Buffer.from(receipt.result).toString();
        }
        console.log(`${fileFromPath(fileList[fileIdx], true)}: ${hashString}`);
    }
}

main();
