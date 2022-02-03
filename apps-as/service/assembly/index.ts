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
//
// Scroll down to modify settings.

import sdk from '../node_modules/@affidaty/trinci-sdk-as';
import * as Errors from './errors';
import {
    BlockchainSettings,
    ContractRegistrationArgs,
    ContractRegistrationData,
    RawAppOutput,
    ConsumeFuelArgs,
    MintArgs,
    BurnArgs,
    TransferArgs,
    IsDelegatedToArgs,
    FuelAssetStats,
} from './types';
import {
    deserializeStringArray,
    serializeStringArray,
    deserializeString,
    serializeU64,
    deserializeU64,
    deserializeValidatorsMap,
    serializeString,
} from './msgpack';

const trueBytes: u8[] = [0xc3];
const falseBytes: u8[] = [0xc2];

const bcSectionName = 'blockchain';
const blockchainSettingsSectionKey = `${bcSectionName}:settings`;
const blockchainValidatorsSectionKey = `${bcSectionName}:validators`;
const blockchainAdminsSectionKey = `${bcSectionName}:admins`;

const contractsSectionName = 'contracts';
const contractsMetadataSectionKey = `${contractsSectionName}:metadata`;
const contractsCodeSectionKey = `${contractsSectionName}:code`;
const contractsVersionsSectionKey = `${contractsSectionName}:versions`;
const preapprovedContractsSectionName = `${contractsSectionName}:preapproved`;

const fuelAssetSectionName = 'fuel';
const fuelAssetStatsSectionName = `${fuelAssetSectionName}:stats`;

// ========================= SETTINGS START =========================
// A trinci network is identified by the hash of the service smart contract
// used to initialize nodes. In order to create a separate network with all the
// same properties of another network you need to change this value.
const NONCE = '_';

const THIS_NAME = 'service';
const THIS_VERSION = '1.0.0';
const THIS_DESCRIPTION = 'Service smart contract';
const THIS_URL = 'https://affidaty.io';

// Id of the accont to which eventual reminders of consume_fuel goes. Once
// enough reminders have been accumulated, they can be redistributed manually by
// an admin by calling redistribute_remainders method. Remainder account cannot
// be used in a mint/burn/transfer transaction with fuel asset. Operations with
// other assets are still possible.
const remainderAccount = 'leaks';

// If the parameter below is set to true, everyone can publish their own smart contracts.
// Otherwise you have to be an admin or your smart contract's hash should be preauthorized
// by an admin ("add_preapproved_contract" method).
const everyoneCanPublish = false;

// if this is set to true, service smart contract can be updated by updateServiceSmartContract method
const thisCanBeUpdated = true;

// This is the default validators accounts and their initial stakes, which gets
// added automatically to validators list during init process. If it's stake
// value is lower than 1, it won't be added to list. If no default validators
// are added, no one can create new blocks and blockchain won't run.
const defaultValidatorsAccounts = new Map<string, u64>()
    .set('<Node ID>', 1);


// Here you can set immutable blockchain parameters
function getSettingsObject(): BlockchainSettings {
    let settings = new BlockchainSettings();

    // Manual blockchain settings
    // modify here
    settings.accept_broadcast = false;
    settings.block_threshold = 100;
    settings.block_timeout = 10;
    settings.burning_fuel_method = 'consume_fuel';

    return settings;
}

// check whether an account can be a validator
function canBeValidator(accId: string): bool {
    // by default everyone can be a validator
    return true;
}

// ========================== SETTINGS END ==========================

const nullByte: u8 = 0xc0;
const trueByte: u8 = 0xc3;
const falseByte: u8 = 0xc2;

let thisAccount: string = '';

export function alloc(size: i32): i32 {
    return heap.alloc(size) as i32;
}

