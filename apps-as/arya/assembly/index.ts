import { encode as b58encode } from 'as-base58';
import { Sha256 } from '../node_modules/as-hmac-sha2/assembly';
import { Types, Utils, MemUtils, HostFunctions, MsgPack } from '../node_modules/@affidaty/trinci-sdk-as';
import {
    arrayBufferToHexString,
    loadProfileData,
    saveProfileData,
    rawPubKeyToAccountId,
    loadCertsList,
    saveCertsList,
    saveCertificateBytes,
    loadCertificate,
    removeCertificate as removeCertificateFromAccount,

    loadDelegList,
    saveDelegList,
    saveDelegationBytes,
    loadDelegation,
    removeDelegation as removeDelegationFromAccound,
} from './utils';
import {
    InitArgs,
    RemoveProfileDataArgs,
    SetCertArgs,
    RemoveCertArgs,
    MerkleTreeVerifyArgs,
    SetDelegationArgs,
    RemoveDelegArgs,
    VerifyCapabilityArgs,
} from './types';
import {
    profileDataDecode,
    RemoveProfileDataArgsDecode,
    decodeCertificate,
    certDataEncodeForVerify,
    decodeVerifyDataArgs,
    decodeDelegation,
    delegDataEncodeForVerify
    // verifyResultEncode,
} from './msgpack';
import { retCodes } from './retcodes';

export function my_alloc(size: i32): i32 {
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
    methodsMap.set('set_delegation', setDelegation);
    methodsMap.set('remove_delegation', removeDelegation);
    methodsMap.set('verify_capability', verifyCapability);

    if (!methodsMap.has(ctx.method)) {
        let success = false;
        let resultBytes = Utils.stringtoU8Array('Method not found.');
        return MsgPack.appOutputEncode(success, resultBytes);
    }

    return methodsMap.get(ctx.method)(ctx, argsU8);
}

// INITIALIZATION

function init(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let args = MsgPack.deserialize<InitArgs>(argsU8);
    HostFunctions.storeData('cryptoAccountId', Utils.stringtoU8Array(args.crypto));
    return MsgPack.appOutputEncode(true, [0xc0]);
}

// BEGIN - PROFILE DATA MANAGEMENT
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
        resultBytes = [0xC0];
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}
// END - PROFILE DATA MANAGEMENT

// BEGIN - CERTIFICATES MANAGEMENT
function setCertificate(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [0xc0];
    if (argsU8.length > 0) {
        let args = MsgPack.deserialize<SetCertArgs>(argsU8);
        if (args.key == '*') {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('This key cannot be used.'));
        }
        let certificate = decodeCertificate(args.certificate);
        let dataToVerify = certDataEncodeForVerify(certificate.data);
        let valid = HostFunctions.verify(certificate.data.certifier, dataToVerify, Utils.arrayBufferToU8Array(certificate.signature));
        if (!valid) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Invalid certificate signature.'));
        }
        let target = certificate.data.target;
        let certifier = rawPubKeyToAccountId(certificate.data.certifier.value);;
        if (ctx.origin != target && ctx.origin != certifier) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Method available only for certificate target or certifier.'));
        }
        let certsList = loadCertsList(target);
        let fullCertkey = `${certifier}:${args.key}`;
        if (certsList.indexOf(fullCertkey) == -1 ) {
            certsList.push(fullCertkey)
        }
        saveCertificateBytes(target, args.certificate, fullCertkey);
        saveCertsList(target, certsList);
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
            let certsList = loadCertsList(args.target);
            let newCertsList: string[] = [];
            let keysToDelete: string[] = [];
            let deleteAllKeys = args.keys.indexOf('*') >= 0;
            for (let certIdx = 0; certIdx < certsList.length; certIdx++) {
                if (certsList[certIdx].includes(`${args.certifier}:`)) {
                    if (deleteAllKeys) {
                        keysToDelete.push(certsList[certIdx])
                    } else {
                        let toDelete = false;
                        for (let keyIdx = 0; keyIdx < args.keys.length; keyIdx++) {
                            if (certsList[certIdx] == `${args.certifier}:${args.keys[keyIdx]}`) {
                                toDelete = true;
                                break;
                            }
                        }
                        if (toDelete) {
                            keysToDelete.push(certsList[certIdx]);
                        } else {
                            newCertsList.push(certsList[certIdx]);
                        }
                    }
                } else {
                    newCertsList.push(certsList[certIdx]);
                }
            }
            saveCertsList(args.target, newCertsList);
            for (let i = 0; i < keysToDelete.length; i++) {
                removeCertificateFromAccount(args.target, keysToDelete[i]);
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

// START - CERTIFICATE DATA VERIFICATION

// create a leaf from data
function makeLeaf(key: string, value: string, salt: ArrayBuffer): ArrayBuffer {
    let strToHash = `${value}${key}${arrayBufferToHexString(salt)}`;
    let strBin: Uint8Array = Utils.u8ArrayToUint8Array(Utils.stringtoU8Array(strToHash));
    const hash: Uint8Array = Sha256.hash(strBin);
    return hash.buffer;
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
    let cryptoAccountId = Utils.u8ArrayToString(HostFunctions.loadData('cryptoAccountId'));
    if (cryptoAccountId.length < 1) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.noInit.msg));
    }

    const falseResultValue: u8[] = [0xc2];
    const trueResultValue: u8[] = [0xc3];
    let args = decodeVerifyDataArgs(argsU8);

    // load the correct certificate
    let certificate = loadCertificate(args.target, args.certificate);
    // if the returned certificate is empty (no signature and other data),
    // then no certificate with the given key was found
    if (certificate.signature.byteLength == 0) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.excessFields.msg));
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

    // add autogenerated leaves if necessary to make tree symmetrical if necessary
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
        Utils.u8ArrayToString(
            HostFunctions.loadData('cryptoAccountId')
        ),
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
// END - CERTIFICATE DATA VERIFICATION
// END - CERTIFICATE MANAGEMENT

