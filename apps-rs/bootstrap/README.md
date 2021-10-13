# Bootstrap Contract

 - Smart contract used only once to register the Service Account.
 - The transaction that use this contract will be registered in the Genesis block, alone.

 - This contract is not meant to be stored within the service account itself, instead is supposed
   to be used once by the wasm machine bootstrap loader the first time the blockchain is started.

