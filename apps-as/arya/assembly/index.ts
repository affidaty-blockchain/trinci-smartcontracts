// This file is part of TRINCI.
//
// Copyright (C) 2021 Affidaty Spa.
//
// TRINCI is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// TRINCI is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with TRINCI. If not, see <https://www.gnu.org/licenses/>.

import { Types, Utils, MemUtils, HostFunctions, MsgPack } from '../node_modules/@affidaty/trinci-sdk-as';
import {
    Certificate,
    Delegation,
    SetCertArgs,
    RemoveCertArgs,
    MerkleTreeVerifyArgs,
    SetDelegationArgs,
    RemoveDelegArgs,
    VerifyCapabilityArgs,
    ImportArgs,
} from './types';
import {
    profileDataDecode,
    profileDataEncode,
    RemoveProfileDataArgsDecode,
    decodeCertificate,
    certDataEncodeForVerify,
    decodeVerifyDataArgs,
    decodeDelegation,
    delegDataEncodeForVerify,
    decodeFieldsCertifiedArgs,
} from './msgpack';
import {
    arrayBufferToHexString,
    rawPubKeyToAccountId,
} from './utils';
import { retCodes } from './retcodes';

const cryptoAccountId: string = '#CRYPTO';

const settingsSectionKey: string = 'settings';
const ownerAccountKey: string = `${settingsSectionKey}:owner`;


export function alloc(size: i32): i32 {
    return heap.alloc(size) as i32;
}

export function run(ctxAddress: i32, ctxSize: i32, argsAddress: i32, argsSize: i32): Types.TCombinedPtr {
    let ctxU8Arr: u8[] = MemUtils.u8ArrayFromMem(ctxAddress, ctxSize);
    let ctx = MsgPack.ctxDecode(ctxU8Arr);
    let argsU8: u8[] = MemUtils.u8ArrayFromMem(argsAddress, argsSize);
    let methodsMap = new Map<string, (ctx: Types.AppContext, args: u8[])=>Types.TCombinedPtr>();

    methodsMap.set('init', init);
    methodsMap.set('set_profile_data', setProfileData);
    methodsMap.set('remove_profile_data', removeProfileData);
    methodsMap.set('set_certificate', setCertificate);
    methodsMap.set('remove_certificate', removeCertificate);
    methodsMap.set('verify_data', verifyData);
    methodsMap.set('fields_certified', fieldsCertified);
    methodsMap.set('set_delegation', setDelegation);
    methodsMap.set('remove_delegation', removeDelegation);
    methodsMap.set('verify_capability', verifyCapability);
    methodsMap.set('import_data', importData);

    if (!methodsMap.has(ctx.method)) {
        let success = false;
        let resultBytes = Utils.stringtoU8Array('Method not found.');
        return MsgPack.appOutputEncode(success, resultBytes);
    }

    return methodsMap.get(ctx.method)(ctx, argsU8);
}

function setOwner(owner: string): void {
    if (owner.length < 1) {
        return;
    }
    HostFunctions.storeData(ownerAccountKey, Utils.stringtoU8Array(owner));
    return;
}

function getOwner(): string {
    let result: string = '';
    const resultBytes = HostFunctions.loadData(ownerAccountKey);
    if (resultBytes.length > 0) {
        result = Utils.u8ArrayToString(resultBytes);
    }
    return result;
}

function init(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    setOwner(ctx.origin);
    return MsgPack.appOutputEncode(true, [0xc0]);
}



// PROFILE DATA MANAGEMENT - BEGIN

const profileDataSectionKey = 'profile_data';

function loadProfileData(account: string): Map<string, string> {
    let result = new Map<string, string>();
    let fullDataKey =  `${account}:${profileDataSectionKey}`;
    let profileDataBytes = HostFunctions.loadData(fullDataKey);
    if (profileDataBytes.length > 0) {
        result = profileDataDecode(profileDataBytes);
    }
    return result;
}

function saveProfileData(account: string, profileData: Map<string, string>): void {
    let fullDataKey =  `${account}:${profileDataSectionKey}`;
    let profileBytes = profileDataEncode(profileData);
    HostFunctions.storeData(fullDataKey, profileBytes);
}

