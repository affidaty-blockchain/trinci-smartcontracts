# Withdraw
 - Use by the exchange to convert asset in currency

### Methods
 - [x] `init` - Initializes the contract
  - can be performed by the account owner or the `config.exchange` account
  ```json
  args: {
      "customer": account-id,    // who wants to exchange asset in currency money 
      "exchange": account-id,    // who pays currency money for the `withdrawn asset`
      "currency_asset": {
          "id": account-id,  // the `currency_asset` (represents the currency money)
          "units": integer
      },
      "withdrawn_asset": {          // the `withdrawn asset`
          "id": account-id,       
          "units": integer,         
      },
  }

- [x] `get_info`
  - can be performed by the `customer` or the `config.exchange` account
  ```json
  args: {}
  ```

  Returns: 
  ```json
  {
      "customer": account-id,    // who wants to exchange asset in currency money 
      "exchange": account-id,    // who pays currency money for the `withdrawn asset`
      "currency_asset": {
          "id": account-id,  // the `currency_asset` (represents the currency money)
          "units": integer
      },
      "withdrawn_asset": {          // the `withdrawn asset`
          "id": account-id,       
          "units": integer,         
      },
      "currency_asset_amount": integer,
      "withdrawn_asset_amount": integer,
  }
  ```
- [x] `update`
  - resolve the contract
  - can be performed by the `config.exchange` account
  ```json
  args: {
      "status": string,    // "ok", "ko"
  }

  ```
  - "ok"
    - Burns the balance of the `withdrawn asset`
    - Burns the balance of the `currency asset`
  - "ko"
    - Transfers the balance of `withdrawn asset` to the `Customer`
    - Transfers the balance of `currency asset` to the `Exchange`

**Note:** - The `exchange` account must be authorized to te asset burn