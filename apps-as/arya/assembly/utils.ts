import { encode as b58encode } from 'as-base58';
import { Sha256 } from '../node_modules/as-hmac-sha2/assembly';
import { HostFunctions, Utils } from '../node_modules/@affidaty/trinci-sdk-as';
import {
    profileDataDecode,
    profileDataEncode,
    certsListDecode,
    certsListEncode,
    decodeCertificate,
    decodeDelegation,
} from './msgpack';
import { Certificate, Delegation } from './types';

export function arrayBufferToHexString(ab: ArrayBuffer): string {
    let result: string = '';
    let dataView = new DataView(ab);
    for (let i = 0; i < dataView.byteLength; i++) {
        let byteStr = dataView.getUint8(i).toString(16);
        for (let i = 0; i < 2 - byteStr.length; i++) {
            byteStr = '0' + byteStr;
        }
        result += byteStr;
    }
    return result;
}

export function loadProfileData(account: string): Map<string, string> {
    let result = new Map<string, string>();
    let profileBytes = HostFunctions.loadData(`${account}:profile_data`);
    if (profileBytes.length > 0) {
        result = profileDataDecode(profileBytes);
    }
    return result;
}

export function saveProfileData(account: string, profile: Map<string, string>): void {
    let profileBytes = profileDataEncode(profile);
    HostFunctions.storeData(`${account}:profile_data`, profileBytes);
}

export function rawPubKeyToAccountId(rawPubKey: ArrayBuffer): string {
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

export function loadCertsList(account: string): string[] {
    let result: string[] = [];
    let certsListBytes = HostFunctions.loadData(`${account}:certificates:list`);
    if (certsListBytes.length > 0) {
        result = certsListDecode(certsListBytes);
    }
    return result;
}

export function saveCertsList(account: string, certsList: string[]): void {
    let certsListBytes = certsListEncode(certsList);
    HostFunctions.storeData(`${account}:certificates:list`, certsListBytes);
}

export function saveCertificateBytes(account: string, certBytes: ArrayBuffer, fullKey: string): void {
    HostFunctions.storeData(`${account}:certificates:${fullKey}`, Utils.arrayBufferToU8Array(certBytes));
}

export function loadCertificate(account: string, fullKey: string): Certificate {
    let result = new Certificate();
    let certBytes = HostFunctions.loadData(`${account}:certificates:${fullKey}`);
    if (certBytes.length > 0) {
        result = decodeCertificate(Utils.u8ArrayToArrayBuffer(certBytes));
    }
    return result;
}

export function removeCertificate(account: string, fullKey: string): void {
    HostFunctions.removeData(`${account}:certificates:${fullKey}`);
}

export function loadDelegList(account: string): string[] {
    let result: string[] = [];
    let certsListBytes = HostFunctions.loadData(`${account}:delegations:list`);
    if (certsListBytes.length > 0) {
        result = certsListDecode(certsListBytes);
    }
    return result;
}

export function saveDelegList(account: string, delegList: string[]): void {
    let certsListBytes = certsListEncode(delegList);
    HostFunctions.storeData(`${account}:delegations:list`, certsListBytes);
}

export function saveDelegationBytes(account: string, delegBytes: ArrayBuffer, fullKey: string): void {
    HostFunctions.storeData(`${account}:delegations:${fullKey}`, Utils.arrayBufferToU8Array(delegBytes));
}

export function loadDelegation(account: string, fullKey: string): Delegation {
    let result = new Delegation();
    let delegBytes = HostFunctions.loadData(`${account}:delegations:${fullKey}`);
    if (delegBytes.length > 0) {
        result = decodeDelegation(Utils.u8ArrayToArrayBuffer(delegBytes));
    }
    return result;
}

export function removeDelegation(account: string, fullKey: string): void {
    HostFunctions.removeData(`${account}:delegations:${fullKey}`);
}