function setProfileData(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [];
    if (argsU8.length > 0) {
        let profileData = loadProfileData(ctx.origin);
        let newProfileDataMap = profileDataDecode(argsU8);
        let newProfileDataKeys = newProfileDataMap.keys();
        for (let i = 0; i < newProfileDataKeys.length; i++) {
            profileData.set(newProfileDataKeys[i], newProfileDataMap.get(newProfileDataKeys[i]));
        }
        saveProfileData(ctx.origin, profileData);
        success = true;
        resultBytes = [0xC0];
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}

function removeProfileData(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [];
    if (argsU8.length > 0) {
        let profileData = loadProfileData(ctx.origin);
        let profileKeysToDelete = RemoveProfileDataArgsDecode(argsU8);
        if (profileKeysToDelete.indexOf('*') == -1) {
            for (let i = 0; i < profileKeysToDelete.length; i++) {
                if (profileData.has(profileKeysToDelete[i])) {
                    profileData.delete(profileKeysToDelete[i]);
                }
            }
        } else {
            profileData.clear();
        }
        saveProfileData(ctx.origin, profileData);
        success = true;
        resultBytes = [0xc0];
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}
// PROFILE DATA MANAGEMENT - END

// CERTIFICATES MANAGEMENT - BEGIN

const certificatesSectionKey = 'certificates';

function getCertsList(account: string, certifiers: string[] = []): string[] {
    const result: string[] = [];
    const prefix: string = `${account}:${certificatesSectionKey}`;
    const keys = HostFunctions.getKeys(`${prefix}:*`);
    for (let keyIdx = 0; keyIdx < keys.length; keyIdx++) {
        if (certifiers.length > 0) {
            for (let certifierIdx = 0; certifierIdx < certifiers.length; certifierIdx++) {
                if (keys[keyIdx].substring(prefix.length + 1).startsWith(certifiers[certifierIdx])) {
                    result.push(keys[keyIdx]);
                    break;
                }
            }
        } else {
            result.push(keys[keyIdx]);
        }
    };
    return result;
}

function saveCertBytes(target: string, certifier: string, key: string, certificate: u8[]): void {
    let certPrefix = `${target}:${certificatesSectionKey}`;
    let fullCertkey = `${certPrefix}:${certifier}:${key}`;
    HostFunctions.storeData(fullCertkey, certificate);
}

function setCertificate(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [0xc0];
    if (argsU8.length > 0) {
        let args = MsgPack.deserialize<SetCertArgs>(argsU8);
        if (args.key == '*' || args.key.length <= 0) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('This key cannot be used.'));
        }
        let certificate = decodeCertificate(args.certificate);
        let dataToVerify = certDataEncodeForVerify(certificate.data);
        let valid = HostFunctions.verify(certificate.data.certifier, dataToVerify, Utils.arrayBufferToU8Array(certificate.signature));
        if (!valid) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Invalid certificate signature.'));
        }
        let target = certificate.data.target;
        let certifier = rawPubKeyToAccountId(certificate.data.certifier.value);
        if (ctx.origin != target && ctx.origin != certifier) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Method available only for certificate target or certifier.'));
        }
        saveCertBytes(target, certifier, args.key, Utils.arrayBufferToU8Array(args.certificate));
        success = true;
        resultBytes = [0xc0];
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}

