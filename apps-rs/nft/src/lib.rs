//! NFT
//!
//! ### Rules                                   
//!
//! 1. To ensure the authenticity of the NFT it can be initialized
//!    only by the creator that will be set automatically as `creator`
//! 2. Anyone can get the NFT information
//! 3. Only the Owner and the Intermediary can set the NFT sellable
//! 4. Only the Owner and the intermediary can set the price
//!    The new price cannot be below the minimum price setted by the creator
//! 5. Only the Creator can set the minimum price
//!    If the current price is lower than the new minimum price the price
//!    will be setted as the new minimum price
//! 6. A new intermediary can be setted only by the current Owner
//!

use trinci_sdk::{
    rmp_deserialize, rmp_serialize_named, AppContext, PackedValue, WasmError, WasmResult,
};

mod types;
use types::*;

trinci_sdk::app_export!(
    init,
    get_info,
    set_sellable,
    set_price,
    set_minimum_price,
    set_intermediary,
    buy
);

const CONFIG_KEY: &str = "config";
const INIT_KEY: &str = "init";

#[inline]
fn is_initialized() -> bool {
    !trinci_sdk::load_data(INIT_KEY).is_empty()
}

#[inline]
fn init_check() -> WasmResult<()> {
    match is_initialized() {
        true => Ok(()),
        false => Err(WasmError::new("not initialized")),
    }
}

/// Initialize contract status.
fn init(ctx: AppContext, args: NFTInitArgs) -> WasmResult<()> {
    if is_initialized() {
        return Ok(());
    }

    if ctx.caller != args.creator {
        return Err(WasmError::new("not authorized"));
    }

    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &args)?;

    trinci_sdk::store_account_data_mp!(INIT_KEY, &[1])?;

    Ok(())
}

/// Returns the NFT information
fn get_info(_ctx: AppContext, _args: PackedValue) -> WasmResult<PackedValue> {
    init_check()?;
    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let config: NFTConfig = rmp_deserialize(&buf)?;

    Ok(PackedValue(rmp_serialize_named(&config)?))
}

/// Set the sellable status of the contract.
fn set_sellable(ctx: AppContext, args: SetSellableArgs) -> WasmResult<()> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let mut config: NFTConfig = rmp_deserialize(&buf)?;

    if ctx.caller != config.owner && ctx.caller != config.intermediary {
        return Err(WasmError::new("not authorized"));
    }

    if config.sellable != args.sellable {
        config.sellable = args.sellable;
        trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;
    }

    Ok(())
}

/// Set the new NFT price
/// Can be called by the NFT Owner or the Intermediary
fn set_price(ctx: AppContext, args: SetPriceArgs) -> WasmResult<()> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let mut config: NFTConfig = rmp_deserialize(&buf)?;

    if ctx.caller != config.owner
        && (config.intermediary.is_empty() || ctx.caller != config.intermediary)
    {
        return Err(WasmError::new("not authorized"));
    }

    if args.price < config.minimum_price {
        return Err(WasmError::new(
            "the price cannot be below the minimim price",
        ));
    }

    if config.price != args.price {
        config.price = args.price;
        trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;
    }

    Ok(())
}

/// Set the new NFT minimum price
/// Can be called by the NFT creator
fn set_minimum_price(ctx: AppContext, args: SetPriceArgs) -> WasmResult<()> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let mut config: NFTConfig = rmp_deserialize(&buf)?;

    if ctx.caller != config.creator {
        return Err(WasmError::new("not authorized"));
    }

    if config.minimum_price != args.price {
        config.minimum_price = args.price;
        if config.price < config.minimum_price {
            config.price = config.minimum_price
        }
        trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;
    }

    Ok(())
}

/// Set the new NFT intermediary
/// Can be called by the NFT Owner
fn set_intermediary(ctx: AppContext, args: SetIntermediaryArgs) -> WasmResult<()> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let mut config: NFTConfig = rmp_deserialize(&buf)?;

    if ctx.caller != config.owner {
        return Err(WasmError::new("not authorized"));
    }

    let fee_margin = 0i64;

    if config.intermediary != args.intermediary
        || (config.intermediary_fee as i64 - args.intermediary_fee as i64).abs() > fee_margin
    {
        config.intermediary = args.intermediary;
        config.intermediary_fee = args.intermediary_fee;
        trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;
    }

    Ok(())
}

