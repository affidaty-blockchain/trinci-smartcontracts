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

//! Basic Asset
//!
//! Asset smart contract exposing a fairly standard behaviour.
//! The interface respects the TAI interface.
//!
//! ### Rules
//!
//! 1. Initialization can be performed by anyone.
//! 2. Minting can be performed only by the asset creator or an authorized account.
//! 4. Burning can be performed only by the asset creator or an authorized account.
//! 5. Funds transfer can be performed only by the asset creator or by the caller
//!    if is the same of the from args field.
//! 6. Balance and Stats can be called by anyone.
//!
//! Note about rule 5. The asset creator is allowed to transfer funds from
//! others accounts to allow asset seizure when is necessary.

use trinci_sdk::{
    rmp_deserialize, rmp_serialize_named, AppContext, PackedValue, WasmError, WasmResult,
};
mod types;
use types::*;

trinci_sdk::app_export!(
    balance,
    burn,
    init,
    mint,
    stats,
    transfer,
    lock,
    add_delegation
);

const CONFIG_KEY: &str = "config";
const INIT_KEY: &str = "init";
const DELEGATIONS_KEY: &str = "delegations";

#[inline]
fn is_initialized() -> bool {
    !trinci_sdk::load_data(INIT_KEY).is_empty()
}

#[inline]
fn init_check() -> WasmResult<()> {
    match is_initialized() {
        true => Ok(()),
        false => Err(WasmError::new("contract not initialized")),
    }
}

/// Load asset structure and check for lock.
fn load_asset_checked(from: &str, shall_not_contain: LockType) -> WasmResult<Asset> {
    let asset: Asset = trinci_sdk::load_asset_typed(from);
    let (lock_privilege, lock_type) = asset.lock.unwrap_or_default();

    let lock_type_flg = lock_type as u8;
    let shall_not_contain_flg = shall_not_contain as u8;

    if (lock_type_flg & shall_not_contain_flg) != 0 {
        let priv_str = match lock_privilege {
            LockPrivilege::Owner => "owner",
            LockPrivilege::Contract => "contract",
            LockPrivilege::Creator => "creator",
        };
        let op_str = match shall_not_contain {
            LockType::Deposit => "deposit",
            LockType::Withdraw => "withdraw",
            LockType::Full => "balance",
            _ => "", // never happens by construction
        };
        let msg = format!("asset {} locked by {}", op_str, priv_str);
        Err(WasmError::new(&msg))
    } else {
        Ok(asset)
    }
}

/// Withdraw some asset units from the given account.
///
/// Returns error if the account is locked.
fn withdraw(from: &str, units: u64) -> WasmResult<()> {
    let mut asset = load_asset_checked(from, LockType::Withdraw)?;
    if asset.units < units {
        return Err(WasmError::new("insufficient funds"));
    }
    asset.units -= units;
    trinci_sdk::store_asset_typed(from, asset);
    Ok(())
}

/// Deposit some asset units to the given account.
///
/// Returns error if the account is locked.
fn deposit(to: &str, units: u64) -> WasmResult<()> {
    let mut asset = load_asset_checked(to, LockType::Deposit)?;
    asset.units += units;
    trinci_sdk::store_asset_typed(to, asset);
    Ok(())
}

/// Initialize the asset status.
///
/// The caller become the asset creator.
fn init(ctx: AppContext, args: InitArgs) -> WasmResult<()> {
    if is_initialized() {
        return Ok(());
    }

    let config = AssetConfig {
        name: args.name,
        creator: ctx.caller,
        description: args.description,
        url: args.url,
        max_units: args.max_units,
        minted: 0,
        burned: 0,
        authorized: args.authorized,
    };

    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;
    trinci_sdk::store_data(INIT_KEY, &[1]);

    Ok(())
}

/// Returns the balance of the asset in the caller account.
fn balance(ctx: AppContext, _args: PackedValue) -> WasmResult<u64> {
    init_check()?;

    let asset = load_asset_checked(ctx.caller, LockType::Full)?;
    Ok(asset.units)
}

