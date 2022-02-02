#!/bin/env node
const t2lib = require('@affidaty/t2-lib');
const HashList = require('./include/hashlist').HashList;

// CONFIGS START
const nodeUrl = 'http://localhost:8000/';
const network = 'skynet';
// CONFIGS END

const c = new t2lib.Client(nodeUrl, network);
let hashList = new HashList();
let scHash = hashList.load('scName');
console.log(`SC HASH  : ${scHash}`);
let newAcc = new t2lib.Account();

async function init() {
    title('init');
    await newAcc.generate();
    console.log(`NEW ACC: ${newAcc.accountId}`);
};

async function sendTx() {
    title('sendTx');
    let args = {};
    let ticket = await c.prepareAndSubmitTx(
        newAcc.accountId,
        scHash,
        'new_method',
        args,
        newAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

function title(str) {
    console.log(`===================|${str}|===================`);
};

async function main() {
    await init();
    await sendTx();
}

main();