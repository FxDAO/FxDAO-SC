use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
pub struct CoreState {
    pub admin: Address,
    pub manager: Address,
    pub governance_token: Address,
    pub accepted_assets: Vec<Address>,
    // For example 0.3% = 0.003 = 30000
    pub fee_percentage: u128,
    pub total_deposited: u128,
    pub share_price: u128,
    pub total_shares: u128,
    pub treasury: Address,
}

#[contracttype]
pub enum CoreStorageKeys {
    CoreState,
}
