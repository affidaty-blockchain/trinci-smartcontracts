import { Writer, Encoder, Decoder, Sizer } from '@wapc/as-msgpack';
import { Utils } from '../node_modules/@affidaty/trinci-sdk-as';
import { Certificate, CertData, Delegation, DelegData, VerifyDataArgs, RetCode } from './types';

// PROFILE DATA
export function profileDataDecode(dataU8Arr: u8[]): Map<string, string> {
    let dataArrayBuffer = Utils.u8ArrayToArrayBuffer(dataU8Arr);
    let result = new Map<string, string>();
    let decoder = new Decoder(dataArrayBuffer);
    let mapSize = decoder.readMapSize();
    for (let i: u32 = 0; i < mapSize; i++) {
        let key = decoder.readString();
        let val = decoder.readString();
        result.set(key, val);
    }
    return result;
}

export function RemoveProfileDataArgsDecode(dataU8Arr: u8[]): string[] {
    let dataArrayBuffer = Utils.u8ArrayToArrayBuffer(dataU8Arr);
    let result: string[] = [];
    let decoder = new Decoder(dataArrayBuffer);
    let arraySize = decoder.readArraySize();
    for (let i: u32 = 0; i < arraySize; i++) {
        result.push(decoder.readString());
    }
    return result;
}

function dataMapEncode(writer: Writer, dataMap: Map<string, string>): void {
    writer.writeMapSize(dataMap.size);
    const keys = dataMap.keys();
    for (let i: i32 = 0; i < keys.length; i++) {
        const key = keys[i];
        const value = dataMap.get(key);
        writer.writeString(key);
        writer.writeString(value);
    }
}

export function profileDataEncode(dataMap: Map<string, string>): u8[] {
    let sizer = new Sizer();
    dataMapEncode(sizer, dataMap);
    let arrayBuffer = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(arrayBuffer);
    dataMapEncode(encoder, dataMap);
    return Utils.arrayBufferToU8Array(arrayBuffer);
}

// CERTIFICATES
export function certsListDecode(certsListBytes: u8[]): string[] {
    let ab = Utils.u8ArrayToArrayBuffer(certsListBytes);
    let decoder = new Decoder(ab);
    let result: string[] = [];
    let arraySize = decoder.readArraySize();
    for (let i: u32 = 0; i < arraySize; i++) {
        result.push(decoder.readString());
    }
    return result;
}

function writeCertsList(writer: Writer, certsList: string[]): void {
    writer.writeArraySize(certsList.length);
    for (let i: i32 = 0; i < certsList.length; i++) {
        writer.writeString(certsList[i]);
    }
}

export function certsListEncode(certsList: string[]): u8[] {
    let sizer = new Sizer();
    writeCertsList(sizer, certsList);
    let arrayBuffer = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(arrayBuffer);
    writeCertsList(encoder, certsList);
    return Utils.arrayBufferToU8Array(arrayBuffer);
}

export function decodeCertificate(certBytes: ArrayBuffer): Certificate {
    let decoder = new Decoder(certBytes);
    let resultCert = new Certificate();
    let hasMultiProof = false;
    let topMostSize = decoder.readArraySize();
    if(topMostSize == 3) {
        hasMultiProof = true;
    }
    decoder.readArraySize();
    resultCert.data.target = decoder.readString();
    let fieldsNum = decoder.readArraySize();
    for (let i: u32 = 0; i < fieldsNum; i++) {
        resultCert.data.fields.push(decoder.readString());
    }
    resultCert.data.salt = decoder.readByteArray();
    resultCert.data.root = decoder.readByteArray();
    decoder.readArraySize();
    resultCert.data.certifier.type = decoder.readString();
    resultCert.data.certifier.curve = decoder.readString();
    resultCert.data.certifier.value = decoder.readByteArray();
    resultCert.signature = decoder.readByteArray();
    if (hasMultiProof) {
        resultCert.multiProof = decoder.readArray<ArrayBuffer>((decoder: Decoder) => {
            return decoder.readByteArray()
        })
    }
    return resultCert;
}

