//! Template for Trinci Smart Contract Development
//!
//! ### Methods
//!
//!  - `init` - can be called only from the account owner
//!  - `apply` - can be called only by a customer
//!  - `get_info` - can be called only by a customer or the restaurateur
//!  - `close` - can be called only by the restaurateur,
//!       transfer the contract asset to the restaurateur
//!       close the contract if all the customers have been paid

use trinci_sdk::{
    rmp_deserialize, rmp_serialize_named, AppContext, PackedValue, WasmError, WasmResult,
};

mod types;
use types::*;

trinci_sdk::app_export!(init, apply, get_info, close);

/// Init method
fn init(ctx: AppContext, args: InitArgs) -> WasmResult<()> {
    if ctx.owner != ctx.caller {
        return Err(WasmError::new("not authorized"));
    }

    // Prevent to withdraw the asset
    trinci_sdk::asset_lock(args.asset, ctx.owner, trinci_sdk::tai::LockType::Withdraw)?;

    trinci_sdk::store_account_data_mp!("config", &args)
}

/// Get_Info method
fn get_info(ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    // Load the contract configuration
    let buf = trinci_sdk::load_data("config");
    let config: InitArgs = match rmp_deserialize(&buf) {
        Ok(val) => val,
        Err(_) => return Err(WasmError::new("not initialized")),
    };

    // Checks on the authorization
    let mut auth_list = vec![config.restaurateur];
    config
        .customers
        .iter()
        .for_each(|(&customer, _)| auth_list.push(customer));

    match auth_list.iter().find(|&&elem| elem == ctx.caller) {
        Some(_) => {
            let buf = rmp_serialize_named(&config)?;
            Ok(PackedValue(buf))
        }
        None => Err(WasmError::new("not authorized")),
    }
}

/// Apply method
fn apply(ctx: AppContext, _args: PackedValue) -> WasmResult<()> {
    // Load the contract configuration
    let buf = trinci_sdk::load_data("config");
    let mut config: InitArgs = match rmp_deserialize(&buf) {
        Ok(val) => val,
        Err(_) => return Err(WasmError::new("not initialized")),
    };

    if config.status != "open" {
        return Err(WasmError::new("contract closed"));
    }

    // Check if the caller is in the list and have already paid
    if let Some(val) = config.customers.get_mut(ctx.caller) {
        if !*val {
            // Make the payment
            trinci_sdk::asset_transfer(ctx.caller, ctx.owner, config.asset, config.part).map_err(
                |err| {
                    println!("err: {}", err.to_string());
                    WasmError::new("transfer from caller failed")
                },
            )?;
            *val = true;
        } else {
            return Err(WasmError::new("already paid"));
        }
    } else {
        return Err(WasmError::new("not authorized"));
    };

    // Store the config and exit
    trinci_sdk::store_account_data_mp!("config", &config)?;

    Ok(())
}

