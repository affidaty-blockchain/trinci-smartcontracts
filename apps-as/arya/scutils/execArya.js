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
aryaHash = hashList.load('arya');
console.log(`ARYA HASH: ${aryaHash}`);
let aryaAcc = new t2lib.Account();
let certifierAcc = new t2lib.Account();
let userAcc = new t2lib.Account();
let testData = {};

async function init() {
    title('init');
    await aryaAcc.generate();
    console.log(`ARYA      : ${aryaAcc.accountId}`);
    await certifierAcc.generate();
    console.log(`CERTIFIER : ${certifierAcc.accountId}`);
    await userAcc.generate();
    console.log(`USER      : ${userAcc.accountId}`);
};

async function setData() {
    title('setData');
    let testData = {
        name: 'John',
        // surname: 'Doe',
        sex: 'male',
        tel: '1634829548',
        email: 'john.doe@mail.net',
    };
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'set_profile_data',
        testData,
        userAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        await printAryaData(userAcc);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function setCert() {
    title('setCert');
    testData = {
        name: 'John',
        surname: 'Doe',
        sex: 'male',
        tel: '1634829548',
        email: 'john.doe@mail.net',
    };
    let cert = new t2lib.Certificate(testData);
    cert.create(['name', 'surname']);
    await cert.sign(certifierAcc.keyPair.privateKey);

    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        '',
        'set_certificate',
        {
            target: userAcc.accountId,
            key: 'main',
            certificate: Buffer.from(await cert.toBytes()),
        },
        certifierAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log('ARYA ASSET DATA ON TARGET:');
        await printAryaData(userAcc);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function verifyData() {
    title('verifyData');

    let personalData = {
        name: 'John',
        surname: 'Doe',
    }

    let myAcc = new t2lib.Account();
    await myAcc.generate();

    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        '',
        'verify_data',
        {
            target: userAcc.accountId,
            certificate: `${certifierAcc.accountId}:main`,
            data: personalData,
        },
        certifierAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log(t2lib.Utils.bytesToObject(receipt.result));
        // console.log('ARYA ASSET DATA ON TARGET:');
        // await printAryaData(userAcc);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function main() {
    await init();
    await setData();
    await setCert();
    await verifyData();
}

main();

function title(str) {
    console.log(`===================|${str}|===================`);
};

async function printAryaData(account) {
    let accData = await c.accountData(account);
    let aryaData = t2lib.Utils.bytesToObject(accData.assets[aryaAcc.accountId]);
    if (aryaData.profile) {
        aryaData.profile = t2lib.Utils.bytesToObject(aryaData.profile);
    }
    if (aryaData.certificates) {
        aryaData.certificates = t2lib.Utils.bytesToObject(aryaData.certificates);
    }
    console.log(`ARYA DATA ON ${account.accountId}:`);
    console.log(aryaData);
}