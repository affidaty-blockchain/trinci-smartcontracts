import { encode as b58encode } from 'as-base58';
import { Sha256 } from '../node_modules/as-hmac-sha2/assembly';
import { Types, Utils, MemUtils, HostFunctions, MsgPack } from '../node_modules/@affidaty/trinci-sdk-as';
import {
    RemoveProfileDataArgs,
    SetCertArgs,
    RemoveCertArgs,
    Identity,
} from './types';
import {
    profileDataDecode,
    profileDataEncode,
    decodeCertificate,
    certsListDecode,
    certsListEncode,
    certDataEncodeForVerify,
} from './msgpack';

export function my_alloc(size: i32): i32 {
    return heap.alloc(size) as i32;
}

export function run(ctxAddress: i32, ctxSize: i32, argsAddress: i32, argsSize: i32): Types.TCombinedPtr {
    let ctxU8Arr: u8[] = MemUtils.u8ArrayFromMem(ctxAddress, ctxSize);
    let ctx = MsgPack.ctxDecode(ctxU8Arr);
    let argsU8: u8[] = MemUtils.u8ArrayFromMem(argsAddress, argsSize);
    let methodsMap = new Map<string, (ctx: Types.AppContext, args: u8[])=>Types.TCombinedPtr>();

    methodsMap.set('set_profile_data', setProfileData);
    methodsMap.set('remove_profile_data', removeProfileData);
    methodsMap.set('set_certificate', setCertificate);
    methodsMap.set('remove_certificate', removeCertificate);

    if (!methodsMap.has(ctx.method)) {
        let success = false;
        let resultBytes = Utils.stringtoU8Array('Method not found.');
        return MsgPack.appOutputEncode(success, resultBytes);
    }

    return methodsMap.get(ctx.method)(ctx, argsU8);
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