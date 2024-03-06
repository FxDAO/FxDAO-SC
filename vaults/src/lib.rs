#![no_std]

mod oracle {
    soroban_sdk::contractimport!(file = "../currencies_oracle.wasm");
}

mod contract;
mod storage;
mod utils;

mod errors;
mod tests;

pub use crate::contract::VaultsContractClient;