/// Returns the stats of the asset.
fn stats(_ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: AssetConfig = rmp_deserialize(&buf)?;
    let buf = rmp_serialize_named(&config)?;
    Ok(PackedValue(buf))
}

/// Mint asset and transfer it to an account.
fn mint(ctx: AppContext, args: MintArgs) -> WasmResult<()> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let mut config: AssetConfig = trinci_sdk::rmp_deserialize(&buf)?;

    if (ctx.depth == 0 && ctx.caller != config.creator && !config.authorized.contains(&ctx.caller))
        || (ctx.depth > 0
            && ctx.origin != config.creator
            && !config.authorized.contains(&ctx.origin))
    {
        return Err(WasmError::new("not authorized"));
    }

    if config.minted + args.units > config.max_units {
        return Err(WasmError::new("minting overcome the max_units value"));
    }

    deposit(args.to, args.units)?;

    config.minted += args.units;
    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;
    Ok(())
}

/// Destroy asset from an account.
fn burn(ctx: AppContext, args: BurnArgs) -> WasmResult<()> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let mut config: AssetConfig = trinci_sdk::rmp_deserialize(&buf)?;

    if (ctx.depth == 0 && ctx.caller != config.creator && !config.authorized.contains(&ctx.caller))
        || (ctx.depth > 0
            && ctx.origin != config.creator
            && !config.authorized.contains(&ctx.origin))
    {
        return Err(WasmError::new("not authorized"));
    }

    withdraw(args.from, args.units)?;

    config.burned += args.units;
    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;
    Ok(())
}

/// Transfer the asset from an account to another.
fn transfer(ctx: AppContext, args: TransferArgs) -> WasmResult<()> {
    init_check()?;

    if ctx.caller != args.from && ctx.caller != ctx.owner {
        // Check if there is an authorization
        let buf = trinci_sdk::load_data(DELEGATIONS_KEY);
        let mut delegations: DelegationsData = rmp_deserialize(&buf).unwrap_or_default();

        if let Some(caller_delegation) = delegations.get_mut(ctx.caller) {
            if let Some(index) = caller_delegation.iter_mut().position(|delegation| {
                (delegation.to == Some(args.to) || delegation.to.is_none())
                    && delegation.units == args.units
            }) {
                caller_delegation.remove(index);

                // Saving the delegations after removing the used one
                trinci_sdk::store_account_data_mp!(DELEGATIONS_KEY, &delegations)?;
            } else {
                return Err(WasmError::new("not authorized"));
            };
        } else {
            return Err(WasmError::new("not authorized"));
        }
    }

    withdraw(args.from, args.units)?;
    deposit(args.to, args.units)?;

    Ok(())
}

/// Lock the asset.
///
/// A locked asset cannot be moved from or into the account.
/// The lock type is inferred from the caller.
/// Rules:
///  - owner can't unlock an asset locked by a contract or the asset creator.
///  - creator can unlock an asset locked by the asset owner or a contract.
fn lock(ctx: AppContext, args: LockArgs) -> WasmResult<()> {
    let mut asset: Asset = trinci_sdk::load_asset_typed(args.to);

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: AssetConfig = trinci_sdk::rmp_deserialize(&buf)?;

    // Deduce the request privilege
    let request_privilege = if ctx.caller == config.creator {
        LockPrivilege::Creator
    } else if ctx.caller == args.to {
        if ctx.depth != 0 {
            LockPrivilege::Contract
        } else {
            LockPrivilege::Owner
        }
    } else {
        return Err(WasmError::new("not authorized"));
    };

    let request_privilege_level = request_privilege as u8;
    let required_privilege_level = asset
        .lock
        .map(|(locktype, _)| locktype as u8)
        .unwrap_or_default();
    if required_privilege_level > request_privilege_level {
        return Err(WasmError::new("not authorized"));
    }

    asset.lock = match args.lock {
        LockType::None => None,
        lock_type => Some((request_privilege, lock_type)),
    };
    trinci_sdk::store_asset_typed(args.to, asset);
    Ok(())
}

