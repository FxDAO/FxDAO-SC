use soroban_sdk::{contracttype, Address, Symbol, Vec};

#[contracttype]
pub struct CoreState {
    pub governance_token: Address,
    pub proposals_fee: u128,
    pub voting_credit_price: u128,
    pub contract_admin: Address,
    pub cooldown_period: u64,
}

#[contracttype]
pub enum CoreStorageKeys {
    CoreState,
    /// These are the contracts and functions this DAO manages and is allowed to call/upgrade
    ManagingContracts, // Returns Vec<Address>
    AllowedContractsFunctions, // Returns Map<Address, Vec<Symbol>>
}
