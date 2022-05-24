#!/bin/env node
const t2lib = require('@affidaty/t2-lib');
const HashList = require('./include/hashlist').HashList;

// CONFIGS START
// const nodeUrl = 'http://t2.dev.trinci.net/0.2.3rc1/';
const nodeUrl = 'http://localhost:8000';
// const network = 'breakingnet';
const network = 'QmVkyfEaxPvEJVJLDK93VBq9GSda8oSXvLkey8oY6DdBNR';
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
let certifierAcc2 = new t2lib.Account();
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
    await certifierAcc2.generate();
    console.log(`CERTIFIER2: ${certifierAcc2.accountId}`);
    await userAcc.generate();
    console.log(`USER      : ${userAcc.accountId}`);
};

async function initArya() {
    title('initArya');
    let ticket = await c.prepareAndSubmitTx(
        cryptoAcc.accountId,
        0,
        cryptoHash,
        'init',
        '',
        cryptoAcc.keyPair.privateKey,
    );
    let receipt = await c.waitForTicket(ticket);
    if (receipt.success) {
        let ticket2 = await c.prepareAndSubmitTx(
            aryaAcc.accountId,
            0,
            aryaHash,
            'init',
            {
                crypto: cryptoAcc.accountId,
            },
            aryaAcc.keyPair.privateKey,
        );
        console.log(`TICKET : ${ticket}`);
        let receipt2 = await c.waitForTicket(ticket2);
        console.log(`SUCCESS: ${receipt2.success}`);
        if (receipt2.success) {
            console.log(`RESULT : [${Buffer.from(receipt2.result).toString('hex')}]`);
            console.log(t2lib.Utils.bytesToObject((await c.accountData(aryaAcc.accountId, ['*'])).requestedData[0]));
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
        surname: 'Dow',
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
        console.log('Profile data:');
        let profileData = await c.accountData(aryaAcc, [`${userAcc.accountId}:profile_data`]);
        console.log(t2lib.Utils.bytesToObject(profileData.requestedData[0]));
        console.log('keys list:');
        let allData = await c.accountData(aryaAcc, ['*']);
        let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
        console.log(keysList);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function updateData() {
    title('setData(update)');
    let testData = {
        surname: 'Doe',
        testField: 'testString',
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
        console.log('Profile data:');
        let profileData = await c.accountData(aryaAcc, [`${userAcc.accountId}:profile_data`]);
        console.log(t2lib.Utils.bytesToObject(profileData.requestedData[0]));
        console.log('keys list:');
        let allData = await c.accountData(aryaAcc, ['*']);
        let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
        console.log(keysList);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}
async function removeData() {
    title('removeData');
    let args = [
        'testField',
    ];
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'remove_profile_data',
        args,
        userAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log('Profile data:');
        let profileData = await c.accountData(aryaAcc, [`${userAcc.accountId}:profile_data`]);
        console.log(t2lib.Utils.bytesToObject(profileData.requestedData[0]));
        console.log('keys list:');
        let allData = await c.accountData(aryaAcc, ['*']);
        let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
        console.log(keysList);
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
    cert.target = userAcc.accountId;
    await cert.sign(certifierAcc.keyPair.privateKey);
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'set_certificate',
        {
            key: 'main',
            certificate: Buffer.from(await cert.toBytes()),
        },
        userAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log('keys list:');
        let allData = await c.accountData(aryaAcc, ['*']);
        let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
        console.log(keysList);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function updateCert() {
    title('updateCert');
    testData = {
        name: 'John',
        surname: 'Doe',
        sex: 'male',
        tel: '1634829548',
        email: 'john.doe@mail.net',
    };
    cert = new t2lib.Certificate(testData);
    cert.create(['name', 'surname']);
    cert.target = userAcc.accountId;
    await cert.sign(certifierAcc.keyPair.privateKey);
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'set_certificate',
        {
            key: 'main',
            certificate: Buffer.from(await cert.toBytes()),
        },
        userAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log('keys list:');
        let allData = await c.accountData(aryaAcc, ['*']);
        let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
        console.log(keysList);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function setCert2() {
    title('setCert2');
    testData = {
        name: 'John',
        surname: 'Dow',
        sex: 'male',
        tel: '1634829548',
        email: 'john.doe@mail.net',
    };
    cert = new t2lib.Certificate(testData);
    cert.create(['name', 'surname']);
    cert.target = userAcc.accountId;
    await cert.sign(certifierAcc2.keyPair.privateKey);
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'set_certificate',
        {
            key: 'one',
            certificate: Buffer.from(await cert.toBytes()),
        },
        userAcc.keyPair.privateKey,
    );
    let receipt = await c.waitForTicket(ticket);
    if (receipt.success) {

        testData = {
            address: '123 Main Street, New York, NY 10030',
        };
        cert = new t2lib.Certificate(testData);
        cert.create(['address']);
        cert.target = userAcc.accountId;
        await cert.sign(certifierAcc2.keyPair.privateKey);
        let ticket2 = await c.prepareAndSubmitTx(
            aryaAcc.accountId,
            aryaHash,
            'set_certificate',
            {
                key: 'two',
                certificate: Buffer.from(await cert.toBytes()),
            },
            userAcc.keyPair.privateKey,
        );
        let receipt = await c.waitForTicket(ticket2);
        if (receipt.success) {
            console.log('keys list:');
            let allData = await c.accountData(aryaAcc, ['*']);
            let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
            console.log(keysList);
        } else {
            console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
        }

    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function fieldsCertified() {
    title('fieldsCertified');

    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'fields_certified',
        {
            target: userAcc.accountId,
            certifier: certifierAcc2.accountId,
            key: '',
            fields: ['email', 'address'],
        },
        certifierAcc.keyPair.privateKey,
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

async function removeCert() {
    title('removeCert');
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'remove_certificate',
        {
            target: userAcc.accountId,
            certifier: certifierAcc2.accountId,
            keys: ['*'],
        },
        userAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log('keys list:');
        let allData = await c.accountData(aryaAcc, ['*']);
        let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
        console.log(keysList);
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
        aryaHash,
        'verify_data',
        {
            target: userAcc.accountId,
            certifier: certifierAcc.accountId,
            key: 'main',
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
        // console.log(t2lib.Utils.bytesToObject(receipt.result));
        // console.log('ARYA ASSET DATA ON TARGET:');
        // await printAryaData(userAcc);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function verifyData2() {
    title('verifyData2');

    let personalData = {
        name: 'John',
        surname: 'Doe',
    }

    cert = new t2lib.Certificate(personalData);
    cert.target = certifierAcc.accountId;
    cert.create();
    await cert.sign(certifierAcc2.keyPair.privateKey);

    personalData = {
        name: 'John',
        surname: 'Dow',
    }

    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'verify_data',
        {
            data: personalData,
            certificate: Buffer.from(await cert.toBytes()),
        },
        certifierAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        // console.log(t2lib.Utils.bytesToObject(receipt.result));
        // console.log('ARYA ASSET DATA ON TARGET:');
        // await printAryaData(userAcc);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function setDelegation() {
    title('setDelegation');
    let d = new t2lib.Delegation();
    d.delegate = userAcc.accountId;
    d.network = network;
    d.target = cryptoAcc.accountId;
    d.capabilities = {
        '*': true,
        method1: false,
    }
    await d.sign(certifierAcc.keyPair.privateKey);
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'set_delegation',
        {
            delegation: Buffer.from(await d.toBytes()),
        },
        certifierAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log('keys list:');
        let allData = await c.accountData(aryaAcc, ['*']);
        let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
        console.log(keysList);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function setDelegation2() {
    title('setdelegation2');
    console.log(cryptoAcc.accountId);
    let d = new t2lib.Delegation();
    d.delegate = userAcc.accountId;
    d.network = network;
    d.target = cryptoAcc.accountId;
    d.capabilities = {
        '*': false,
        method1: true,
    }
    await d.sign(certifierAcc2.keyPair.privateKey);
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'set_delegation',
        {
            delegation: Buffer.from(await d.toBytes()),
        },
        certifierAcc2.keyPair.privateKey,
    );
    let receipt = await c.waitForTicket(ticket);
    if (receipt.success) {
        d.target = aryaAcc.accountId
        await d.sign(certifierAcc2.keyPair.privateKey);
        let ticket = await c.prepareAndSubmitTx(
            aryaAcc.accountId,
            '',
            'set_delegation',
            {
                delegation: Buffer.from(await d.toBytes()),
            },
            userAcc.keyPair.privateKey,
        );
        console.log(`TICKET : ${ticket}`);
        let receipt = await c.waitForTicket(ticket);
        console.log(`SUCCESS: ${receipt.success}`);
        if (receipt.success) {
            console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
            console.log('keys list:');
            let allData = await c.accountData(aryaAcc, ['*']);
            let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
            console.log(keysList);
        } else {
            console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
        }
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function removeDelegation() {
    title('removeDelegation');
    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'remove_delegation',
        {
            delegate: userAcc.accountId,
            delegator: certifierAcc2.accountId,
            targets: ['*'],
        },
        userAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log('keys list:');
        let allData = await c.accountData(aryaAcc, ['*']);
        let keysList = t2lib.Utils.bytesToObject(allData.requestedData[0]);
        console.log(keysList);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function verifyCapability() {
    title('verifyCapability');

    let ticket = await c.prepareAndSubmitTx(
        aryaAcc.accountId,
        aryaHash,
        'verify_capability',
        {
            delegate: userAcc.accountId,
            delegator: certifierAcc.accountId,
            target: cryptoAcc.accountId,
            method: 'method1',
            // '*': true,
            // method1: false,
        },
        userAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        // console.log(t2lib.Utils.bytesToObject(receipt.result));
        // console.log('ARYA ASSET DATA ON TARGET:');
        // await printAryaData(userAcc);
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

async function main() {
    await init();
    await initArya();
    // await setData();
    // await updateData();
    // await removeData();
    // await setCert();
    // await updateCert();
    // await setCert2();
    // await fieldsCertified();
    // await removeCert();
    // await verifyData();
    // await verifyData2();
    // await setDelegation();
    // await setDelegation2();
    // await removeDelegation();
    // await verifyCapability();
}

main();

function title(str) {
    console.log(`===================|${str}|===================`);
};