function removeCertificate(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [];
    if (argsU8.length > 0) {
        let args = MsgPack.deserialize<RemoveCertArgs>(argsU8);
        if (
            (
                ctx.origin == args.target
                || ctx.origin == args.certifier
            )
            && args.keys.length > 0
        ) {
            let certsList = getCertsList(args.target, [args.certifier]);
            let deleteAllCerts = args.keys.indexOf('*') >= 0;
            let certsToDelete: string[] = [];
            if (deleteAllCerts) {
                certsToDelete = certsList;
            } else {
                let prefix = `${args.target}:${certificatesSectionKey}:${args.certifier}:`;
                for (let certIdx = 0; certIdx < certsList.length; certIdx++) {
                    if (args.keys.indexOf(certsList[certIdx].substring(prefix.length)) >= 0) {
                        certsToDelete.push(certsList[certIdx]);
                    };
                };
            };
            for (let certIdx = 0; certIdx < certsToDelete.length; certIdx++) {
                HostFunctions.removeData(certsToDelete[certIdx]);
            };
            success = true;
            resultBytes = [0xc0];
        } else {
            resultBytes = Utils.stringtoU8Array('Not allowed');
        }
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}

// create a leaf from data
function makeLeaf(key: string, value: string, salt: ArrayBuffer): ArrayBuffer {
    let strToHash = `${value}${key}${arrayBufferToHexString(salt)}`;
    let strBin: u8[] = Utils.stringtoU8Array(strToHash);
    const hash: u8[] = HostFunctions.sha256(strBin);
    return Utils.u8ArrayToArrayBuffer(hash);
}

// calculates minimal tree depth needed to host given leaves number
function calculateSymmetryDepth(givenLeaves: u32): u32 {
    return Math.ceil(Math.log2(givenLeaves as f32)) as u32;
}

// calculates number of missing leaves needed to create a SYMMETRICAL tree
// with minimal depth needed to host given leaves number
function missingSymmetryLeaves(givenLeaves: u32): u32 {
    return (2 ** calculateSymmetryDepth(givenLeaves)) - givenLeaves;
}

function verifyData(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    const falseResultValue: u8[] = [0xc2];
    const trueResultValue: u8[] = [0xc3];
    let args = decodeVerifyDataArgs(argsU8);

    let certificate = new Certificate();
    let passedCert = false;
    // try to load the correct certificate
    // if a certificate has been passed, ignore
    // everything else and use it directly
    let certBytes: u8[] = [];
    if (args.certificate.byteLength > 0) {
        certBytes = Utils.arrayBufferToU8Array(args.certificate);
        passedCert = true;
    } else {
        certBytes = HostFunctions.loadData(`${args.target}:${certificatesSectionKey}:${args.certifier}:${args.key}`);
    }
    if (certBytes.length > 0) {
        certificate = decodeCertificate(Utils.u8ArrayToArrayBuffer(certBytes));
    }
    // if the returned certificate is empty (no signature and other data),
    // then no certificate with the given key was found
    if (certificate.signature.byteLength == 0) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.noCert.msg));
    }

    // if certificate was passed, then also check it's signature
    if (passedCert) {
        let dataToVerify = certDataEncodeForVerify(certificate.data);
        let valid = HostFunctions.verify(certificate.data.certifier, dataToVerify, Utils.arrayBufferToU8Array(certificate.signature));
        if (!valid) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Invalid certificate signature.'));
        }
    }

    if (args.target.length > 1) {
        if (args.target != certificate.data.target) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Wrong target in certificate.'));
        }
    }

    if (args.certifier.length > 1) {
        let certifier = rawPubKeyToAccountId(certificate.data.certifier.value);
        if (args.certifier != certifier) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Wrong certifier in certificate.'));
        }
    }

    // get target certificate
    // all fields certified by the certificate
    let certFields = certificate.data.fields.sort();
    // if any of the passed clear fields isn't present in the
    // certificate, then it cannot be verified. Therefore return false.
    for (let i = 0; i < args.data.keys().length; i++) {
        if (certFields.indexOf(args.data.keys()[i]) < 0) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.excessFields.msg));
        }
    }

    // certificate merkle tree depth
    let depth: u32 = calculateSymmetryDepth(certFields.length);
    // indexes of all fields with clear data relative to the certificate fields
    let clearIndexes: u32[] = [];
    // tree leaves (SHA256(`${value}${key}${salt.toStrinh('hex')}`) at positions specified in "clearIndexes" array.
    let clearLeaves: ArrayBuffer[] = [];

    // final compilation of clear data (args + eventual profile data)
    let clearData: Map<string, string> = new Map<string, string>();
    // fields that are missing in args
    let missingFields: string[] = [];
    // number of fields that should be generated automatically
    // to make the tree symmetrical (only if no multiproof, otherwise always 0)
    let autoLeavesNumber: u32 = 0;
    // find out if there are any missing field in args
    for (let i = 0; i < certFields.length; i++) {
        if (args.data.keys().indexOf(certFields[i]) >= 0) {
            clearData.set(certFields[i], args.data.get(certFields[i]));
        } else {
            missingFields.push(certFields[i]);
        }
    }

    // check profile data only if there are missing fields and no multiproof
    if (missingFields.length > 0 && args.multiproof.length < 1) {
        // load profile data
        let profileData = loadProfileData(args.target);

        for (let i = 0; i < missingFields.length; i++) {
            if (profileData.has(missingFields[i])) {
                clearData.set(missingFields[i], profileData.get(missingFields[i]));
            } else {
                // incomplete data without multiproof. Cannot verify.
                return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.missingData.msg));
            }
        }

        // at this point we have all data in clear
        // check if any leaves should be added automatically
        autoLeavesNumber = missingSymmetryLeaves(certificate.data.fields.length);
    }

    // create leaves from clear data
    for (let i = 0; i < certFields.length; i++) {
        if (clearData.has(certFields[i])) {
            clearIndexes.push(i);
            clearLeaves.push(makeLeaf(certFields[i], clearData.get(certFields[i]), certificate.data.salt));
        }
    }

    // add autogenerated leaves if necessary to make tree symmetrical
    for (let i = certFields.length; i < certFields.length + autoLeavesNumber; i++) {
        clearIndexes.push(i);
        clearLeaves.push(clearLeaves[certFields.length - 1]);
    }

    let cryptoCallArgs = new MerkleTreeVerifyArgs();
    cryptoCallArgs.depth = depth;
    cryptoCallArgs.root = arrayBufferToHexString(certificate.data.root);
    cryptoCallArgs.indices = clearIndexes;

    for (let i = 0; i < clearLeaves.length; i++) {
        cryptoCallArgs.leaves.push(arrayBufferToHexString(clearLeaves[i]));
    }
    for (let i = 0; i < args.multiproof.length; i++) {
        cryptoCallArgs.proofs.push(arrayBufferToHexString(args.multiproof[i]));
    }

    let callReturn =  HostFunctions.call(
        cryptoAccountId,
        'merkle_tree_verify',
        MsgPack.serialize(cryptoCallArgs)
    );
    let returnData: u8[] = [];
    if (callReturn.success) {
        returnData = [0xc0];
    } else {
        returnData = Utils.stringtoU8Array('invalid data');
    }

    return MsgPack.appOutputEncode(callReturn.success, returnData);
}

