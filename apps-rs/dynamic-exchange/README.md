Dynamic Exchange
===

## Features
 - Allows an account to exchange asset type with other accounts

## Methods

### `init`

 - Allows to initialize the Dynamic Exchange
 - Can be sent only by the account owner.
 - Account-ids and the assets list cannot be empty
 - Locks the asset for sale and the penalty asset for the withdrawal operation

```json
args: {
	"seller": account-id,		   	// who sells the asset
	"asset": account-id,		    // source asset
	"guarantor": account-id,	   	// exchange owner
	"guarantor_fee": integer,	   	// fee percentage for the guarantor [1]
	"penalty_fee": integer,		   	// fee percentage for the exchange abort [2]
	"penalty_asset": account-id,	// fee asset for the exchange abort 
	"assets": {					    // map of accepted assets and exchange rate [3]
		"asset1_id": exchange_rateA,	
		...							// asset-id: account-id
		...							// exchange_rate: integer
		"assetX_id": exchange_rateX,
}
```
 - The asset amount is in the relative asset field of the account (needs to be transfer by the seller after the init)
 - If this transaction succeed the Dynamic Exchange status is set to `open`
 - [1] This fee is expressed in thousandths: e.g. 5 => 5/1000 * 100 = 0.5%
 - [2] This fee is expressed in thousandths if the penalty asset is the same of the asset for sale, it is asset units if the penalty asset is not the same.
 - [3] The exchange rates are expressed in hundredths: 
   - asset_X = RATE / 100 * asset_for_sale


### `abort`

 - Allows to prematurely close the Dynamic Exchange
 - Can be called by the `seller` or the `guarantor`
 - Transfers the remain asset amount to the Seller except for guarantor's part (penalty)

```json
args: {} 
```

 - If this transaction succeed the Dynamic Exchange status is set to `aborted`


### `apply`

 - Allows a `buyer` to buy the source asset with another on the assets list
 - The source asset amount would be equal or greater than the asset that the buyer wants to buy multiplied for the exchange rate
 - The buyer must own the amount of destination asset for which it applies 

```json
args: {
	"asset": account-id, 	// asset that the buyer wants to exchange
	"amount": integer,		// quantity of the asset that the buyer wants to exchange
}

```
 - This transaction triggers:
   - A transfer of the destination asset from the Buyer to the Dynamic Exchange
   - A transfer of the destination asset from the Dynamic Exchange to the Seller 
   - A transfer of the destination asset from the Dynamic Exchange to the Guarantor
   - A transfer of the source asset from the Dynamic Exchange to the Buyer 
   - If the source balance is zero after this transfers the Dynamic Exchange status is set to `exhausted`

### `get_info`
 - Allows to retrieve information about the Dynamic Exchange

 ```json
 args: {}
 ```

 Returns:
 ```json
 {
     "config": {
 	    "seller": account-id,		    // who sells the asset
		"asset": account-id,		    // source asset
        "guarantor": account-id,	    // exchange owner
     	"guarantor_fee": integer,		// fee percentage for the guarantor of apply asset
 	    "penalty_fee": integer,	   	    // fee percentage for the exchange abort 
 	    "penalty_asset": account-id, 	// fee asset for the exchange abort 
 	    "assets": {					    // map of accepted assets and exchange rate
 		    asset1-id: exchange_rate1,
 		    ...
 		    assetX-id: exchange_rateX,
     },
     "amount": integer,
     "status": {"open" | "exhausted" | "aborted"},
 }
 ```
