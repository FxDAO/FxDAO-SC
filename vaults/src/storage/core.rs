use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct CoreState {
    pub col_token: Address,
    pub stable_issuer: Address,
    pub admin: Address,
    pub oracle_admin: Address,
    pub protocol_manager: Address,
    pub panic_mode: bool,
    pub treasury: Address,
    pub fee: u128,
}

#[contracttype]
pub enum CoreDataKeys {
    CoreState,
}
