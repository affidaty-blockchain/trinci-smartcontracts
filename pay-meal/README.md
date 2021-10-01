# Pay Meal
 - Tutorial contract (to use only for study purpose)
 - Contract that allows to split the meal bill between all the diners

### Methods
 - `init` - Initialize the contract
 ```json
 args:{
    "restaurateur": account-id,   // is the merchant account
    "asset": account-id,          // is the asset account
    "part": integer,              // is the the share for each diner
    "customers": {
       customer_1: false,            // this will become `true` when the diner_1 will pay
       ...
       customer_N: false,
    },
    "status": string              // status of the contract
}
```

 - `get_info` - retrieves the contract information
 ```json
 args: {}
 ```
 Returns:
 ```json
 {
    "restaurateur": account-id,   // is the merchant account
    "asset": account-id,          // is the asset account
    "part": integer,              // is the the share for each diner
    "customers": {
       customer_1: false,            // this will become `true` when the diner_1 will pay
       ...
       customer_N: false,
    },
    "status": string              // status of the contract
}
```
 
- `apply` - allow a customer to pay his share
```json
args: {}
```

- `close` - allows the restaurateur to collect the shares that the customers have already paid
```json
args: {}
```
 - if all the customers have already paid put the contract status on "close"
