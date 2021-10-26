import { Types } from '../node_modules/@affidaty/trinci-sdk-as'

// ARGS
// args structure for "init" method
@msgpackable
export class InitArgs {
    // id of the account with crypto smart contact (for merkle tree verification)
    crypto: string = '';
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
    certificate: string = '';
    data: Map<string, string> = new Map<string, string>();
    multiproof: ArrayBuffer[] = [];
}

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

export class RetCode {
    num: u8;
    msg: string;
    constructor(num: u8, msg: string) {
        this.num = num;
        this.msg = msg;
    }
}

 
