#![no_std]

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

mod contract;
mod errors;
mod storage;
mod tests;
mod utils;

pub use crate::contract::SafetyPoolContractClient;
