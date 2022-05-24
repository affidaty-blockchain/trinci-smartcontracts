#!/bin/env node
const { Uint64BE } = require("int64-buffer");
const t2lib = require('@affidaty/t2-lib');
const HashList = require('./include/hashlist').HashList;

// CONFIGS START
const nodeUrl = 'http://localhost:8000';
const network = 'QmZxyHgnfBxiD5joCfjn6uBsgyzsYuVjMVezdW1btb9Qw3';
// CONFIGS END

function title(str) {
    console.log(`===================|${str}|===================`);
};

const client = new t2lib.Client(nodeUrl, network);
let hashList = new HashList();
let kronoHash = hashList.load('krono');
console.log(`KRONO HASH: ${kronoHash}`);
let kronoAcc = new t2lib.Account();
let oracleAcc1 = new t2lib.Account();
let oracleAcc2 = new t2lib.Account();
let userAcc = new t2lib.Account();

async function init() {
    title('init');
    await kronoAcc.generate();
    console.log(`KRONO : ${kronoAcc.accountId}`);
    const cronoKey = t2lib.binConversions.arrayBufferToBase58((await kronoAcc.keyPair.privateKey.getPKCS8()).buffer);
    console.log(cronoKey);
    await oracleAcc1.generate();
    console.log(`ORACLE_1 : ${oracleAcc1.accountId}`);
    await oracleAcc2.generate();
    console.log(`ORACLE_2 : ${oracleAcc2.accountId}`);
    await userAcc.generate();
    console.log(`USER : ${userAcc.accountId}`);
};

async function initKrono() {
    title('initKrono');
    let ticket = await client.prepareAndSubmitTx(
        kronoAcc.accountId,
        0,
        kronoHash,
        'init',
        '',
        kronoAcc.keyPair.privateKey,
    );
    let receipt = await client.waitForTicket(ticket);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log(t2lib.Utils.bytesToObject((await client.accountData(kronoAcc.accountId, ['*'])).requestedData[0]));
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function setIsoTime() {
    title('setTime');
    let ticket = await client.prepareAndSubmitTx(
        kronoAcc.accountId,
        0,
        kronoHash,
        'set_unix_timestamp',
        {
            time: new Uint64BE(1000000000),
        },
        kronoAcc.keyPair.privateKey,
    );
    let receipt = await client.waitForTicket(ticket);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log(t2lib.Utils.bytesToObject((await client.accountData(kronoAcc.accountId, ['*'])).requestedData[0]));
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function mint() {
    title('mint');
    let ticket = await client.prepareAndSubmitTx(
        kronoAcc.accountId,
        0,
        kronoHash,
        'mint',
        {
            to: kronoAcc.accountId,
            units: 10000000,
        },
        kronoAcc.keyPair.privateKey,
    );
    let receipt = await client.waitForTicket(ticket);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        const accData = await client.accountData(kronoAcc.accountId, ['*', 'asset:stats']);
        console.log(t2lib.Utils.bytesToObject(accData.requestedData[0]))
        console.log(t2lib.Utils.bytesToObject(accData.requestedData[1]));
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function getIsoTime() {
    title('getIsoTime');
    let ticket = await client.prepareAndSubmitTx(
        kronoAcc.accountId,
        0,
        kronoHash,
        'get_unix_timestamp',
        '',
        kronoAcc.keyPair.privateKey,
    );
    let receipt = await client.waitForTicket(ticket);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log(t2lib.Utils.bytesToObject(receipt.result));
        const accData = await client.accountData(kronoAcc.accountId, ['*', 'asset:stats']);
        console.log(t2lib.Utils.bytesToObject(accData.requestedData[0]));
        console.log(t2lib.Utils.bytesToObject(accData.requestedData[1]));
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function burn() {
    title('burn');
    let ticket = await client.prepareAndSubmitTx(
        kronoAcc.accountId,
        0,
        kronoHash,
        'burn',
        {
            from: kronoAcc.accountId,
            units: 5,
        },
        kronoAcc.keyPair.privateKey,
    );
    let receipt = await client.waitForTicket(ticket);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        const accData = await client.accountData(kronoAcc.accountId, ['*', 'asset:stats']);
        console.log(t2lib.Utils.bytesToObject(accData.requestedData[0]))
        console.log(t2lib.Utils.bytesToObject(accData.requestedData[1]));
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function main() {
    await init();
    await initKrono();
    await setIsoTime();
    await mint();
    await getIsoTime();
    await burn();
}

main();