/// Close method
fn close(ctx: AppContext, _args: PackedValue) -> WasmResult<()> {
    // Load the contract configuration
    let buf = trinci_sdk::load_data("config");
    let mut config: InitArgs = match rmp_deserialize(&buf) {
        Ok(val) => val,
        Err(_) => return Err(WasmError::new("not initialized")),
    };

    // Check if the caller is the restaurateur
    if ctx.caller != config.restaurateur {
        return Err(WasmError::new("not authorized"));
    }

    // Check if the contract is still opened
    if config.status != "open" {
        return Err(WasmError::new("contract closed"));
    }

    // Unlock the asset
    trinci_sdk::asset_lock(config.asset, ctx.owner, trinci_sdk::tai::LockType::None)?;

    // Get Balance
    let amount: u64 = trinci_sdk::asset_balance(config.asset)?;

    if amount > 0 {
        // Perform the Transfer
        trinci_sdk::asset_transfer(ctx.owner, config.restaurateur, config.asset, amount)?;
    }

    // Lock again the asset
    trinci_sdk::asset_lock(config.asset, ctx.owner, trinci_sdk::tai::LockType::Withdraw)?;

    // If all the customers have paid set the status to close
    if config.customers.values().all(|&val| val) {
        //  All the customers have paid
        config.status = "close";
        trinci_sdk::store_account_data_mp!("config", &config)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::types::tests::{
        create_init_args, ASSET_ID, CUSTOMER1_ID, CUSTOMER2_ID, CUSTOMER3_ID, PAY_ID,
        RESTAURATEUR_ID,
    };
    use trinci_sdk::{not_wasm, rmp_deserialize, rmp_serialize, tai::Asset, value, Value};

    #[test]
    fn test_init() {
        let ctx = not_wasm::create_app_context(PAY_ID, PAY_ID);
        let args = create_init_args();

        // Associate a mock asset_lock to the asset account
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);

        not_wasm::call_wrap(init, ctx, args.clone()).unwrap();

        // Checks on the account
        let buf = not_wasm::get_account_data(PAY_ID, "config");
        let data: InitArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data, args);
    }

    #[test]
    fn test_init_not_authorized() {
        let ctx = not_wasm::create_app_context(PAY_ID, "Unknown");
        let args = create_init_args();

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);

        let err = not_wasm::call_wrap(init, ctx, args.clone()).unwrap_err();

        // Checks on the error
        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_get_info() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, CUSTOMER1_ID);

        // Prepare the account data/config
        let data = create_init_args();
        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        let args = PackedValue::default();

        let res = not_wasm::call_wrap(get_info, ctx, args).unwrap();

        let data: Value = rmp_deserialize(&res.0).unwrap();

        // Checks on the contract info
        let restaurateur = data.get(&value!("restaurateur")).unwrap().as_str().unwrap();

        assert_eq!(restaurateur, RESTAURATEUR_ID);
    }

    #[test]
    fn test_get_info_not_initialized() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, RESTAURATEUR_ID);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(get_info, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not initialized");
    }

    #[test]
    fn test_get_info_not_authorized() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, "unknown");

        // Prepare the account data/config
        let data = create_init_args();
        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(get_info, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_apply() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, CUSTOMER1_ID);

        // Prepare the account data/config
        let data = create_init_args();
        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        // Give the customer some asset
        not_wasm::set_account_asset_gen(CUSTOMER1_ID, ASSET_ID, Asset::new(100));

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);

        let args = PackedValue::default();

        not_wasm::call_wrap(apply, ctx, args).unwrap();

        // Checks on the contract account config
        let buf = not_wasm::get_account_data(PAY_ID, "config");
        let data: InitArgs = rmp_deserialize(&buf).unwrap();
        let customer1 = data.customers.get(CUSTOMER1_ID).unwrap();

        assert!(customer1);

        // Checks on the contract asset
        let asset: Asset = not_wasm::get_account_asset_gen(PAY_ID, ASSET_ID);

        assert_eq!(asset.units, 30);
    }

    #[test]
    fn test_apply_on_already_paid() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, CUSTOMER1_ID);

        // Prepare the account data/config
        let mut data = create_init_args();
        let customer1 = data.customers.get_mut(&CUSTOMER1_ID).unwrap();
        *customer1 = true;

        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        // Give the customer some asset
        not_wasm::set_account_asset_gen(CUSTOMER1_ID, ASSET_ID, Asset::new(100));

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(apply, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "already paid");
    }

    #[test]
    fn test_apply_on_closed_contract() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, CUSTOMER1_ID);

        // Prepare the account data/config
        let mut data = create_init_args();
        data.status = "close";
        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        // Give the customer some asset
        not_wasm::set_account_asset_gen(CUSTOMER1_ID, ASSET_ID, Asset::new(100));

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(apply, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "contract closed");
    }

    #[test]
    fn test_apply_not_initialized() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, CUSTOMER1_ID);

        // Give the customer some asset
        not_wasm::set_account_asset_gen(CUSTOMER1_ID, ASSET_ID, Asset::new(100));

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(apply, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not initialized");
    }

    #[test]
    fn test_apply_not_authorized() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, "unknown");

        // Prepare the account data/config
        let data = create_init_args();
        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        // Give the customer some asset
        not_wasm::set_account_asset_gen(CUSTOMER1_ID, ASSET_ID, Asset::new(100));

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(apply, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_close() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, RESTAURATEUR_ID);

        // Prepare the account data/config
        let mut data = create_init_args();
        *data.customers.get_mut(CUSTOMER1_ID).unwrap() = true;
        *data.customers.get_mut(CUSTOMER2_ID).unwrap() = true;
        *data.customers.get_mut(CUSTOMER3_ID).unwrap() = true;

        let amount = data.part * 3;
        // Put the asset on the contract account
        not_wasm::set_account_asset_gen(PAY_ID, ASSET_ID, Asset::new(amount));

        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);

        // Associate a mock asset_lock to the asset account
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);

        // Associate a mock balance to the asset account
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);

        let args = PackedValue::default();

        not_wasm::call_wrap(close, ctx, args).unwrap();

        // Checks on the contract account config
        let buf = not_wasm::get_account_data(PAY_ID, "config");
        let data: InitArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data.status, "close");

        // Checks on the contract asset
        let asset: Asset = not_wasm::get_account_asset_gen(PAY_ID, ASSET_ID);

        assert_eq!(asset.units, 0);

        // Checks on the restaurateur asset
        let asset: Asset = not_wasm::get_account_asset_gen(RESTAURATEUR_ID, ASSET_ID);

        assert_eq!(asset.units, amount);
    }

    #[test]
    fn test_close_with_unpaid_bill() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, RESTAURATEUR_ID);

        // Prepare the account data/config
        let mut data = create_init_args();
        *data.customers.get_mut(CUSTOMER1_ID).unwrap() = true;
        *data.customers.get_mut(CUSTOMER3_ID).unwrap() = true;

        let amount = data.part * 2;
        // Put the asset on the contract account
        not_wasm::set_account_asset_gen(PAY_ID, ASSET_ID, Asset::new(amount));

        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);

        // Associate a mock asset_lock to the asset account
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);

        // Associate a mock balance to the asset account
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);

        let args = PackedValue::default();

        not_wasm::call_wrap(close, ctx, args).unwrap();

        // Checks on the contract account config
        let buf = not_wasm::get_account_data(PAY_ID, "config");
        let data: InitArgs = rmp_deserialize(&buf).unwrap();

        assert_eq!(data.status, "open");

        // Checks on the contract asset
        let asset: Asset = not_wasm::get_account_asset_gen(PAY_ID, ASSET_ID);

        assert_eq!(asset.units, 0);

        // Checks on the restaurateur asset
        let asset: Asset = not_wasm::get_account_asset_gen(RESTAURATEUR_ID, ASSET_ID);

        assert_eq!(asset.units, amount);
    }

    #[test]
    fn test_close_not_authorized() {
        // Prepare the environment
        // Prepare the context
        let ctx = not_wasm::create_app_context(PAY_ID, CUSTOMER1_ID);

        // Prepare the account data/config
        let mut data = create_init_args();
        *data.customers.get_mut(CUSTOMER1_ID).unwrap() = true;
        *data.customers.get_mut(CUSTOMER3_ID).unwrap() = true;

        let amount = data.part * 2;
        // Put the asset on the contract account
        not_wasm::set_account_asset_gen(PAY_ID, ASSET_ID, Asset::new(amount));

        let data = rmp_serialize(&data).unwrap();
        not_wasm::set_account_data(PAY_ID, "config", &data);

        // Associate a mock transfer to the asset account
        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);

        // Associate a mock asset_lock to the asset account
        not_wasm::set_contract_method(ASSET_ID, "lock", not_wasm::asset_lock);

        // Associate a mock balance to the asset account
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(close, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }
}