function arrayBufferToHexString(ab: ArrayBuffer): string {
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

function isDelegatedTo(delegator: string, delegate: string, action: string, data: u8[]): bool {
    let result = false;
    let callArgs = new IsDelegatedToArgs();
    callArgs.delegate = delegate;
    callArgs.action = action;
    callArgs.data = sdk.Utils.u8ArrayToArrayBuffer(data);
    let callArgsU8: u8[] = sdk.MsgPack.serialize<IsDelegatedToArgs>(callArgs);
    let callResult = sdk.HostFunctions.call(delegator, 'is_delegated_to', callArgsU8);
    if (callResult.success) {
        let resultBytes = sdk.Utils.arrayBufferToU8Array(callResult.result);
        if (resultBytes.length > 0 && resultBytes[0] == 0xc3) {
            result = true;
        }
    }

    return result;
}

export function run(ctxAddress: i32, ctxSize: i32, argsAddress: i32, argsSize: i32): sdk.Types.TCombinedPtr {
    let ctxU8Arr: u8[] = sdk.MemUtils.u8ArrayFromMem(ctxAddress, ctxSize);
    let ctx = sdk.MsgPack.ctxDecode(ctxU8Arr);
    thisAccount = ctx.owner;
    let argsU8: u8[] = sdk.MemUtils.u8ArrayFromMem(argsAddress, argsSize);
    let methodsMap = new Map<string, (ctx: sdk.Types.AppContext, args: u8[])=>sdk.Types.TCombinedPtr>();

    // Init function registered as callable by transaction's "method" field
    // comment the following line to make init callable only from inside core's code
    methodsMap.set('init', initInternal);
    methodsMap.set('contract_registration', contractRegistration);
    methodsMap.set('service_contract_update', updateServiceSmartContract);
    methodsMap.set('add_preapproved_contract', addPreapprovedContract);
    methodsMap.set('remove_preapproved_contract', removePreapprovedContracts);

    methodsMap.set('add_admins', addAdmins);
    methodsMap.set('remove_admins', removeAdmins);

    methodsMap.set('is_validator', isValidator);
    methodsMap.set('add_validators', addValidators);
    methodsMap.set('remove_validators', removeValidators);
    methodsMap.set('increase_validators_stakes', increaseValidatorsStakes);
    methodsMap.set('decrease_validators_stakes', decreaseValidatorsStakes);

    methodsMap.set('mint', mint);
    methodsMap.set('burn', burn);
    methodsMap.set('transfer', transfer);
    methodsMap.set('balance', balance);

    methodsMap.set('consume_fuel', consumeFuel);
    methodsMap.set('redistribute_remainders', redistributeRemainders);

    if (!methodsMap.has(ctx.method)) {
        let test: string = '';
        let success = false;
        let resultBytes = sdk.Utils.stringtoU8Array('Method not found.');
        return sdk.MsgPack.appOutputEncode(success, resultBytes);
    }
    return methodsMap.get(ctx.method)(ctx, argsU8);
}

// this is callable from wasm module's exports directly by core's code, just like any other contract's "run"
export function init(ctxAddress: i32, ctxSize: i32, argsAddress: i32, argsSize: i32): sdk.Types.TCombinedPtr {
    let ctxU8Arr: u8[] = sdk.MemUtils.u8ArrayFromMem(ctxAddress, ctxSize);
    let ctx = sdk.MsgPack.ctxDecode(ctxU8Arr);
    thisAccount = ctx.owner;
    // We should have received binary code of this very smart contract for self registration
    let argsU8: u8[] = sdk.MemUtils.u8ArrayFromMem(argsAddress, argsSize);
    return initInternal(ctx, argsU8);
}

// This is actual initialization function which gets called by "run" or
// "init" (depending on the smart contract invocation method).
// Here we need args in order for this function to be callable also through "run" even if they aren't used
function initInternal(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    // first let's check if settings were already defined (any data under the settings key)
    // this means init was already executed, so exit early.
    let blockchainSettingsBytes = sdk.HostFunctions.loadData(blockchainSettingsSectionKey);
    if (blockchainSettingsBytes.length > 0) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.INIT_DONE));
    }

    // saving settings
    sdk.HostFunctions.storeData(blockchainSettingsSectionKey, sdk.MsgPack.serialize(getSettingsObject()));

    // saving default validators
    let defValAccKeys: string[] = defaultValidatorsAccounts.keys();
    for (let i = 0; i < defValAccKeys.length; i++) {
        let stake = defaultValidatorsAccounts.get(defValAccKeys[0]);
        if (stake > 0) {
            let defaultValidatorKey = `${blockchainValidatorsSectionKey}:${defValAccKeys[0]}`;
            sdk.HostFunctions.storeData(defaultValidatorKey, serializeU64(stake));
        }
    }

    // saving the transaction submitter as first admin
    sdk.HostFunctions.storeData(blockchainAdminsSectionKey, serializeStringArray([ctx.caller]));

    // self registration
    let selfRegistrationArgs = new ContractRegistrationArgs();
    selfRegistrationArgs.name = THIS_NAME;
    selfRegistrationArgs.version = THIS_VERSION;
    selfRegistrationArgs.description = THIS_DESCRIPTION;
    selfRegistrationArgs.url = THIS_URL;
    selfRegistrationArgs.bin = sdk.Utils.u8ArrayToArrayBuffer(argsU8);
    let selfRegisterResult = contractRegistrationInternal(ctx, selfRegistrationArgs, false);

    if (!selfRegisterResult.success) {
        return sdk.MsgPack.appOutputEncode(
            selfRegisterResult.success,
            sdk.Utils.stringtoU8Array('self registration error: ').concat(selfRegisterResult.result),
        );
    }

    let assetStats = new FuelAssetStats();
    writeAssetStats(assetStats);

    let resultBytes: u8[] = [0xc0];
    return sdk.MsgPack.appOutputEncode(true, resultBytes);
}

