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

export class RawAppOutput {
    public success: bool;
    public result: u8[];
    constructor(success: bool = false, result: u8[] = []) {
        this.success = success;
        this.result = result;
    }
}

@msgpackable
export class ContractRegistrationArgs {
    public name: string = '';
    public version: string = '';
    public description: string = '';
    public url: string = '';
    public bin: ArrayBuffer = new ArrayBuffer(0);
}

@msgpackable
export class ContractRegistrationData {
    public name: string = '';
    public version: string = '';
    public publisher: string = '';
    public description: string = '';
    public url: string = '';
}

@msgpackable
export class ConsumeFuelArgs {
    from: string = '';
    units: u64 = 0;
}

@msgpackable
export class MintArgs {
    to: string = '';
    units: u64 = 0;
}

@msgpackable
export class BurnArgs {
    from: string = '';
    units: u64 = 0;
}

@msgpackable
export class TransferArgs {
    from: string = '';
    to: string = '';
    units: u64 = 0;
}

@msgpackable
export class IsDelegatedToArgs {
    delegate: string = '';
    action: string = '';
    data: ArrayBuffer = new ArrayBuffer(0);
}

@msgpackable
export class BlockchainSettings {

    /** Controls whether node accepts broadcast (made for '*' network) transactions */
    accept_broadcast: bool = false;

    /** Max number of transactions per block. */
    block_threshold: u64 = 42;

    /** Max time (in seconds) after which a validator node tries to create a block from unconfirmed transactions even if it has not reached block_treshold value. */
    block_timeout: u64 = 5;

    /** Name of the method to call to handle fuel burning. If empty, fuel won't be burned.*/
    burning_fuel_method: string = '';
}

@msgpackable
export class FuelAssetStats {
    circulating_volume: u64 = 0;
}
