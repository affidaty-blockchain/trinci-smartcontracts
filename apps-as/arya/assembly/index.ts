import { encode as b58encode } from 'as-base58';
import { Sha256 } from '../node_modules/as-hmac-sha2/assembly';
import { Types, Utils, MemUtils, HostFunctions, MsgPack } from '../node_modules/@affidaty/trinci-sdk-as';
import { arrayBufferToHexString } from './utils';
import {
    RemoveProfileDataArgs,
    SetCertArgs,
    RemoveCertArgs,
    Identity,
    RetCode,
} from './types';
import {
    profileDataDecode,
    profileDataEncode,
    decodeCertificate,
    certsListDecode,
    certsListEncode,
    certDataEncodeForVerify,
    decodeVerifyDataArgs,
    verifyResultEncode,
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

    if (!methodsMap.has(ctx.method)) {
        let success = false;
        let resultBytes = Utils.stringtoU8Array('Method not found.');
        return MsgPack.appOutputEncode(success, resultBytes);
    }

    return methodsMap.get(ctx.method)(ctx, argsU8);
}

// INITIALIZATION
@msgpackable
class InitArgs {
    // id of the account with crypto smart contact (for merkle tree verification)
    crypto: string = '';
}

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
        let identityBytes = HostFunctions.loadAsset(ctx.origin);
        let identity = new Identity();
        if (identityBytes.length > 0) {
            identity = MsgPack.deserialize<Identity>(identityBytes);
        }
        let profileBytes = Utils.arrayBufferToU8Array(identity.profile);
        let profile = new Map<string, string>();
        if (profileBytes.length > 0) {
            profile = profileDataDecode(profileBytes);
        }
        let newProfileDataMap = profileDataDecode(argsU8);
        let newProfileDataKeys = newProfileDataMap.keys();
        for (let i = 0; i < newProfileDataKeys.length; i++) {
            profile.set(newProfileDataKeys[i], newProfileDataMap.get(newProfileDataKeys[i]));
        }
        identity.profile = Utils.u8ArrayToArrayBuffer(profileDataEncode(profile));
        HostFunctions.storeAsset(ctx.origin, MsgPack.serialize<Identity>(identity));
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
        let identityBytes = HostFunctions.loadAsset(ctx.caller);
        let identity = new Identity();
        if (identityBytes.length > 0) {
            identity = MsgPack.deserialize<Identity>(identityBytes);
        }
        let profileBytes = Utils.arrayBufferToU8Array(identity.profile);
        let profile = new Map<string, string>();
        if (profileBytes.length > 0) {
            profile = profileDataDecode(profileBytes);
        }
        let profileKeysToDelete = MsgPack.deserialize<RemoveProfileDataArgs>(argsU8).keys;
        if (profileKeysToDelete.indexOf('*') == -1) {
            for (let i = 0; i < profileKeysToDelete.length; i++) {
                if (profile.has(profileKeysToDelete[i])) {
                    profile.delete(profileKeysToDelete[i]);
                }
            }
        } else {
            profile.clear();
        }
        profileBytes = profileDataEncode(profile);
        identity.profile = Utils.u8ArrayToArrayBuffer(profileBytes);
        HostFunctions.storeAsset(ctx.origin, MsgPack.serialize<Identity>(identity));
        success = true;
        resultBytes = [0xC0];
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}

// END - PROFILE DATA MANAGEMENT
// BEGIN - CERTIFICATES MANAGEMENT

function rawPubKeyToAccountId(rawPubKey: ArrayBuffer): string {
    const protobufHeader: u8[] = [
        0x08, 0x03, // Algorythm type identifier (ECDSA)
        0x12, 0x78, // Content length
    ];
    const asn1Header: u8[]
     = [
        0x30, 0x76, // byte count
        0x30, 0x10, // byte len
        0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, // EC Public key OID
        0x06, 0x05, 0x2b, 0x81, 0x04, 0x00, 0x22, // secp384r1 curve OID
        0x03, 0x62, 0x00, // bitstring (bytes count)
    ];
    let data: u8[] = [];
    data = data.concat(protobufHeader);
    data = data.concat(asn1Header);
    data = data.concat(Utils.arrayBufferToU8Array(rawPubKey));
    const hash: Uint8Array = Sha256.hash(Utils.u8ArrayToUint8Array(data));
    const multihashHeader: u8[] = [
        0x12, // hash algorithm identifier (SHA256)
        0x20, // hash length  (32)
    ];
    let accountIdBytes: u8[] = [];
    accountIdBytes = accountIdBytes.concat(multihashHeader);
    accountIdBytes = accountIdBytes.concat(Utils.uint8ArrayToU8Array(hash));
    let accountId: string = b58encode(Utils.u8ArrayToUint8Array(accountIdBytes));
    return accountId;
}