function writeCertData(writer: Writer, certData: CertData): void {
    writer.writeArraySize(5);
    writer.writeString(certData.target);
    writer.writeArraySize(certData.fields.length);
    for (let fieldIdx = 0; fieldIdx < certData.fields.length; fieldIdx++) {
        writer.writeString(certData.fields[fieldIdx]);
    }
    writer.writeByteArray(certData.salt);
    writer.writeByteArray(certData.root);
    writer.writeArraySize(3);
    writer.writeString(certData.certifier.type);
    writer.writeString(certData.certifier.curve);
    writer.writeByteArray(certData.certifier.value);
}

export function certDataEncodeForVerify(certData: CertData): u8[] {
    let sizer = new Sizer();
    writeCertData(sizer, certData);
    let ab = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(ab);
    writeCertData(encoder, certData);
    return Utils.arrayBufferToU8Array(ab);
}

export function decodeVerifyDataArgs(argsU8: u8[]): VerifyDataArgs {
    let result = new VerifyDataArgs();
    let decoder = new Decoder(Utils.u8ArrayToArrayBuffer(argsU8));
    let mapSize = decoder.readMapSize();
    for (let i: u32 = 0; i < mapSize; i++) {
        let fieldName = decoder.readString();
        if (fieldName == 'target') {
            result.target = decoder.readString();
        } else if (fieldName == 'certificate') {
            result.certificate = decoder.readString();
        } else if (fieldName == 'data') {
            let dataMapSize = decoder.readMapSize();
            for (let i: u32 = 0; i < dataMapSize; i++) {
                result.data.set(decoder.readString(), decoder.readString())
            }
        } else if (fieldName == 'multiproof') {
            let multiProofLength = decoder.readArraySize();
            for (let i: u32 = 0; i < multiProofLength; i++) {
                result.multiproof.push(decoder.readByteArray());
            }
        } else {
            throw new Error(`Unknown field: ${fieldName}`);
        }
    }
    return result;
}

function writeVerifyResult(writer: Writer, retCode: RetCode): void {
    writer.writeArraySize(2);
    writer.writeUInt8(retCode.num);
    writer.writeString(retCode.msg);
}

export function verifyResultEncode(retCode: RetCode): u8[] {
    let sizer = new Sizer();
    writeVerifyResult(sizer, retCode);
    let ab = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(ab);
    writeVerifyResult(encoder, retCode);
    return Utils.arrayBufferToU8Array(ab);
}

export function decodeDelegation(certBytes: ArrayBuffer): Delegation {
    let decoder = new Decoder(certBytes);
    let result = new Delegation();
    decoder.readArraySize(); // topmost
    decoder.readArraySize(); // data
    result.data.delegate = decoder.readString();
    decoder.readArraySize(); // delegator
    result.data.delegator.type = decoder.readString();
    result.data.delegator.curve = decoder.readString();
    result.data.delegator.value = decoder.readByteArray();
    result.data.network = decoder.readString();
    result.data.target = decoder.readString();
    result.data.expiration = decoder.readUInt64();
    let capsSize = decoder.readMapSize(); // capabilities size
    for (let i: u32 = 0; i < capsSize; i++) {
        let key = decoder.readString();
        let value = decoder.readBool();
        result.data.capabilities.set(key, value);
    }
    result.signature = decoder.readByteArray();
    return result;
}

function writeDelegData(writer: Writer, delegData: DelegData): void {
    writer.writeArraySize(6);
    writer.writeString(delegData.delegate);
    writer.writeArraySize(3);
    writer.writeString(delegData.delegator.type);
    writer.writeString(delegData.delegator.curve);
    writer.writeByteArray(delegData.delegator.value);
    writer.writeString(delegData.network);
    writer.writeString(delegData.target);
    writer.writeUInt64(delegData.expiration);
    let caps = delegData.capabilities.keys();
    writer.writeMapSize(caps.length);
    for (let i = 0; i < caps.length; i++) {
        writer.writeString(caps[i]);
        writer.writeBool(delegData.capabilities.get(caps[i]))
    }
}

export function delegDataEncodeForVerify(delegData: DelegData): u8[] {
    let sizer = new Sizer();
    writeDelegData(sizer, delegData);
    let ab = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(ab);
    writeDelegData(encoder, delegData);
    return Utils.arrayBufferToU8Array(ab);
}