function fieldsCertified(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [];
    if (argsU8.length > 0) {
        let args = decodeFieldsCertifiedArgs(argsU8);
        args.fields = args.fields.sort();
        let initialCertsKeysList = getCertsList(args.target, [args.certifier]);
        let validCertsKeysList: string[] = [];
        if (args.key.length > 0 && args.key != '*') {
            let fullValidCertKey = `${args.target}:${certificatesSectionKey}:${args.certifier}:${args.key}`;
            let validCertIdx = initialCertsKeysList.indexOf(fullValidCertKey);
            if (validCertIdx >= 0) {
                validCertsKeysList.push(initialCertsKeysList[validCertIdx]);
            }
        } else {
            validCertsKeysList = initialCertsKeysList;
        }
        let validCertsList: Certificate[] = [];
        for (let i = 0; i < validCertsKeysList.length; i++) {
            validCertsList.push(
                decodeCertificate(
                    Utils.u8ArrayToArrayBuffer(
                        HostFunctions.loadData(
                            validCertsKeysList[i]
                        )
                    )
                )
            );
        };

        let foundArray: bool[] = new Array<bool>(args.fields.length);
        foundArray.fill(false);

        for (let certIdx = 0; certIdx < validCertsList.length; certIdx++) {
            for (let argsFieldIdx = 0; argsFieldIdx < args.fields.length; argsFieldIdx++) {
                if (!foundArray[argsFieldIdx] && (validCertsList[certIdx].data.fields.indexOf(args.fields[argsFieldIdx]) >= 0)) {
                    foundArray[argsFieldIdx] = true;
                }
            }
        }
        success = true;
        resultBytes = [0xc0];
        for (let i = 0; i < foundArray.length; i++) {
            if (!foundArray[i]) {
                success = false;
            }
        }
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}
// CERTIFICATE MANAGEMENT - END

// DELEGATION MANAGEMENT - BEGIN

const delegationsSectionKey = 'delegations';

function getDelegList(account: string, delegators: string[] = []): string[] {
    const result: string[] = [];
    const prefix: string = `${account}:${delegationsSectionKey}`;
    const keys = HostFunctions.getKeys(`${prefix}:*`);
    for (let keyIdx = 0; keyIdx < keys.length; keyIdx++) {
        if (delegators.length > 0) {
            for (let certifierIdx = 0; certifierIdx < delegators.length; certifierIdx++) {
                if (keys[keyIdx].substring(prefix.length + 1).startsWith(delegators[certifierIdx])) {
                    result.push(keys[keyIdx]);
                    break;
                }
            }
        } else {
            result.push(keys[keyIdx]);
        }
    };
    return result;
}

function saveDelegationBytes(delegate: string, delegator: string, target: string, delegation: u8[]): void {
    let delegPrefix = `${delegate}:${delegationsSectionKey}`;
    let fullDelegkey = `${delegPrefix}:${delegator}:${target}`;
    HostFunctions.storeData(fullDelegkey, delegation);
}

function setDelegation(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [0xc0];
    if (argsU8.length > 0) {
        let args = MsgPack.deserialize<SetDelegationArgs>(argsU8);
        let delegation = decodeDelegation(args.delegation);
        if (delegation.data.target == '*' || delegation.data.target.length <= 0) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Invalid delegation target.'));
        }
        let dataToVerify = delegDataEncodeForVerify(delegation.data);
        let valid = HostFunctions.verify(delegation.data.delegator, dataToVerify, Utils.arrayBufferToU8Array(delegation.signature));
        if (!valid) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Invalid delegation signature.'));
        }
        if (ctx.network != delegation.data.network) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Wrong network.'));
        }
        let delegate = delegation.data.delegate;
        let delegator = rawPubKeyToAccountId(delegation.data.delegator.value);;
        if (ctx.origin != delegate && ctx.origin != delegator) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Method available only for delegate or delegator.'));
        }
        saveDelegationBytes(delegate, delegator, delegation.data.target, Utils.arrayBufferToU8Array(args.delegation));
        success = true;
        resultBytes = [0xc0];
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}

