//! The [address lookup table program][np].
//!
//! [np]: https://docs.solana.com/developing/runtime-facilities/programs#address-lookup-table-program

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("AddressLookupTab1e1111111111111111111111111");
