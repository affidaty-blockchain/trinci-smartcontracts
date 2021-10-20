#!/bin/env node
const t2lib = require('@developer2/t2-lib');
const HashList = require('./include/hashlist').HashList;
const fileFromPath = require('./include/hashlist').fileFromPath;

// CONFIGS START
const nodeUrl = 'http://localhost:8000/';
const network = 'nightly';
// CONFIGS END

const c = new t2lib.Client(nodeUrl, network);
let hashList = new HashList();
cryptoHash = hashList.load('crypto');
console.log(`CRYPTO HASH: ${cryptoHash}`);
let cryptoAcc = new t2lib.Account();
let args1 = {
    depth: 3,
    root: '57642fbe91a4710080ef401b7f1f15aa77a16fea85a5f757deaa215e05b2733a',
    indices: [1,3],
    leaves: [
        'a3b3c1ae3b803953d5b89e67e25802a38f525fe8ff447914e67794f06ac6d87b',
        '40846390c66a0d1a77bd22513a2a51cd6a4ff597b1bec4d5e73bd8cf736e12b1',
    ],
    proofs: [
        '4b87b4a516220f0429483b18eaac96a332a5045a41e34346dd7a15489812d82b',
        '350f6198a9425ce4945837b9ae88863177e9c04dcea2c0be9acfbb86eb9a8d58',
        'e5fbae86ba26e1f9b3e16bd8761d88701451c45e2ba2686e75582b1ae3db41ad'
    ],
};
let args2 = {
    depth: 3,
    root: '57642fbe91a4710080ef401b7f1f15aa77a16fea85a5f757deaa215e05b2733a',
    indices: [0,1,2,3,4,5,6,7],
    leaves: [
        '350f6198a9425ce4945837b9ae88863177e9c04dcea2c0be9acfbb86eb9a8d58',
        'a3b3c1ae3b803953d5b89e67e25802a38f525fe8ff447914e67794f06ac6d87b',
        '4b87b4a516220f0429483b18eaac96a332a5045a41e34346dd7a15489812d82b',
        '40846390c66a0d1a77bd22513a2a51cd6a4ff597b1bec4d5e73bd8cf736e12b1',
        '2cdfdd8a76fdf4becab0f91884edd2ecdf0f348508956e2a43c344fdd4fcea38',
        '2cdfdd8a76fdf4becab0f91884edd2ecdf0f348508956e2a43c344fdd4fcea38',
        '2cdfdd8a76fdf4becab0f91884edd2ecdf0f348508956e2a43c344fdd4fcea38',
        '2cdfdd8a76fdf4becab0f91884edd2ecdf0f348508956e2a43c344fdd4fcea38'
    ],
    proofs: [],
}

async function init() {
    title('init');
    await cryptoAcc.generate();
    console.log(`CRYPTO      : ${cryptoAcc.accountId}`);
};

async function setAccount() {
    title('setAccount');
    let ticket = await c.prepareAndSubmitTx(
        cryptoAcc.accountId,
        cryptoHash,
        'merkle_tree_verify',
        args2,
        cryptoAcc.keyPair.privateKey,
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

async function main() {
    await init();
    await setAccount();
}

main();

function title(str) {
    console.log(`===================|${str}|===================`);
};