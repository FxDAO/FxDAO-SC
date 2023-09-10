use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Deposit {
    pub depositor: Address,
    pub amount: u128,
    pub last_deposit: u64,
    pub current_collateral_factor: u128,
    pub current_deposit_factor: u128,
}

#[contracttype]
pub enum DepositsDataKeys {
    Deposit(Address), // Deposit
    Depositors,       // Vec<Address>
}
