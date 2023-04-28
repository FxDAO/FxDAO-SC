use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
pub struct CoreState {
    pub vaults_contract: Address,
    pub deposit_asset: BytesN<32>,
    pub min_deposit: u128,
}

#[contracttype]
pub enum CoreStorageKeys {
    Admin,
    CoreState,
}
