use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
pub struct CoreState {
    pub governance_token: Address,
    pub proposals_fee: u128,
    pub voting_credit_price: u128,
    pub contract_admin: Address,
}

#[contracttype]
pub enum CoreStorageKeys {
    CoreState,
}
