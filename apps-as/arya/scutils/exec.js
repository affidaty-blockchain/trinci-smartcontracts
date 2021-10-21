#!/bin/env node
const t2lib = require('@affidaty/t2-lib');
const HashList = require('./include/hashlist').HashList;

// CONFIGS START
const nodeUrl = 'http://localhost:8000/';
const network = 'nightly';
// CONFIGS END

const c = new t2lib.Client(nodeUrl, network);
let hashList = new HashList();
let aryaHash = hashList.load('arya');
let cryptoHash = hashList.load('crypto');
console.log(`ARYA HASH  : ${aryaHash}`);
console.log(`CRYPTO HASH: ${cryptoHash}`);
let aryaAcc = new t2lib.Account();
let cryptoAcc = new t2lib.Account();
let certifierAcc = new t2lib.Account();
let userAcc = new t2lib.Account();
let testData = {};
let cert = new t2lib.Certificate();

async function init() {
    title('init');
    await cryptoAcc.generate();
    console.log(`CRYPTO    : ${cryptoAcc.accountId}`);
    await aryaAcc.generate();
    console.log(`ARYA      : ${aryaAcc.accountId}`);
    await certifierAcc.generate();
    console.log(`CERTIFIER : ${certifierAcc.accountId}`);
    await userAcc.generate();
    console.log(`USER      : ${userAcc.accountId}`);
};

async function initArya() {
    title('initArya');

    let ticket = await c.prepareAndSubmitTx(
        cryptoAcc.accountId,
        cryptoHash,
        'merkle_tree_verify',
        {
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
        },
        userAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    console.log(receipt);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        let ticket2 = await c.prepareAndSubmitTx(
            aryaAcc.accountId,
            aryaHash,
            'init',
            {
                crypto: cryptoAcc.accountId,
            },
            userAcc.keyPair.privateKey,
        );
        console.log(`TICKET : ${ticket}`);
        let receipt2 = await c.waitForTicket(ticket2);
        console.log(`SUCCESS: ${receipt2.success}`);
        if (receipt2.success) {
            console.log(`RESULT : [${Buffer.from(receipt2.result).toString('hex')}]`);
        } else {
            console.log(`ERROR : ${Buffer.from(receipt2.result).toString()}`);
        }
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function setData() {
    title('setData');
    let testData = {
        name: 'John',
        surname: 'Doe',
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
    cert = new t2lib.Certificate(testData);
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
    let certWithBuffers = await cert.toObjectWithBuffers();

    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        '',
        'verify_data',
        {
            target: userAcc.accountId,
            certificate: `${certifierAcc.accountId}:main`,
            data: personalData,
            // multiproof: certWithBuffers.multiProof,
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
    await initArya();
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