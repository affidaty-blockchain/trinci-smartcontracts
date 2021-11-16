Basic Asset
===

## Features

-   Performs the transfer of a specific asset

## Methods

### `init`

-   Initializes the asset account

    ```json
    args: {
        "name": string,             // Friendly asset name (in the future could be the asset alias)
        "authorized": [account-id], // List of authorized account to perform mint and burn methods
        "description": string,      // Brief description of the asset
        "url": string,              // Asset website
        "max_units": integer,       // Maximum mintable quantity
    }
    ```
    These values will be stored with key `config` in the account:

    ```json
    "config": {
        "name": string,
        "creator": account-id,      // Caller account id
        "authorized": [account-id], // List of authorized account to perform mint and burn methods
        "description": string,
        "url": string,
        "max_units": integer,
        "minted": integer,      // Current quantity minted of the asset
        "burned": integer,      // Current quantity burned of the asset
    }
    ```

### `transfer`

-   Performs the transfer of a specific asset by custom rules

    ```json
    args: {
        "from": account-id,   // Source Account id
        "to": account-id,     // Destination Account id
        "units": integer,     // Amount in asset units
    }
    ```

    `transfer` usage example:

    ```json
    args: {
        "to": "QmSCRCPFznxEX6S316M4yVmxdxPB6XN63ob2LjFYkP6MLq",
        "asset": "QmQH99ydqr7Ci1Hsj5Eb5DnbR1hDZRFpRrQVeQZHkibuEp",
        "amount": 42
    }
    ```

### `mint`

- Creates some asset units and transfer them to the specified account
- Can be performed only by the asset `creator` or by an `authorized` account.
- If the call is from a contract the `origin` signer must be the `creator`
  or an `authorized` account.

    ```json
    args: {
        "to": account-id,   // Destination Account ID
        "units": integer,   // Amount of asset to mint
    }
    ```

### `burn`

- Destroys some asset from the specified account
- Can be performed only by the asset `creator` or by an `authorized` account.
- If the call is from a contract the `origin` signer must be the `creator`
  or an `authorized` account.

    ```json
    args: {
        "from": account-id,   // Source Account ID
        "units": integer,     // Amount of asset to mint
    }
    ```

### `balance`

-   Returns the asset balance of the caller account

    ```json
    args: {}
    ```

    Return example:

    ```json
    {
        42
    }
    ```

### `stats`

-   Returns the asset configuration

    ```json
    args: {}
    ```

    Return example:

    ```json
    {
        "name": "MyCoolAsset",
        "creator": "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i",
        "authorized": ["QmX...123", "QmY...456"], 
        "description": "This is my cool asset",
        "url": "coolasset.org",
        "max_units": 100_000,
        "minted": 25023,
        "burned": 1235
    }
    ```

### `lock`

-   Locks the asset.

    -   A locked asset cannot be moved from or into the accout.
    -   The lock type is inferred from the caller.
    -   The owner can't unlock an asset locked by a smart contract or by the asset creator.

    ```json
    args: {
        "to": account-id,
        "lock": LockType,
    }
    ```

    **Note:** The lock level is implicitly derived from the caller account

