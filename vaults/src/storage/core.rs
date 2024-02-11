use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct CoreState {
    pub col_token: Address,
    pub stable_issuer: Address,
    pub admin: Address,
    pub protocol_manager: Address,
    pub panic_mode: bool,
    pub treasury: Address,
    pub fee: u128,
    pub oracle: Address,
}

#[contracttype]
pub enum CoreDataKeys {
    CoreState,
}