function readPreapprovedList(): string[] {
    let preapprovedBytes = sdk.HostFunctions.loadData(preapprovedContractsSectionName);
    let result: string[] = preapprovedBytes.length > 0 ? deserializeStringArray(preapprovedBytes) : [];
    return result;
}

function writePreapprovedList(list: string[]): void {
    sdk.HostFunctions.storeData(preapprovedContractsSectionName, serializeStringArray(list));
}

function contractIsPreapproved(hash: string): bool {
    let preapprovedList = readPreapprovedList();
    if (preapprovedList.indexOf(hash) >= 0) {
        return true;
    }
    return false;
}

function addToPreapprovedListInternal(hashList: string[]): void {
    let preapprovedList = readPreapprovedList();
    for (let i = 0; i < hashList.length; i++) {
        if (preapprovedList.indexOf(hashList[i]) < 0) {
            preapprovedList.push(hashList[i]);
        }
    }
    writePreapprovedList(preapprovedList);
}

function removeFromPreapprovedListInternal(hashList: string[]): void {
    let preapprovedList = readPreapprovedList();
    let newList: string[] = [];
    for (let i = 0; i < preapprovedList.length; i++) {
        if (hashList.indexOf(preapprovedList[i]) < 0) {
            newList.push(preapprovedList[i]);
        }
    }
    if (newList.length != preapprovedList.length) {
        writePreapprovedList(newList);
    }
}

function addPreapprovedContract(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.origin)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    addToPreapprovedListInternal(deserializeStringArray(argsU8));
    return sdk.MsgPack.appOutputEncode(true, [0xc0]);
}

function removePreapprovedContracts(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.origin)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    removeFromPreapprovedListInternal(deserializeStringArray(argsU8));
    return sdk.MsgPack.appOutputEncode(true, [0xc0]);
}

function contractRegistration(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    let args: ContractRegistrationArgs = sdk.MsgPack.deserialize<ContractRegistrationArgs>(argsU8);
    let internalResult = contractRegistrationInternal(ctx, args);
    return sdk.MsgPack.appOutputEncode(internalResult.success, internalResult.result);
}

function updateServiceSmartContract(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if(!thisCanBeUpdated) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.UPDATE_NOT_ENABLED));
    }
    if(!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let args: ContractRegistrationArgs = sdk.MsgPack.deserialize<ContractRegistrationArgs>(argsU8);
    let internalResult = contractRegistrationInternal(ctx, args, false);
    if(internalResult.success) {
        sdk.HostFunctions.emit('service_contract_update',sdk.HostFunctions.loadData(`${contractsVersionsSectionKey}:${args.name}:${args.version}`));
    }
    return sdk.MsgPack.appOutputEncode(internalResult.success, internalResult.result);
}

