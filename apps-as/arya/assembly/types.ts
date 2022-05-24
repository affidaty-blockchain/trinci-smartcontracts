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

import { Types } from '../node_modules/@affidaty/trinci-sdk-as'
// INTERNAL
// nested in Certificate class
export class CertData {
    target: string = '';
    fields: string[] = [];
    salt: ArrayBuffer = new ArrayBuffer(0);
    root: ArrayBuffer = new ArrayBuffer(0);
    certifier: Types.PublicKey = new Types.PublicKey();
}

// main certficate structure
export class Certificate {
    data: CertData = new CertData();
    signature: ArrayBuffer = new ArrayBuffer(0);
    multiProof: ArrayBuffer[] = [];
}

export class DelegData {
    delegate: string = '';
    delegator: Types.PublicKey = new Types.PublicKey();
    network: string = '';
    target: string = '';
    expiration: u64 = 0;
    capabilities: Map<string, bool> = new Map<string, bool>();
}

export class Delegation {
    data: DelegData = new DelegData();
    signature: ArrayBuffer = new ArrayBuffer(0);
}

// args structure for "remove_profile_data" method
@msgpackable
export class RemoveProfileDataArgs {
    keys: string[] = [];
}

// args structure for "set_certificate" method
@msgpackable
export class SetCertArgs {
    key: string = '';
    certificate: ArrayBuffer = new ArrayBuffer(0);
}

// args structure for "remove_certificate" method
@msgpackable
export class RemoveCertArgs {
    target: string = '';
    certifier: string = '';
    keys: string[] = [];
}

export class VerifyDataArgs {
    target: string = '';
    certifier: string = '';
    key: string = '';
    data: Map<string, string> = new Map<string, string>();
    multiproof: ArrayBuffer[] = [];
    certificate: ArrayBuffer = new ArrayBuffer(0);
}

@msgpackable
export class MerkleTreeVerifyArgs {
    root: string = '';
    indices: u32[] = [];
    leaves: string[] = [];
    depth: u32 = 0;
    proofs: string[] = [];
}

@msgpackable
export class SetDelegationArgs {
    delegation: ArrayBuffer = new ArrayBuffer(0);
}

// args structure for "remove_certificate" method
@msgpackable
export class RemoveDelegArgs {
    delegate: string = '';
    delegator: string = '';
    targets: string[] = [];
}

@msgpackable
export class VerifyCapabilityArgs {
    delegate: string = '';
    delegator: string = '';
    target: string = '';
    method: string = '';
}

export class RetCode {
    num: u8;
    msg: string;
    constructor(num: u8, msg: string) {
        this.num = num;
        this.msg = msg;
    }
}

@msgpackable
export class FieldsCertifiedArgs {
    target: string = '';
    certifier: string = '';
    key: string = '';
    fields: string[] = [];
}

@msgpackable
export class ImportArgs {
    key: string = '';
    data: ArrayBuffer = new ArrayBuffer(0);
}