function removeDelegation(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [];
    if (argsU8.length > 0) {
        let args = MsgPack.deserialize<RemoveDelegArgs>(argsU8);
        if (
            (
                ctx.origin == args.delegate
                || ctx.origin == args.delegator
            )
            && args.targets.length > 0
        ) {
            let delegList = getDelegList(args.delegate, [args.delegator]);
            let deleteAllDelegs = args.targets.indexOf('*') >= 0;
            let delegsToDelete: string[] = [];
            if (deleteAllDelegs) {
                delegsToDelete = delegList;
            } else {
                let prefix = `${args.delegate}:${delegationsSectionKey}:${args.delegator}:`;
                for (let delegIdx = 0; delegIdx < delegList.length; delegIdx++) {
                    if (args.targets.indexOf(delegList[delegIdx].substring(prefix.length)) >= 0) {
                        delegsToDelete.push(delegList[delegIdx]);
                    };
                };
            };
            for (let delegIdx = 0; delegIdx < delegsToDelete.length; delegIdx++) {
                HostFunctions.removeData(delegsToDelete[delegIdx]);
            };
            success = true;
            resultBytes = [0xc0];
        } else {
            resultBytes = Utils.stringtoU8Array('Not allowed');
        }
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}

function checkCapAgainstList(method: string, caps: Map<string, bool>): bool {
    let result = false;
    if (caps.has('*') && caps.get('*') == true) {
        result = true;
    }
    if (caps.has(method)) {
        result = caps.get(method) ? true: false;
    }
    return result;
}

function verifyCapability(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {

    const falseResultValue: u8[] = [0xc2];
    const trueResultValue: u8[] = [0xc3];
    let args = MsgPack.deserialize<VerifyCapabilityArgs>(argsU8);

    // trying to load the correct delegation
    let delegation = new Delegation();
    let delegBytes = HostFunctions.loadData(`${args.delegate}:${delegationsSectionKey}:${args.delegator}:${args.target}`);
    if (delegBytes.length > 0) {
        delegation = decodeDelegation(Utils.u8ArrayToArrayBuffer(delegBytes));
    }
    // if the returned certificate is empty (no signature and other data),
    // then no certificate with the given key was found
    if (delegation.signature.byteLength == 0) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.noDeleg.msg));
    }

    let result = checkCapAgainstList(args.method, delegation.data.capabilities);

    return MsgPack.appOutputEncode(result, [0xc0]);
}
// DELEGATION MANAGEMENT - END

function importData(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    if (ctx.caller != getOwner()) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array("Not authorized."));
    }
    const args = MsgPack.deserialize<ImportArgs>(argsU8);
    if (args.key == '' || args.key == ownerAccountKey) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array("Wrong key."));
    }
    HostFunctions.storeData(args.key, Utils.arrayBufferToU8Array(args.data));
    return MsgPack.appOutputEncode(true, [0xc0]);
}