/// Transfer the asset from an account to another.
fn add_delegation(ctx: AppContext, args: AddDelegationArgs) -> WasmResult<()> {
    init_check()?;

    // Load the delegations and add the delegation
    let buf = trinci_sdk::load_data(DELEGATIONS_KEY);
    let mut delegations: DelegationsData = rmp_deserialize(&buf).unwrap_or_default();

    let mut delegate = match delegations.get_mut(args.delegate) {
        Some(d) => d.to_owned(),
        None => Vec::<DelegateTransferArgs>::default(),
    };

    let transfer_delegation = DelegateTransferArgs {
        from: ctx.caller,
        to: args.to,
        units: args.units,
    };

    delegate.push(transfer_delegation);

    delegations.insert(args.delegate, delegate.to_owned());

    // Save the delegations
    trinci_sdk::store_account_data_mp!(DELEGATIONS_KEY, &delegations)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use trinci_sdk::{not_wasm, rmp_serialize};

    use super::*;

    const CALLER_ID: &str = "QmSCRCPFznxEX6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq";
    const DESTINATION_ID: &str = "QmDestination1Hsj5Eb5DnbR1hDZRFpRrQVeQZHkibuEp";
    const DELEGATE_ID: &str = "QmDelegateQH99ydqr7Ci1Hsj5Eb5DnbR1hDZRFpRrQVeQ";
    const OWNER_ID: &str = "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";
    const ASSET_ID: &str = "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i";
    const AUTH_ACCOUNT_1: &str = "QmxACC1Ldf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPf123";
    const AUTH_ACCOUNT_2: &str = "QmxACC2Ldf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfxyz";
    const CONTRACT_ID: &str = "QmContract_h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfxyz";

    const EMPTIED_DELEGATION_DATA_HEX: &str = "81d92e516d44656c656761746551483939796471723743693148736a35456235446e62523168445a52467052725156655190";

    fn create_init_args() -> InitArgs<'static> {
        InitArgs {
            name: "FCK",
            description: "ipse lorem",
            url: "www.xyz.com",
            max_units: 1000,
            authorized: vec![AUTH_ACCOUNT_1, AUTH_ACCOUNT_2],
        }
    }

    fn create_asset_config() -> AssetConfig<'static> {
        AssetConfig {
            name: "FCK",
            creator: OWNER_ID,
            description: "ipse lorem",
            url: "www.xyz.com",
            max_units: 1000,
            minted: 100,
            burned: 10,
            authorized: vec![AUTH_ACCOUNT_1, AUTH_ACCOUNT_2],
        }
    }

    fn prepare_full_env() -> AppContext<'static> {
        let config = create_asset_config();
        let data = trinci_sdk::rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(ASSET_ID, CONFIG_KEY, &data);
        not_wasm::set_account_data(ASSET_ID, INIT_KEY, &[1]);
        not_wasm::create_app_context(OWNER_ID, CALLER_ID)
    }

    #[test]
    fn init_contract() {
        let mut ctx = not_wasm::create_app_context(OWNER_ID, CALLER_ID);
        ctx.caller = OWNER_ID;

        let args: InitArgs = create_init_args();

        not_wasm::call_wrap(init, ctx, args).unwrap();
    }

    #[test]
    fn owner_lock_all_test() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Full,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Full)));
    }

    #[test]
    fn owner_lock_in_test() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Deposit,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Deposit)));
    }

    #[test]
    fn owner_lock_out_test() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Withdraw,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.lock, Some((LockPrivilege::Owner, LockType::Withdraw)));
    }

    #[test]
    fn contract_lock_all_test() {
        let mut ctx = prepare_full_env();
        ctx.depth = 1;
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Full,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.lock, Some((LockPrivilege::Contract, LockType::Full)));
    }

    #[test]
    fn contract_lock_in_test() {
        let mut ctx = prepare_full_env();
        ctx.depth = 1;
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Deposit,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(
            asset.lock,
            Some((LockPrivilege::Contract, LockType::Deposit))
        );
    }

    #[test]
    fn contract_lock_out_test() {
        let mut ctx = prepare_full_env();
        ctx.depth = 1;
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Withdraw,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(
            asset.lock,
            Some((LockPrivilege::Contract, LockType::Withdraw))
        );
    }

    #[test]
    fn creator_lock_all_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = ASSET_ID;
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Full,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.lock, Some((LockPrivilege::Creator, LockType::Full)));
    }

    #[test]
    fn creator_lock_in_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = ASSET_ID;
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Deposit,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(
            asset.lock,
            Some((LockPrivilege::Creator, LockType::Deposit))
        );
    }

    #[test]
    fn creator_lock_out_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = ASSET_ID;
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::Withdraw,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(
            asset.lock,
            Some((LockPrivilege::Creator, LockType::Withdraw))
        );
    }

    #[test]
    fn owner_unlock_test() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(
            CALLER_ID,
            ASSET_ID,
            Asset {
                units: 10,
                lock: Some((LockPrivilege::Owner, LockType::Deposit)),
            },
        );
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::None,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.lock, None);
    }

    #[test]
    fn creator_unlock_test_failure() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(
            CALLER_ID,
            ASSET_ID,
            Asset {
                units: 10,
                lock: Some((LockPrivilege::Creator, LockType::Withdraw)),
            },
        );
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::None,
        };

        let err = not_wasm::call_wrap(lock, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn creator_unlock_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = ASSET_ID;
        not_wasm::set_account_asset_gen(
            CALLER_ID,
            ASSET_ID,
            Asset {
                units: 10,
                lock: Some((LockPrivilege::Creator, LockType::Full)),
            },
        );
        let args = LockArgs {
            to: CALLER_ID,
            lock: LockType::None,
        };

        not_wasm::call_wrap(lock, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.lock, None);
    }

    #[test]
    fn transfer_test() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));

        let args = TransferArgs {
            from: CALLER_ID,
            to: DESTINATION_ID,
            units: 30,
            data: None,
        };

        not_wasm::call_wrap(transfer, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.units, 70);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 30);
    }

    #[test]
    fn pay_without_funds() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(0));

        let args = TransferArgs {
            from: CALLER_ID,
            to: DESTINATION_ID,
            units: 30,
            data: None,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "insufficient funds");
    }

    #[test]
    fn pay_with_source_locked_in_asset() {
        let ctx = prepare_full_env();
        let asset = Asset {
            units: 100,
            lock: Some((LockPrivilege::Owner, LockType::Deposit)),
        };
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, asset);

        let args = TransferArgs {
            from: CALLER_ID,
            to: DESTINATION_ID,
            units: 30,
            data: None,
        };

        not_wasm::call_wrap(transfer, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(CALLER_ID, ASSET_ID);
        assert_eq!(asset.units, 70);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 30);
    }

    #[test]
    fn pay_with_dest_locked_in_asset() {
        let ctx = prepare_full_env();

        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));

        let asset = Asset {
            units: 100,
            lock: Some((LockPrivilege::Owner, LockType::Deposit)),
        };

        not_wasm::set_account_asset_gen(DESTINATION_ID, ASSET_ID, asset);

        let args = TransferArgs {
            from: CALLER_ID,
            to: DESTINATION_ID,
            units: 30,
            data: None,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "asset deposit locked by owner");
    }

    #[test]
    fn pay_with_locked_all_asset() {
        let ctx = prepare_full_env();
        let asset = Asset {
            units: 10,
            lock: Some((LockPrivilege::Owner, LockType::Full)),
        };
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, asset);

        let args = TransferArgs {
            from: CALLER_ID,
            to: DESTINATION_ID,
            units: 5,
            data: None,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "asset withdraw locked by owner");
    }

    #[test]
    fn pay_with_locked_out_asset() {
        let ctx = prepare_full_env();
        let asset = Asset {
            units: 10,
            lock: Some((LockPrivilege::Owner, LockType::Withdraw)),
        };
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, asset);

        let args = TransferArgs {
            from: CALLER_ID,
            to: DESTINATION_ID,
            units: 5,
            data: None,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "asset withdraw locked by owner");
    }

    #[test]
    fn balance_test() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(42));

        let args = PackedValue::default();

        let res = not_wasm::call_wrap(balance, ctx, args).unwrap();

        assert_eq!(res, 42);
    }

    #[test]
    fn balance_with_locked_asset() {
        let ctx = prepare_full_env();
        let asset = Asset {
            units: 10,
            lock: Some((LockPrivilege::Creator, LockType::Full)),
        };
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, asset);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(balance, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "asset balance locked by creator");
    }

    #[test]
    fn stats_test() {
        let ctx = prepare_full_env();
        let args = PackedValue::default();

        let res = not_wasm::call_wrap(stats, ctx, args).unwrap();

        let config = create_asset_config();
        let buf = rmp_serialize_named(&config).unwrap();
        assert_eq!(res.0, buf);
    }

    #[test]
    fn mint_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = OWNER_ID;

        let args = MintArgs {
            to: DESTINATION_ID,
            units: 30,
        };

        not_wasm::call_wrap(mint, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(ASSET_ID, CONFIG_KEY);
        let config: AssetConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(config.minted, 130);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 30);
    }

    #[test]
    fn mint_too_much() {
        let mut ctx = prepare_full_env();
        ctx.caller = OWNER_ID;

        let args = MintArgs {
            to: DESTINATION_ID,
            units: 1000,
        };

        let err = not_wasm::call_wrap(mint, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "minting overcome the max_units value");
    }

    #[test]
    fn authorized_mint_from_contract_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = CONTRACT_ID;
        ctx.origin = AUTH_ACCOUNT_1;
        ctx.depth = 1;

        let args = MintArgs {
            to: DESTINATION_ID,
            units: 30,
        };

        not_wasm::call_wrap(mint, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(ASSET_ID, CONFIG_KEY);
        let config: AssetConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(config.minted, 130);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 30);
    }
    #[test]
    fn not_authorized_mint_from_contract_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = CONTRACT_ID;
        ctx.origin = CALLER_ID;
        ctx.depth = 1;

        let args = MintArgs {
            to: DESTINATION_ID,
            units: 30,
        };

        let err = not_wasm::call_wrap(mint, ctx, args).unwrap_err();
        assert_eq!("not authorized", err.to_string());
    }

    #[test]
    fn mint_from_authorized_account_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = AUTH_ACCOUNT_2;

        let args = MintArgs {
            to: DESTINATION_ID,
            units: 30,
        };

        not_wasm::call_wrap(mint, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(ASSET_ID, CONFIG_KEY);
        let config: AssetConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(config.minted, 130);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 30);
    }

    #[test]
    fn unauthorized_mint_test() {
        let ctx = prepare_full_env();

        let args = MintArgs {
            to: DESTINATION_ID,
            units: 30,
        };

        let err = not_wasm::call_wrap(mint, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn burn_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = OWNER_ID;

        not_wasm::set_account_asset_gen(DESTINATION_ID, ASSET_ID, Asset::new(100));

        let args = BurnArgs {
            from: DESTINATION_ID,
            units: 30,
        };

        not_wasm::call_wrap(burn, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(ASSET_ID, CONFIG_KEY);
        let config: AssetConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(config.burned, 40);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 70);
    }

    #[test]
    fn burn_from_authorized_account_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = AUTH_ACCOUNT_1;

        not_wasm::set_account_asset_gen(DESTINATION_ID, ASSET_ID, Asset::new(100));

        let args = BurnArgs {
            from: DESTINATION_ID,
            units: 30,
        };

        not_wasm::call_wrap(burn, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(ASSET_ID, CONFIG_KEY);
        let config: AssetConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(config.burned, 40);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 70);
    }

    #[test]
    fn burn_not_enough_funds() {
        let mut ctx = prepare_full_env();
        ctx.caller = OWNER_ID;

        not_wasm::set_account_asset_gen(DESTINATION_ID, ASSET_ID, Asset::new(10));

        let args = BurnArgs {
            from: DESTINATION_ID,
            units: 30,
        };

        let err = not_wasm::call_wrap(burn, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "insufficient funds");
    }

    #[test]
    fn burn_not_authorized() {
        let ctx = prepare_full_env();
        not_wasm::set_account_asset_gen(CALLER_ID, ASSET_ID, Asset::new(100));

        let args = BurnArgs {
            from: DESTINATION_ID,
            units: 30,
        };

        let err = not_wasm::call_wrap(burn, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn authorized_burn_from_smart_contract() {
        let mut ctx = prepare_full_env();
        ctx.caller = CONTRACT_ID;
        ctx.depth = 1;
        ctx.origin = AUTH_ACCOUNT_1;

        not_wasm::set_account_asset_gen(DESTINATION_ID, ASSET_ID, Asset::new(100));

        let args = BurnArgs {
            from: DESTINATION_ID,
            units: 30,
        };

        not_wasm::call_wrap(burn, ctx, args).unwrap();

        let buf = not_wasm::get_account_data(ASSET_ID, CONFIG_KEY);
        let config: AssetConfig = trinci_sdk::rmp_deserialize(&buf).unwrap();
        assert_eq!(config.burned, 40);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 70);
    }

    #[test]
    fn not_authorized_burn_from_smart_contract() {
        let mut ctx = prepare_full_env();
        ctx.caller = CONTRACT_ID;
        ctx.depth = 1;
        ctx.origin = CALLER_ID;

        not_wasm::set_account_asset_gen(DESTINATION_ID, ASSET_ID, Asset::new(100));

        let args = BurnArgs {
            from: DESTINATION_ID,
            units: 30,
        };

        let err = not_wasm::call_wrap(burn, ctx, args).unwrap_err();

        assert_eq!("not authorized", err.to_string());
    }

    #[test]
    fn add_delegation_test() {
        let ctx = prepare_full_env();

        let args: AddDelegationArgs = AddDelegationArgs {
            delegate: DELEGATE_ID,
            to: Some(DESTINATION_ID),
            units: 42,
        };

        not_wasm::call_wrap(add_delegation, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(ASSET_ID, DELEGATIONS_KEY);
        let delegations: DelegationsData = rmp_deserialize(&buf).unwrap();
        let delegation = delegations
            .get(DELEGATE_ID)
            .unwrap()
            .get(0)
            .unwrap()
            .to_owned();

        let expected = DelegateTransferArgs {
            from: CALLER_ID,
            to: args.to,
            units: args.units,
        };

        assert_eq!(delegation, expected);
    }

    #[test]
    fn add_other_delegation_test() {
        let ctx = prepare_full_env();

        let mut args: AddDelegationArgs = AddDelegationArgs {
            delegate: DELEGATE_ID,
            to: Some(DESTINATION_ID),
            units: 42,
        };

        let mut data = DelegationsData::new();
        data.insert(
            DELEGATE_ID,
            vec![DelegateTransferArgs {
                from: ctx.caller,
                to: args.to,
                units: args.units,
            }],
        );

        let data = rmp_serialize(&data).unwrap();

        not_wasm::set_account_data(OWNER_ID, DELEGATIONS_KEY, &data);

        args.units = 101;

        not_wasm::call_wrap(add_delegation, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(ASSET_ID, DELEGATIONS_KEY);
        let delegations: DelegationsData = rmp_deserialize(&buf).unwrap();
        let delegation_size = delegations.get(DELEGATE_ID).unwrap().len();
        assert_eq!(delegation_size, 2);
    }

    #[test]
    fn transfer_with_delegation_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = DELEGATE_ID;
        not_wasm::set_account_asset_gen(OWNER_ID, ASSET_ID, Asset::new(100));

        let mut delegations_data = DelegationsData::new();
        delegations_data.insert(
            DELEGATE_ID,
            vec![DelegateTransferArgs {
                from: OWNER_ID,
                to: Some(DESTINATION_ID),
                units: 42,
            }],
        );

        let data = rmp_serialize(&delegations_data).unwrap();

        not_wasm::set_account_data(OWNER_ID, DELEGATIONS_KEY, &data);

        let args = TransferArgs {
            from: OWNER_ID,
            to: DESTINATION_ID,
            units: 42,
            data: None,
        };

        not_wasm::call_wrap(transfer, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(OWNER_ID, ASSET_ID);
        assert_eq!(asset.units, 58);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 42);

        let deleg_data = not_wasm::get_account_data(OWNER_ID, DELEGATIONS_KEY);
        assert_eq!(EMPTIED_DELEGATION_DATA_HEX, hex::encode(&deleg_data));
    }

    #[test]
    fn transfer_with_bad_delegation_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = DESTINATION_ID;
        not_wasm::set_account_asset_gen(OWNER_ID, ASSET_ID, Asset::new(100));

        let mut delegations_data = DelegationsData::new();
        delegations_data.insert(
            DELEGATE_ID,
            vec![DelegateTransferArgs {
                from: ctx.caller,
                to: Some(DESTINATION_ID),
                units: 43, // Different from the args!
            }],
        );

        let data = rmp_serialize(&delegations_data).unwrap();

        not_wasm::set_account_data(OWNER_ID, DELEGATIONS_KEY, &data);

        let args = TransferArgs {
            from: OWNER_ID,
            to: DESTINATION_ID,
            units: 42,
            data: None,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn transfer_with_generic_to_delegation_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = DELEGATE_ID;
        not_wasm::set_account_asset_gen(OWNER_ID, ASSET_ID, Asset::new(100));

        let mut delegations_data = DelegationsData::new();
        delegations_data.insert(
            DELEGATE_ID,
            vec![DelegateTransferArgs {
                from: OWNER_ID,
                to: None,
                units: 42,
            }],
        );

        let data = rmp_serialize(&delegations_data).unwrap();

        not_wasm::set_account_data(OWNER_ID, DELEGATIONS_KEY, &data);

        let args = TransferArgs {
            from: OWNER_ID,
            to: DESTINATION_ID,
            units: 42,
            data: None,
        };

        not_wasm::call_wrap(transfer, ctx, args).unwrap();

        let asset: Asset = not_wasm::get_account_asset_gen(OWNER_ID, ASSET_ID);
        assert_eq!(asset.units, 58);
        let asset: Asset = not_wasm::get_account_asset_gen(DESTINATION_ID, ASSET_ID);
        assert_eq!(asset.units, 42);

        let deleg_data = not_wasm::get_account_data(OWNER_ID, DELEGATIONS_KEY);
        assert_eq!(EMPTIED_DELEGATION_DATA_HEX, hex::encode(&deleg_data));
    }

    #[test]
    fn transfer_with_generic_to_delegation_and_bad_delegation_test() {
        let mut ctx = prepare_full_env();
        ctx.caller = DESTINATION_ID;
        not_wasm::set_account_asset_gen(OWNER_ID, ASSET_ID, Asset::new(100));

        let mut delegations_data = DelegationsData::new();
        delegations_data.insert(
            DELEGATE_ID,
            vec![DelegateTransferArgs {
                from: ctx.caller,
                to: None,
                units: 43, // Different from the args!
            }],
        );

        let data = rmp_serialize(&delegations_data).unwrap();

        not_wasm::set_account_data(OWNER_ID, DELEGATIONS_KEY, &data);

        let args = TransferArgs {
            from: OWNER_ID,
            to: DESTINATION_ID,
            units: 42,
            data: None,
        };

        let err = not_wasm::call_wrap(transfer, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }
}
