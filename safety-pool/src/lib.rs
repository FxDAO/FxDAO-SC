#![no_std]

mod vaults {
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/vaults.wasm");
}

mod oracle {
    soroban_sdk::contractimport!(file = "../currencies_oracle.wasm");
}

mod contract;
mod errors;
mod storage;
mod tests;
mod utils;