function setCertificate(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [0xc0];
    if (argsU8.length > 0) {
        let setCertArgs = MsgPack.deserialize<SetCertArgs>(argsU8);
        let certificate = decodeCertificate(setCertArgs.certificate);
        let dataToVerify = certDataEncodeForVerify(certificate.data);
        let valid = HostFunctions.verify(certificate.data.certifier, dataToVerify, Utils.arrayBufferToU8Array(certificate.signature));
        if (!valid) {
            return MsgPack.appOutputEncode(false, Utils.stringtoU8Array('Invalid certificate signature.'));
        }
        let identity = new Identity();
        let identityBytes = HostFunctions.loadAsset(setCertArgs.target);
        if (identityBytes.length > 0) {
            identity = MsgPack.deserialize<Identity>(identityBytes);
        }
        let certsList = new Map<string, ArrayBuffer>();
        let certsListBytes = identity.certificates;
        if (certsListBytes.byteLength > 0) {
            certsList = certsListDecode(certsListBytes);
        }
        let issuer = rawPubKeyToAccountId(certificate.data.certifier.value);
        let certKey = `${issuer}:${setCertArgs.key}`;
        certsList.set(certKey, setCertArgs.certificate);
        identity.certificates = certsListEncode(certsList);
        HostFunctions.storeAsset(setCertArgs.target, MsgPack.serialize<Identity>(identity));
        success = true;
        resultBytes = [0xC0];
    } else {
        resultBytes = Utils.stringtoU8Array('Arguments error.');
    }
    return MsgPack.appOutputEncode(success, resultBytes);
}

function removeCertificate(ctx: Types.AppContext, argsU8: u8[]): Types.TCombinedPtr {
    let success = false;
    let resultBytes: u8[] = [];
    if (argsU8.length > 0) {
        let removeCertArgs = MsgPack.deserialize<RemoveCertArgs>(argsU8);
        if (
            (
                ctx.origin == removeCertArgs.target
                || ctx.origin == removeCertArgs.issuer
            )
            && removeCertArgs.keys.length > 0
        ) {
            let identity = new Identity();
            let identityBytes = HostFunctions.loadAsset(removeCertArgs.target);
            if (identityBytes.length > 0) {
                identity = MsgPack.deserialize<Identity>(identityBytes);
            }

            let certsList = new Map<string, ArrayBuffer>();
            let certsListBytes = identity.certificates;
            if (certsListBytes.byteLength > 0) {
                certsList = certsListDecode(certsListBytes);
            }
            for (let i = 0; i < removeCertArgs.keys.length; i++) {
                let certFullKey: string = `${removeCertArgs.issuer}:${removeCertArgs.keys[i]}`;
                if (certsList.has(certFullKey)) {
                    certsList.delete(certFullKey);
                }
            }
            identity.certificates = certsListEncode(certsList);
            HostFunctions.storeAsset(removeCertArgs.target, MsgPack.serialize<Identity>(identity));
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
// END - CERTIFICATE MANAGEMENT

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

@msgpackable
class MerkleTreeVerifyArgs {
    root: string = '';
    indices: u32[] = [];
    leaves: string[] = [];
    depth: u32 = 0;
    proofs: string[] = [];
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
    let identity = new Identity();
    let identityBytes = HostFunctions.loadAsset(args.target);
    if (identityBytes.length > 0) {
        identity = MsgPack.deserialize<Identity>(identityBytes);
    }

    let certsList = new Map<string, ArrayBuffer>();
    let certsListBytes = identity.certificates;
    if (certsListBytes.byteLength > 0) {
        certsList = certsListDecode(certsListBytes);
    }
    if (!certsList.has(args.certificate)) {
        return MsgPack.appOutputEncode(false, Utils.stringtoU8Array(retCodes.noCert.msg));
    }

    // get target certificate
    let certificate = decodeCertificate(certsList.get(args.certificate));
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
        // deserialize profile data
        let profileBytes = Utils.arrayBufferToU8Array(identity.profile);
        let profile = new Map<string, string>();
        if (profileBytes.length > 0) {
            profile = profileDataDecode(profileBytes);
        };

        for (let i = 0; i < missingFields.length; i++) {
            if (profile.has(missingFields[i])) {
                clearData.set(missingFields[i], profile.get(missingFields[i]));
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