//! Escrow
//!
//! ### Rules
//!
//! 1. Initialization is performed by the escrow account owner.
//! 2. Initialization checks if the account owns at least the quantity required by the escrow init arguments.
//! 3. Once initialized the asset under escrow is locked by the contract and
//!    operations on it can be performed only by passing through the contract
//!    methods.
//! 4. Balance returns the total amount under escrow (not the total amount on
//!    the account, that is temporary hidden).
//! 5. Balance can be invoked only by customer, merchants or guarantor.
//! 6. Update method, to resolve the escrow, can be invoked only by the guarantor.
//!
//! ### Warning
//!
//! To work correctly the contract shall be used with an asset respecting the
//! **TAI** interface.  In particular it is assumed that the asset `lock` type
//! `Contract` is implemented to not be bypassed via direct asset contract call.
//!
