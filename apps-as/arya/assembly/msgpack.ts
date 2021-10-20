import { Writer, Encoder, Decoder, Sizer } from '@wapc/as-msgpack';
import { Utils } from '../node_modules/@affidaty/trinci-sdk-as';
import { Certificate, CertData } from './types';

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

export function decodeCertificate(certBytes: ArrayBuffer): Certificate {
    let decoder = new Decoder(certBytes);

    let resultCert = new Certificate();

    let hasMultiProof = false
    if(decoder.readArraySize() == 3) {
        hasMultiProof = true;
    }

    decoder.readArraySize();
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
    writer.writeArraySize(4);
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

export function certsListDecode(certsList: ArrayBuffer): Map<string, ArrayBuffer> {
    let decoder = new Decoder(certsList);
    let result = new Map<string, ArrayBuffer>();
    let mapSize = decoder.readMapSize();
    for (let i: u32 = 0; i < mapSize; i++) {
        let key = decoder.readString();
        let value = decoder.readByteArray();
        result.set(key, value);
    }
    return result;
}

function writeCertsList(writer: Writer, certsMap: Map<string, ArrayBuffer>): void {
    writer.writeMapSize(certsMap.size);
    const keys = certsMap.keys();
    for (let i: i32 = 0; i < keys.length; i++) {
        writer.writeString(keys[i]);
        writer.writeByteArray(certsMap.get(keys[i]));
    }
}

export function certsListEncode(certsMap: Map<string, ArrayBuffer>): ArrayBuffer {
    let sizer = new Sizer();
    writeCertsList(sizer, certsMap);
    let arrayBuffer = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(arrayBuffer);
    writeCertsList(encoder, certsMap);
    return arrayBuffer;
}

class VerifyDataArgs {
    target: string = '';
    certificate: string = '';
    data: Map<string, string> = new Map<string, string>();
    multiproof: ArrayBuffer[] = [];
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

export class CallReturn {
    success: bool = false;
    result: ArrayBuffer = new ArrayBuffer(0);
}

export function callReturnDecode(u8Arr: u8[]): CallReturn {
    let ab = Utils.u8ArrayToArrayBuffer(u8Arr);
    let decoder = new Decoder(ab);
    let ret = new CallReturn();
    let arrSize = decoder.readArraySize();
    ret.success = decoder.readBool();
    ret.result = decoder.readByteArray();
    return ret;
}