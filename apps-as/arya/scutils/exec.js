#!/bin/env node
const t2lib = require('@affidaty/t2-lib');
const HashList = require('./include/hashlist').HashList;
const fileFromPath = require('./include/hashlist').fileFromPath;

// CONFIGS START
const nodeUrl = 'http://localhost:8000/';
const network = 'nightly';
// CONFIGS END

const c = new t2lib.Client(nodeUrl, network);

async function execArya() {
    let testData = {
        name: 'John',
        surname: 'Doe',
        sex: 'male',
        tel: '1634829548',
        email: 'john.doe@mail.net',
    }
    let certifier = new t2lib.Account
    await certifier.generate();
    let cert = new t2lib.Certificate(testData);
    cert.create(['name', 'surname']);
    await cert.sign(certifier.keyPair.privateKey);
    console.log(await cert.toUnnamedObject());

    let hashList = new HashList();
    let contractHash = hashList.load('arya');
    console.log(`SC HASH: ${contractHash}`);
    let testAcc = new t2lib.Account();
    await testAcc.generate();
    console.log(`CERTIFIER: ${certifier.accountId}`);
    console.log(`TARGET : ${testAcc.accountId}`);
    let ticket = await c.prepareAndSubmitTx(
        testAcc.accountId,
        contractHash,
        'set_certificate',
        {
            target: testAcc.accountId,
            key: 'test',
            certificate: Buffer.from(await cert.toBytes()),
        },
        testAcc.keyPair.privateKey,
    );
    console.log(`TICKET : ${ticket}`);
    let receipt = await c.waitForTicket(ticket);
    console.log(`SUCCESS: ${receipt.success}`);
    if (receipt.success) {
        console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
        console.log('ARYA ASSET DATA ON TARGET:');
        let accData = await c.accountData(testAcc);
        console.log(Buffer.from(accData.assets[testAcc.accountId]).toString('hex'));
    } else {
        console.log(`ERROR : ${Buffer.from(receipt.result).toString()}`);
    }
}

// async function execHash() {
//     console.log('====================HASH====================')
//     let hashList = new HashList();
//     let contractHash = hashList.load('crypto');
//     console.log(`SC HASH: ${contractHash}`);
//     let testAcc = new t2lib.Account();
//     await testAcc.generate();
//     console.log(`TARGET : ${testAcc.accountId}`);
//     let ticket = await c.prepareAndSubmitTx(
//         testAcc.accountId,
//         contractHash,
//         'hash',
//         {
//             algorithm: 0,
//             data: Buffer.from([0xff, 0xfa]),
//         },
//         testAcc.keyPair.privateKey,
//     );
//     console.log(`TICKET : ${ticket}`);
//     let receipt = await c.waitForTicket(ticket);
//     console.log(`SUCCESS: ${receipt.success}`);
//     if (receipt.success) {
//         console.log(`RESULT : [${Buffer.from(receipt.result).toString('hex')}]`);
//     } else {
//         console.log(`ERROR  : [${Buffer.from(receipt.result).toString()}]`);
//     }
//     console.log('========================')
// }

execArya();
// execHash();
