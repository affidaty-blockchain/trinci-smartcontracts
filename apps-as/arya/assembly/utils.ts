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

import { encode as b58encode } from 'as-base58';
import { HostFunctions, Utils } from '../node_modules/@affidaty/trinci-sdk-as';

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
    const hash: u8[] = HostFunctions.sha256(data);
    const multihashHeader: u8[] = [
        0x12, // hash algorithm identifier (SHA256)
        0x20, // hash length  (32)
    ];
    let accountIdBytes: u8[] = [];
    accountIdBytes = accountIdBytes.concat(multihashHeader);
    accountIdBytes = accountIdBytes.concat(hash);
    let accountId: string = b58encode(Utils.u8ArrayToUint8Array(accountIdBytes));
    return accountId;
}
