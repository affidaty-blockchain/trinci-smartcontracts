#!/bin/env node

const fs = require('fs');
const t2lib = require('@affidaty/t2-lib');
const fileFromPath = require('./include/hashlist').fileFromPath;
const HashList = require('./include/hashlist').HashList;
const hashList = new HashList();

// CONFIGS START
const nodeUrl = 'http://localhost:8000';
const network = 'skynet';

const fileList = [
    './build/crypto.wasm',
    './build/arya.wasm'
]
// CONFIGS END

let c = new t2lib.Client(nodeUrl, network);
const publisher = new t2lib.Account();

async function main() {
    await publisher.generate();
    console.log(`PUBLISHER: ${publisher.accountId}`);
    for (let fileIdx = 0; fileIdx < fileList.length; fileIdx++) {
        const scVersion = `${(new Date().toUTCString()).replace(/ /g, '_')}`;
        const scName = `${fileFromPath(fileList[fileIdx], true)}-${scVersion}`;
        const scDescription = `${fileFromPath(fileList[fileIdx], true)} smart contract`;
        const scUrl = 'https://www.example.net/';
        let scBin = new Uint8Array(fs.readFileSync(fileList[fileIdx]));
        let tx = new t2lib.stdTxPrepareUnsigned.service.contract_registration(
            c.serviceAccount,
            network,
            {
                name: scName,
                description: scDescription,
                version: scVersion,
                url: scUrl,
                bin: scBin,
            }
        );
        let ticket = await c.signAndSubmitTx(tx, publisher.keyPair.privateKey);
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
