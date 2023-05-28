use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct Deposit {
    pub depositor: Address,
    pub amount: u128,
    pub deposit_time: u64,
}

#[contracttype]
pub enum DepositsDataKeys {
    Deposit(Address),
    Depositors, // Vec<Address>
}
