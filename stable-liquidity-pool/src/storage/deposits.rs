use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct Deposit {
    pub depositor: Address,
    pub shares: u128,
    pub locked: bool,
    pub unlocks_at: u64,

    // This is the snapshot of the factor at the moment of locking this deposit
    pub snapshot: u128,
}

#[contracttype]
pub enum DepositsDataKeys {
    Deposit(Address),
}
