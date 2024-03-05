use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct Deposit {
    pub depositor: Address,
    pub shares: u128,
    pub last_deposit: u64,
}

#[contracttype]
pub enum DepositsDataKeys {
    Deposit(Address),
}
