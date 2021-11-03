Service
===

## Features
 - Allows the registration of new assets, contracts, oracles and aliases
 - Allows the retrieval of information about assets, contracts, oracles and aliases registered on the blockchain

## Methods

### `contract_registration`

- Allows anyone to register a smart contract on the blockchain

  ```json
  args: {
     "name": string,         // contract name
     "description": string,  // brief description
     "url": string,          // web page
     "bin": binary,         // contract binary
  }
  ```

  The contract information will be saved with the app-hash key in the service account `contracts` field

  ```json
  "contracts": {
     ...
     app-hash: {              // data binary hash
        "name": string,         // contract name
        "creator": account-id,  // caller account id
        "description": string,  // brief description
        "url": string,          // web page
     },
     ...
  }
  ```

  The contract binary data will be stored as:

  ```json
  {
   ...
   "app_hash_1": binary,
   ...
   "app_hash_N": binary
   ...
  }
  ```

### `get_contract_information`

- Allows to retrieve information about a contract

  ```json
  args: {
     "contract": string,         // contract hash
  }
  ```

  This method will return the contract data, e.g.:

  ```json
  {
    "name": "MyContract",
    "creator": "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i",
    "description": "Contract for my purpose",
    "url": "mycontract.org"
  }
  ```

### `asset_registration`

- Allows anyone to register a new asset on the blockchain

  ```json
  args: {
     "id": account-id,   // asset account id
     "name": string,     // friendly name
     "url": string,      // webpage
     "contract": binary, // related contract hash
  }
  ```

  The Asset data will be stored with the asset-account-id key in the service account `assets` field

  ```json
  "assets": {
     ...
     asset-account-id: {
        "name": string,         // friendly name
        "creator": account-id,  // asset creator account id
        "url": string,          // webpage
        "contract": binary,     // related contract hash
     },
     ...
  }
  ```

### `get_asset_information`

- Allows to retrieve information about an asset

  ```json
  args: {
     "asset_id": account-id,         // asset account id
  }
  ```

  This method will return the asset data, e.g.:

  ```json
  {
      "name": "MyCoolCoin",
      "creator": "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i",
      "url": "mycoolcoin.org",
      "contract": [122,12,33,254,...],
  }
  ```

### `oracle_registration`

- Allows anyone to register a new oracle on the blockchain

  ```json
  args: {
     "id": account-id,       // oracle account id
     "name": string,         // friendly name
     "description": string,  // brief description
     "url": string,          // webpage
     "contract": binary,     // binary oracle contract hash
  }
  ```

  The Oracle data will be stored with the oracle-account-id key in the service account `oracles` field

  ```json
  "oracles": {
     ...
     oracle-account-id:
     {
        "name": string,
        "creator": account-id,  // caller account id
        "description": string,
        "url": string,
        "contract": binary
     },
     ...
  }
  ```

### `get_oracle_information`

- Allows to retrieve information about an oracle

  ```json
  args: {
     "oracle_id": account-id,         // oracle account id
  }
  ```

  This method will return the oracle data, e.g.:

  ```json
  {
   "name": "MyCoolOracle",
   "creator": "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i",
   "description": "The purpose of this Oracle is to no nothing",
   "url": "mycooloracle.org",
   "contract": [122,12,33,254,...],
  }
  ```

### `alias_registration`

- Allows to register a new alias for an account on the blockchain

  ```json
  args: {
     "alias": string,     // the alias for the caller account id
  }
  ```

  The alias will be stored as map key/value with `alias` as key and `caller-account-id` as value in the service account `aliases` field

  ```json
  "aliases": {
     ...
     alias: caller-account-id,
     ...
  }
  ```

### `alias_lookup`

- Allows anyone to retrieve an account id starting from an alias

  ```json
  args: {
     "alias": string,     // the alias for the wanted account id
  }
  ```

  This method will return the alias account id, e.g.:

  ```json
  {
     "QmYHnEQLdf5h7KYbjFPuHSRk2SPgdXrJWFh5W696HPfq7i"
  }
  ```

### `alias_deletion`

- Allows the owner account to delete an alias

  ```json
  args: {
     "alias": string,     // the alias for the wanted account id
  }
  ```