/// Allows a user to buy thr NFT
/// The new owner can set the sellable status and a new price
fn buy(ctx: AppContext, args: BuyArgs) -> WasmResult<()> {
    init_check()?;

    let buf = trinci_sdk::load_data(CONFIG_KEY);
    let mut config: NFTConfig = rmp_deserialize(&buf)?;

    if !config.sellable {
        return Err(WasmError::new("item not sellable"));
    }

    // Calculate fees
    let creator_part =
        math::round::half_up(config.price as f64 * config.creator_fee as f64 / 1000f64, 0) as u64;

    let mut intermediary_part: u64 = 0;
    if !config.intermediary.is_empty() {
        intermediary_part = math::round::half_up(
            config.price as f64 * config.intermediary_fee as f64 / 1000f64,
            0,
        ) as u64;
    }

    let owner_part = config.price - creator_part - intermediary_part;

    // Transfer from buyer to NFT account. This require a delegation from the buyer
    if trinci_sdk::asset_transfer(ctx.caller, ctx.owner, config.asset, config.price).is_err() {
        return Err(WasmError::new("error during withdraw from buyer"));
    }

    // Transfer from NFT to owner
    if trinci_sdk::asset_transfer(ctx.owner, config.owner, config.asset, owner_part).is_err() {
        return Err(WasmError::new("error during transfer to owner"));
    }

    // Transfer from NFT to creator
    if trinci_sdk::asset_transfer(ctx.owner, config.creator, config.asset, creator_part).is_err() {
        return Err(WasmError::new("error during transfer to creator"));
    }

    // Transfer from NFT to intermediary
    if intermediary_part != 0
        && trinci_sdk::asset_transfer(
            ctx.owner,
            config.intermediary,
            config.asset,
            intermediary_part,
        )
        .is_err()
    {
        return Err(WasmError::new("error during transfer to intermediary"));
    }

    // Set the new owner
    config.owner = ctx.caller;

    if args.new_price >= config.price {
        config.sellable = args.sellable;
        config.price = args.new_price;
    } else {
        config.sellable = false;
    }

    // Save the new config
    trinci_sdk::store_account_data_mp!(CONFIG_KEY, &config)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::types::tests::OWNER_1_ID;

    use super::*;
    use trinci_sdk::{
        not_wasm, rmp_serialize,
        tai::{self, Asset},
    };

    const NFT_ID: &str = types::tests::NFT_ID;
    const INTERMEDIARY_ID: &str = types::tests::INTERMEDIARY_ID;
    const CREATOR_ID: &str = types::tests::CREATOR_ID;
    const UNKNOWN_ID: &str = types::tests::UNKNOWN_ID;
    const BUYER_ID: &str = types::tests::UNKNOWN_ID;
    const ASSET_ID: &str = types::tests::ASSET_ID;

    const INIT_STATUS_HEX: &str = "9101";

    #[test]
    fn test_nft_initialization() {
        let ctx = not_wasm::create_app_context(NFT_ID, CREATOR_ID);

        let args = types::tests::create_nft_init_config_data();

        not_wasm::call_wrap(init, ctx, args.clone()).unwrap();

        let status = not_wasm::get_account_data(NFT_ID, INIT_KEY);
        assert_eq!(hex::encode(&status), INIT_STATUS_HEX);
        let config = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        assert_eq!(config, rmp_serialize(&args).unwrap());
    }

    #[test]
    fn test_nft_not_authorized_initialization() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);

        let args = types::tests::create_nft_init_config_data();

        let err = not_wasm::call_wrap(init, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
        let status = not_wasm::get_account_data(NFT_ID, INIT_KEY);
        assert_eq!(hex::encode(&status), "");
    }

    #[test]
    fn test_nft_not_initialized_get_info() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);

        let args = PackedValue::default();

        let err = not_wasm::call_wrap(get_info, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not initialized");
    }

    #[test]
    fn test_nft_get_info() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = PackedValue::default();

        let res = not_wasm::call_wrap(get_info, ctx, args).unwrap();
        let res: NFTConfig = rmp_deserialize(&res.0).unwrap();

        assert_eq!(res, config);
    }

    #[test]
    fn test_nft_not_initialized_set_sellable() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);

        let args = SetSellableArgs { sellable: false };

        let err = not_wasm::call_wrap(set_sellable, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not initialized");
        let status = not_wasm::get_account_data(NFT_ID, INIT_KEY);
        assert_eq!(hex::encode(&status), "");
    }

    #[test]
    fn test_nft_not_authorized_set_sellable() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetSellableArgs { sellable: true };

        let err = not_wasm::call_wrap(set_sellable, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_nft_set_sellable_by_owner() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetSellableArgs { sellable: true };

        not_wasm::call_wrap(set_sellable, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        let config: NFTConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(args.sellable, config.sellable);
    }

    #[test]
    fn test_nft_set_sellable_by_intermediary() {
        let ctx = not_wasm::create_app_context(NFT_ID, INTERMEDIARY_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let mut config = types::tests::create_nft_init_config_data();

        config.intermediary = INTERMEDIARY_ID;
        config.intermediary_fee = 4;

        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetSellableArgs { sellable: false };

        not_wasm::call_wrap(set_sellable, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        let config: NFTConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(args.sellable, config.sellable);
    }

    #[test]
    fn test_nft_not_initialized_set_price() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);

        let args = SetPriceArgs { price: 42 };

        let err = not_wasm::call_wrap(set_price, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "not initialized");
        let status = not_wasm::get_account_data(NFT_ID, INIT_KEY);
        assert_eq!(hex::encode(&status), "");
    }

    #[test]
    fn test_nft_not_authorized_set_price() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 142 };

        let err = not_wasm::call_wrap(set_price, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_nft_set_price_too_low() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 42 };

        let err = not_wasm::call_wrap(set_price, ctx, args.clone()).unwrap_err();

        assert_eq!(
            err.to_string(),
            "the price cannot be below the minimim price"
        );
    }

    #[test]
    fn test_nft_set_price_by_owner() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 52000 };

        not_wasm::call_wrap(set_price, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        let config: NFTConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(args.price, config.price);
    }

    #[test]
    fn test_nft_set_price_by_intermediary() {
        let ctx = not_wasm::create_app_context(NFT_ID, INTERMEDIARY_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let mut config = types::tests::create_nft_init_config_data();

        config.intermediary = INTERMEDIARY_ID;
        config.intermediary_fee = 12; // 1.2%

        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 52_000 };

        not_wasm::call_wrap(set_price, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        let config: NFTConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(args.price, config.price);
    }

    #[test]
    fn test_nft_set_price_by_empty_intermediary() {
        let ctx = not_wasm::create_app_context(NFT_ID, "");
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();

        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 142 };

        let err = not_wasm::call_wrap(set_price, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_nft_set_minimum_price_not_initialized() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);

        let args = SetPriceArgs { price: 42 };

        let err = not_wasm::call_wrap(set_minimum_price, ctx, args.clone()).unwrap_err();

        assert_eq!(err.to_string(), "not initialized");
        let status = not_wasm::get_account_data(NFT_ID, INIT_KEY);
        assert_eq!(hex::encode(&status), "");
    }

    #[test]
    fn test_nft_set_minimum_price_not_authorized() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 142 };

        let err = not_wasm::call_wrap(set_minimum_price, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_nft_set_minimum_price_by_owner() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let mut config = types::tests::create_nft_init_config_data();

        config.intermediary = INTERMEDIARY_ID;
        config.intermediary_fee = 4;

        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 142 };

        let err = not_wasm::call_wrap(set_minimum_price, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_nft_set_minimum_price_by_intermediary() {
        let ctx = not_wasm::create_app_context(NFT_ID, INTERMEDIARY_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let mut config = types::tests::create_nft_init_config_data();

        config.intermediary = INTERMEDIARY_ID;
        config.intermediary_fee = 4;

        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 52_000 };

        let err = not_wasm::call_wrap(set_minimum_price, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_nft_set_minimum_price_by_creator() {
        let ctx = not_wasm::create_app_context(NFT_ID, CREATOR_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetPriceArgs { price: 52000 };

        not_wasm::call_wrap(set_minimum_price, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        let config: NFTConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(args.price, config.minimum_price);
        assert_eq!(config.price, config.minimum_price);
    }

    #[test]
    fn test_nft_set_intermediary_not_initialized() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);

        let args = SetIntermediaryArgs {
            intermediary: INTERMEDIARY_ID,
            intermediary_fee: 32, // 3.2%
        };

        let err = not_wasm::call_wrap(set_intermediary, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not initialized");
        let status = not_wasm::get_account_data(NFT_ID, INIT_KEY);
        assert_eq!(hex::encode(&status), "");
    }

    #[test]
    fn test_nft_not_authorized_set_intermediary() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetIntermediaryArgs {
            intermediary: INTERMEDIARY_ID,
            intermediary_fee: 32,
        };
        let err = not_wasm::call_wrap(set_intermediary, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not authorized");
    }

    #[test]
    fn test_nft_set_intermediary() {
        let ctx = not_wasm::create_app_context(NFT_ID, OWNER_1_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = SetIntermediaryArgs {
            intermediary: INTERMEDIARY_ID,
            intermediary_fee: 32,
        };
        not_wasm::call_wrap(set_intermediary, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        let config: NFTConfig = rmp_deserialize(&buf).unwrap();

        assert_eq!(args.intermediary, config.intermediary);
        assert_eq!(config.intermediary_fee, config.intermediary_fee);
    }

    #[test]
    fn test_nft_buy_not_initialized() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);

        let args = BuyArgs {
            sellable: false,
            new_price: 150_000,
        };

        let err = not_wasm::call_wrap(buy, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "not initialized");
        let status = not_wasm::get_account_data(NFT_ID, INIT_KEY);
        assert_eq!(hex::encode(&status), "");
    }

    #[test]
    fn test_nft_buy_nft_not_sellable() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let config = types::tests::create_nft_init_config_data();
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        let args = BuyArgs {
            sellable: false,
            new_price: 150_000,
        };

        let err = not_wasm::call_wrap(buy, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "item not sellable");
    }

    #[test]
    fn test_nft_buy_nft_insufficient_funds() {
        let ctx = not_wasm::create_app_context(NFT_ID, UNKNOWN_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let mut config = types::tests::create_nft_init_config_data();
        config.sellable = true;
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        not_wasm::set_account_asset_gen(BUYER_ID, ASSET_ID, tai::Asset::new(49_999));

        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);

        let args = BuyArgs {
            sellable: false,
            new_price: 150_000,
        };

        let err = not_wasm::call_wrap(buy, ctx, args).unwrap_err();

        assert_eq!(err.to_string(), "error during withdraw from buyer");
    }

    #[test]
    fn test_nft_buy_nft() {
        let ctx = not_wasm::create_app_context(NFT_ID, BUYER_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let mut config = types::tests::create_nft_init_config_data();
        config.sellable = true;
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        not_wasm::set_account_asset_gen(BUYER_ID, ASSET_ID, tai::Asset::new(100_000));

        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);

        let args = BuyArgs {
            sellable: false,
            new_price: 55_000,
        };

        not_wasm::call_wrap(buy, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        let config: NFTConfig = rmp_deserialize(&buf).unwrap();
        assert_eq!(config.owner, BUYER_ID);
        assert_eq!(config.sellable, args.sellable);
        assert_eq!(config.price, args.new_price);
    }

    #[test]
    fn test_nft_buy_nft_with_intermediary() {
        let ctx = not_wasm::create_app_context(NFT_ID, BUYER_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let mut config = types::tests::create_nft_init_config_data();
        config.sellable = true;
        config.intermediary = INTERMEDIARY_ID;
        config.intermediary_fee = 33; // 3.3%

        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        not_wasm::set_account_asset_gen(BUYER_ID, ASSET_ID, tai::Asset::new(100_000));

        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);

        let args = BuyArgs {
            sellable: false,
            new_price: 55000,
        };

        not_wasm::call_wrap(buy, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);
        let config: NFTConfig = rmp_deserialize(&buf).unwrap();
        assert_eq!(config.owner, BUYER_ID);
        assert_eq!(config.sellable, args.sellable);
        assert_eq!(config.price, args.new_price);

        let buyer_asset: Asset =
            rmp_deserialize(&not_wasm::get_account_asset(BUYER_ID, ASSET_ID)).unwrap();
        assert_eq!(buyer_asset.units, 50_000);
        let creator_asset: Asset =
            rmp_deserialize(&not_wasm::get_account_asset(CREATOR_ID, ASSET_ID)).unwrap();
        assert_eq!(creator_asset.units, 1750);
        let owner_asset: Asset =
            rmp_deserialize(&not_wasm::get_account_asset(OWNER_1_ID, ASSET_ID)).unwrap();
        assert_eq!(owner_asset.units, 46_600);
        let intermediary_asset: Asset =
            rmp_deserialize(&not_wasm::get_account_asset(INTERMEDIARY_ID, ASSET_ID)).unwrap();
        assert_eq!(intermediary_asset.units, 1650);
    }

    #[test]
    fn test_nft_buy_nft_with_wrong_new_price() {
        let ctx = not_wasm::create_app_context(NFT_ID, BUYER_ID);
        let data = hex::decode(INIT_STATUS_HEX).unwrap();
        not_wasm::set_account_data(NFT_ID, INIT_KEY, &data);

        let mut config = types::tests::create_nft_init_config_data();
        config.sellable = true;
        let buf = rmp_serialize(&config).unwrap();
        not_wasm::set_account_data(NFT_ID, CONFIG_KEY, &buf);

        not_wasm::set_account_asset_gen(BUYER_ID, ASSET_ID, tai::Asset::new(100_000));

        not_wasm::set_contract_method(ASSET_ID, "transfer", not_wasm::asset_transfer);
        not_wasm::set_contract_method(ASSET_ID, "balance", not_wasm::asset_balance);

        let args = BuyArgs {
            sellable: true,
            new_price: 10,
        };

        not_wasm::call_wrap(buy, ctx, args.clone()).unwrap();

        let buf = not_wasm::get_account_data(NFT_ID, CONFIG_KEY);

        let config: NFTConfig = rmp_deserialize(&buf).unwrap();
        assert_eq!(config.owner, BUYER_ID);
        assert_eq!(config.sellable, false);
        assert!(config.price >= args.new_price);
    }
}