function contractRegistrationInternal(ctx: sdk.Types.AppContext, args: ContractRegistrationArgs, checkPermissions: bool = true): RawAppOutput {
    let metaData: ContractRegistrationData = new ContractRegistrationData();
    metaData.name = args.name;
    metaData.version = args.version;
    metaData.description = args.description;
    metaData.url = args.url;
    metaData.publisher = ctx.caller;

    let contractHashBin = sdk.HostFunctions.sha256(sdk.Utils.arrayBufferToU8Array(args.bin));
    let contractMutliHashBin: u8[] = [0x12 as u8, 0x20 as u8].concat(contractHashBin);
    let contractHashStr = arrayBufferToHexString(sdk.Utils.u8ArrayToArrayBuffer(contractHashBin));
    let contractMutliHashStr = `1220${contractHashStr}`;

    let removeFromList = false;
    if((checkPermissions && !everyoneCanPublish) && !isAdmin(ctx.origin)) {
        if(!contractIsPreapproved(contractMutliHashStr)) {
            return new RawAppOutput(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
        }
        removeFromList = true;
    }
    // compute keys
    let versionKey = `${contractsVersionsSectionKey}:${metaData.name}:${metaData.version}`;
    let codekey = `${contractsCodeSectionKey}:${contractMutliHashStr}`;
    let metadataKey = `${contractsMetadataSectionKey}:${contractMutliHashStr}`;

    // check if keys are already present
    if (sdk.HostFunctions.loadData(versionKey).length > 0) {
        return new RawAppOutput(false, sdk.Utils.stringtoU8Array(Errors.SAME_CONTRACT_NAME_VER));
    }
    if (sdk.HostFunctions.loadData(metadataKey).length > 0) {
        return new RawAppOutput(false, sdk.Utils.stringtoU8Array(Errors.SAME_CONTRACT_HASH));
    }

    // associate name and version to hash
    sdk.HostFunctions.storeData(versionKey, contractMutliHashBin);
    // store metadata
    sdk.HostFunctions.storeData(metadataKey, sdk.MsgPack.serialize<ContractRegistrationData>(metaData));
    // store code
    sdk.HostFunctions.storeData(codekey, sdk.Utils.arrayBufferToU8Array(args.bin));

    if (removeFromList) {
        removeFromPreapprovedListInternal([contractMutliHashStr]);
    }

    return new RawAppOutput(true, serializeString(contractMutliHashStr));
}

// checks whether an account is also admin
function isAdmin(accId: string): bool {
    if(accId == thisAccount) {
        return true;
    }
    let adminsList: string[] = [];
    const adminsListBytes = sdk.HostFunctions.loadData(blockchainAdminsSectionKey);
    if (adminsListBytes.length > 0) {
        adminsList = deserializeStringArray(adminsListBytes);
    }
    if (adminsList.length > 0 && adminsList.indexOf(accId) >= 0) {
        return true;
    }
    return false;
}

function addAdmins(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let addList: string[] = []
    if (argsU8.length > 0) {
        addList = deserializeStringArray(argsU8);
    }
    let currList: string[] = [];
    const currListBytes = sdk.HostFunctions.loadData(blockchainAdminsSectionKey);
    if(currListBytes.length > 0) {
        currList = deserializeStringArray(currListBytes);
    }
    let initialLength = currList.length;
    if(addList.length > 0) {
        for (let i = 0; i < addList.length; i++) {
            if (currList.indexOf(addList[i]) < 0) {
                currList.push(addList[i]);
            }
        }
        if(currList.length != initialLength) {
            sdk.HostFunctions.storeData(blockchainAdminsSectionKey, serializeStringArray(currList));
        }
    }
    let result: u8[] = [0xc3];
    return sdk.MsgPack.appOutputEncode(true, result);
}

function removeAdmins(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let removeList = deserializeStringArray(argsU8);
    let currList: string[] = [];
    let newList: string[] = [];
    let currListBytes = sdk.HostFunctions.loadData(blockchainAdminsSectionKey);
    if(currListBytes.length > 0) {
        currList = deserializeStringArray(currListBytes);
    }
    if (removeList.length > 0 && currList.length > 0) {
        for (let i = 0; i < currList.length; i++) {
            if(removeList.indexOf(currList[i]) < 0) {
                newList.push(currList[i]);
            }
        }
        if (newList.length < 1) {
            return sdk.MsgPack.appOutputEncode(
                false,
                sdk.Utils.stringtoU8Array(Errors.NO_EMPTY_ADMINS),
            );
        }
        if(newList.length != currList.length) {
            sdk.HostFunctions.storeData(blockchainAdminsSectionKey, serializeStringArray(newList));
        }
    }
    let result: u8[] = [0xc3];
    return sdk.MsgPack.appOutputEncode(true, result);
}

function isValidator(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    let acc = deserializeString(argsU8);
    let internalResult = isValidatorInternal(acc);
    return sdk.MsgPack.appOutputEncode(internalResult.success, internalResult.result);
}

function isValidatorInternal(account: string): RawAppOutput {
    let result: u8[] = [0xc3];
    let validatorKey = `${blockchainValidatorsSectionKey}:${account}`;
    let validatorBytes = sdk.HostFunctions.loadData(validatorKey);
    if (validatorBytes.length < 1) {
        if (defaultValidatorsAccounts.keys().indexOf(account) < 0) {
            result = [0xc2]
        }
    }
    return new RawAppOutput(true, result);
}

function addValidators(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let validatorsMap = deserializeValidatorsMap(argsU8);
    let validatorsKeys: string[] = validatorsMap.keys();
    for (let i = 0; i < validatorsKeys.length; i++) {
        if (validatorsMap.get(validatorsKeys[i]) < 1) {
            continue;
        }
        if (!canBeValidator(validatorsKeys[i])) {
            let errorMessage = `${validatorsKeys[i]} cannot be a validator.`;
            return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(errorMessage));
        }
        let key = `${blockchainValidatorsSectionKey}:${validatorsKeys[i]}`;
        sdk.HostFunctions.storeData(key, serializeU64(validatorsMap.get(validatorsKeys[i])));
    }
    return sdk.MsgPack.appOutputEncode(true, [0xc0]);
}

