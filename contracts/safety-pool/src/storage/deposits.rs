use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Deposit {
    pub depositor: Address,
    pub amount: u128,
    pub last_deposit: u64,
    pub shares: u128,
    pub share_price_paid: u128,
    pub liquidation_index: u64,
}

#[contracttype]
pub enum DepositsDataKeys {
    Deposit(Address), // Deposit
    Depositors,       // Vec<Address>
}
