use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct Deposit {
    pub depositor: Address,
    pub amount: u128,
    pub last_deposit: u64,
}

#[contracttype]
pub enum DepositsDataKeys {
    Deposit(Address), // Returns a Deposit
    Depositors,       // Returns a Vec<Address>
}