function removeValidators(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let accList = deserializeStringArray(argsU8);
    for (let i = 0; i < accList.length; i++) {
        let key = `${blockchainValidatorsSectionKey}:${accList[i]}`;
        sdk.HostFunctions.removeData(key);
    }
    return sdk.MsgPack.appOutputEncode(true, [0xc0]);
}

function increaseValidatorsStakes(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let validatorsMap = deserializeValidatorsMap(argsU8);
    let internalReturn = increaseValidatorsStakesInternal(validatorsMap);
    return sdk.MsgPack.appOutputEncode(internalReturn.success, internalReturn.result);
}

function increaseValidatorsStakesInternal(validatorsMap: Map<string, u64>): RawAppOutput {
    let validatorsKeys: string[] = validatorsMap.keys();
    for (let i = 0; i < validatorsKeys.length; i++) {
        let newValue: u64 = validatorsMap.get(validatorsKeys[i]);
        if (newValue <= 0) {
            continue;
        }
        if (!canBeValidator(validatorsKeys[i])) {
            let errorMessage = `${validatorsKeys[i]} cannot be a validator.`;
            return new RawAppOutput(false, sdk.Utils.stringtoU8Array(errorMessage));
        }
        let key = `${blockchainValidatorsSectionKey}:${validatorsKeys[i]}`;
        let currValueBytes = sdk.HostFunctions.loadData(key);
        if (currValueBytes.length > 0) {
            let currValue = deserializeU64(currValueBytes);
            newValue += currValue;
        }
        sdk.HostFunctions.storeData(key, serializeU64(newValue));
    }
    return new RawAppOutput(true, [0xc0]);
}

function decreaseValidatorsStakes(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let validatorsMap = deserializeValidatorsMap(argsU8);
    let internalReturn = decreaseValidatorsStakesInternal(validatorsMap);
    return sdk.MsgPack.appOutputEncode(internalReturn.success, internalReturn.result);
}

