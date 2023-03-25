// TODO: Handle decimals
#![no_std]

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

mod contract;
mod storage_types;
mod utils;

mod test;
mod tests;

pub use crate::contract::VaultsContractClient;