// BEGIN - DELEGATION MANAGEMENT
function setDelegation(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    HostFunctions.log('===============================');
    let success = false;
    let resultBytes: u8[] = [0xc0];
    if (argsU8.length > 0) {
        let args = MsgPack.deserialize<SetDelegationArgs>(argsU8);
        if (args.key == '*') {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('This key cannot be used.'));
        }
        let delegation = decodeDelegation(args.delegation);
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
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Method available only for degate or delegator.'));
        }
        let delegList = loadDelegList(delegate);
        let fullDelegKey = `${delegator}:${delegation.data.target}`;
        if (delegList.indexOf(fullDelegKey) == -1 ) {
            delegList.push(fullDelegKey)
        }
        saveDelegationBytes(delegate, args.delegation, fullDelegKey);
        saveDelegList(delegate, delegList);
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
            let delegList = loadDelegList(args.delegate);
            let newDelegList: string[] = [];
            let targetsToDelete: string[] = [];
            let deleteAllKeys = args.targets.indexOf('*') >= 0;
            for (let delegIdx = 0; delegIdx < delegList.length; delegIdx++) {
                if (delegList[delegIdx].includes(`${args.delegator}:`)) {
                    if (deleteAllKeys) {
                        targetsToDelete.push(delegList[delegIdx]);
                    } else {
                        let toDelete = false;
                        for (let targetIdx = 0; targetIdx < args.targets.length; targetIdx++) {
                            if (delegList[delegIdx] == `${args.delegator}:${args.targets[targetIdx]}`) {
                                toDelete = true;
                                break;
                            }
                        }
                        if (toDelete) {
                            targetsToDelete.push(delegList[delegIdx]);
                        } else {
                            newDelegList.push(delegList[delegIdx]);
                        }
                    }
                } else {
                    newDelegList.push(delegList[delegIdx]);
                }
            }
            saveDelegList(args.delegate, newDelegList);
            for (let i = 0; i < targetsToDelete.length; i++) {
                removeDelegationFromAccound(args.delegate, targetsToDelete[i]);
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

// START - DELEGATION VERIFICATION

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
    let cryptoAccountId = Utils.u8ArrayToString(HostFunctions.loadData('cryptoAccountId'));
    if (cryptoAccountId.length < 1) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.noInit.msg));
    }

    const falseResultValue: u8[] = [0xc2];
    const trueResultValue: u8[] = [0xc3];
    let args = MsgPack.deserialize<VerifyCapabilityArgs>(argsU8);

    // load the correct certificate
    let delegation = loadDelegation(args.delegate, `${args.delegator}:${args.target}`);
    // if the returned certificate is empty (no signature and other data),
    // then no certificate with the given key was found
    if (delegation.signature.byteLength == 0) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.excessFields.msg));
    }

    let result = checkCapAgainstList(args.method, delegation.data.capabilities);

    return MsgPack.appOutputEncode(result, [0xc0]);
}
// END - DELEGATION VERIFICATION
// END - DELEGATION MANAGEMENT