function decreaseValidatorsStakesInternal(validatorsMap: Map<string, u64>): RawAppOutput {
    let validatorsKeys: string[] = validatorsMap.keys();
    for (let i = 0; i < validatorsKeys.length; i++) {
        if (!canBeValidator(validatorsKeys[i])) {
            let errorMessage = `${validatorsKeys[i]} cannot be a validator.`;
            return new RawAppOutput(false, sdk.Utils.stringtoU8Array(errorMessage));
        }
        let newValue: u64 = validatorsMap.get(validatorsKeys[i]);
        if (newValue <= 0) {
            continue;
        }
        let key = `${blockchainValidatorsSectionKey}:${validatorsKeys[i]}`;
        let currValueBytes = sdk.HostFunctions.loadData(key);
        if (currValueBytes.length > 0) {
            let currValue = deserializeU64(currValueBytes);
            if (newValue >= currValue) {
                sdk.HostFunctions.removeData(key);
            } else {
                newValue = currValue - newValue;
                sdk.HostFunctions.storeData(key, serializeU64(newValue));
            }
        }
    }
    return new RawAppOutput(true, [0xc0]);
}

function readAssetStats(): FuelAssetStats {
    let statsBytes = sdk.HostFunctions.loadData(fuelAssetStatsSectionName);
    let stats = new FuelAssetStats();
    if (statsBytes.length > 0) {
        stats = sdk.MsgPack.deserialize<FuelAssetStats>(statsBytes);
    }
    return stats;
}

function writeAssetStats(stats: FuelAssetStats): void {
    let statsBytes = sdk.MsgPack.serialize<FuelAssetStats>(stats);
    sdk.HostFunctions.storeData(fuelAssetStatsSectionName ,statsBytes);
}

function balance(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    const accId = deserializeString(argsU8);
    return sdk.MsgPack.appOutputEncode(true, sdk.HostFunctions.loadAsset(accId));
}

function readBalance(account: string): u64 {
    let balanceBytes = sdk.HostFunctions.loadAsset(account);
    let balance: u64 = balanceBytes.length > 0 ? deserializeU64(balanceBytes) : 0;
    return balance;
}

function writeBalance(account: string, newBalance: u64): void {
    let balanceBytes = serializeU64(newBalance);
    sdk.HostFunctions.storeAsset(account, balanceBytes);
}

function mint(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    const args = sdk.MsgPack.deserialize<MintArgs>(argsU8);
    if (args.to == remainderAccount) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let internalReturn = mintInternal(args.to, args.units);
    return sdk.MsgPack.appOutputEncode(internalReturn.success, internalReturn.result);
}

function mintInternal(to: string, units: u64, updateStats: bool = true): RawAppOutput {
    const currBalance = readBalance(to);
    sdk.HostFunctions.storeAsset(to, serializeU64(currBalance + units));
    if (updateStats && units > 0) {
        let assetStats = readAssetStats();
        assetStats.circulating_volume += units;
        writeAssetStats(assetStats);
    }
    return new RawAppOutput(true, [0xc3]);
}

