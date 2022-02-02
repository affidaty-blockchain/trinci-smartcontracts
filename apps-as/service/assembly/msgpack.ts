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

import { Decoder, Encoder, Sizer } from '@wapc/as-msgpack';
import { Utils } from '../node_modules/@affidaty/trinci-sdk-as'

export function deserializeStringArray(u8Arr: u8[]): string[] {
    let result: string[] = [];
    let ab = Utils.u8ArrayToArrayBuffer(u8Arr);
    let decoder = new Decoder(ab);
    let arrSize = decoder.readArraySize();
    for (let i: u32 = 0; i < arrSize; i++) {
        result.push(decoder.readString());
    }
    return result;
}

export function serializeStringArray(array: string[]): u8[] {
    let sizer = new Sizer();
    sizer.writeArraySize(array.length);
    for (let i = 0; i < array.length; i++) {
        sizer.writeString(array[i]);
    }
    let ab = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(ab);
    encoder.writeArraySize(array.length);
    for (let i = 0; i < array.length; i++) {
        encoder.writeString(array[i]);
    }
    return Utils.arrayBufferToU8Array(ab);
}

export function deserializeString(u8Arr: u8[]): string {
    let ab = Utils.u8ArrayToArrayBuffer(u8Arr);
    let decoder = new Decoder(ab);
    return decoder.readString();
}

export function serializeString(value: string): u8[] {
    let sizer = new Sizer();
    sizer.writeString(value);
    let ab = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(ab);
    encoder.writeString(value);
    return Utils.arrayBufferToU8Array(ab);
}

export function deserializeU64(u8Arr: u8[]): u64 {
    let ab = Utils.u8ArrayToArrayBuffer(u8Arr);
    let decoder = new Decoder(ab);
    return decoder.readUInt64();
}

export function serializeU64(value: u64): u8[] {
    let sizer = new Sizer();
    sizer.writeUInt64(value);
    let ab = new ArrayBuffer(sizer.length);
    let encoder = new Encoder(ab);
    encoder.writeUInt64(value);
    return Utils.arrayBufferToU8Array(ab);
}

export function deserializeValidatorsMap(u8Arr: u8[]): Map<string, u64> {
    let result = new Map<string, u64>();
    let ab = Utils.u8ArrayToArrayBuffer(u8Arr);
    let decoder = new Decoder(ab);
    let mapSize = decoder.readMapSize();
    for (let i: u32 = 0; i < mapSize; i++) {
        result.set(decoder.readString(), decoder.readUInt64());
    }
    return result;
}