function burn(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if (!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    const args = sdk.MsgPack.deserialize<BurnArgs>(argsU8);
    if (args.from == remainderAccount) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let internalResult = burnInternal(args.from, args.units);
    return sdk.MsgPack.appOutputEncode(internalResult.success, internalResult.result);
}

function burnInternal(from: string, units: u64, updateStats: bool = true): RawAppOutput {
    const currBalance = readBalance(from);
    if (currBalance < units) {
        return new RawAppOutput(false, sdk.Utils.stringtoU8Array(Errors.INSUFFICIENT_FUNDS));
    }
    sdk.HostFunctions.storeAsset(from, serializeU64(currBalance - units));
    if (updateStats && units > 0) {
        let assetStats = readAssetStats();
        assetStats.circulating_volume = assetStats.circulating_volume >= units ? assetStats.circulating_volume - units : 0;
        writeAssetStats(assetStats);
    }
    return new RawAppOutput(true, [0xc3]);
}

function transfer(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    const args = sdk.MsgPack.deserialize<TransferArgs>(argsU8);
    if (args.from == remainderAccount || args.to == remainderAccount) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    if (
        args.from == remainderAccount
        || args.to == remainderAccount
        || (
            ctx.caller != args.from
            && !isDelegatedTo(args.from, ctx.caller, 'transfer', argsU8)
        )
    ) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let internalResult = transferInternal(args.from, args.to, args.units);
    return sdk.MsgPack.appOutputEncode(internalResult.success, internalResult.result);
}

function transferInternal(from: string, to: string, units: u64): RawAppOutput {
    const currFromBalance = readBalance(from);
    if (currFromBalance < units) {
        return new RawAppOutput(false, sdk.Utils.stringtoU8Array(Errors.INSUFFICIENT_FUNDS));
    }
    const currToBalance = readBalance(to);
    writeBalance(from, currFromBalance - units);
    writeBalance(to, currToBalance + units);
    return new RawAppOutput(true, trueBytes);
}

function consumeFuel(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if(ctx.caller != ctx.owner) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    let args = sdk.MsgPack.deserialize<ConsumeFuelArgs>(argsU8);
    // if transaction was performed by myself, do nothing. otherwise it's a loop
    if (args.from == ctx.owner) {
        return sdk.MsgPack.appOutputEncode(true, [0xc3]);
    }
    let internalResult = consumeFuelInternal(args.from, args.units);
    return sdk.MsgPack.appOutputEncode(internalResult.success, internalResult.result);
}

function consumeFuelInternal(from: string, units: u64): RawAppOutput {
    let resultBytes: u8[] = trueBytes;

    //check supplier's balance
    const fromBalance = readBalance(from);

    // if supplier has less than needed, then burn that amount
    // also transaction won't pe executed because of the "false" return
    let unitsToBurn = units;
    if (unitsToBurn > fromBalance) {
        unitsToBurn = fromBalance;
        resultBytes = falseBytes;
    }
    if (unitsToBurn <= 0) {
        return new RawAppOutput(true, resultBytes);
    }

    const validatorsKeysList: string[] = sdk.HostFunctions.getKeys(`${blockchainValidatorsSectionKey}:*`);
    const validatorsAccounts: string[] = [];
    const validatorsStakes: u64[] = [];
    for (let i = 0; i < validatorsKeysList.length; i++) {
        let validatorStakeBytes = sdk.HostFunctions.loadData(validatorsKeysList[i]);
        if (validatorStakeBytes.length > 0) {
            let validatorStakeValue = deserializeU64(validatorStakeBytes);
            if (validatorStakeValue > 0) {
                validatorsAccounts.push(validatorsKeysList[i].substr(blockchainValidatorsSectionKey.length + 1));
                validatorsStakes.push(validatorStakeValue);
            }
        }
    }
    let stakesSum = validatorsStakes.reduce(
        (acc: u64, nextItem: u64) => {
            return acc + nextItem;
        },
        0 as u64,
    );

    let unitsToAdd = new Array<u64>(validatorsAccounts.length).fill(0);
    for (let i = 0; i < validatorsStakes.length; i++) {
        // calculating individual validators shares of the burned gas using
        // a proportion. <burned> : <validatorShare> = <stakesSum> : <currentStake>
        unitsToAdd[i] = unitsToBurn*validatorsStakes[i]/stakesSum;
    }

    let remainder = unitsToBurn - unitsToAdd.reduce(
        (acc: u64, nextItem: u64) => {
            return acc + nextItem;
        },
        0 as u64,
    );

    burnInternal(from, unitsToBurn, false);

    // minting gas directly to validators
    for (let i = 0; i < unitsToAdd.length; i++) {
        mintInternal(validatorsAccounts[i], unitsToAdd[i]);
    }
    // minting remainder to a specially designated account
    if (remainder > 0) {
        mintInternal(remainderAccount, remainder);
    }
    return new RawAppOutput(true, resultBytes);
}

function redistributeRemainders(ctx: sdk.Types.AppContext, argsU8: u8[]): sdk.Types.TCombinedPtr {
    if(!isAdmin(ctx.caller)) {
        return sdk.MsgPack.appOutputEncode(false, sdk.Utils.stringtoU8Array(Errors.NOT_AUTHORIZED));
    }
    //check remainders balance
    const remainderAccountBalance = readBalance(remainderAccount) //sdk.HostFunctions.loadAsset(remainderAccount);
    const internalResult = consumeFuelInternal(remainderAccount, remainderAccountBalance);
    return sdk.MsgPack.appOutputEncode(internalResult.success, internalResult.result);
}
