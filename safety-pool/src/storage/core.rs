use soroban_sdk::{contracttype, Address, BytesN, Symbol};

#[contracttype]
pub struct CoreState {
    pub vaults_contract: Address,
    pub treasury_contract: Address,
    pub collateral_asset: BytesN<32>,
    pub deposit_asset: BytesN<32>,
    pub denomination_asset: Symbol,
    pub min_deposit: u128,
}

#[contracttype]
pub enum CoreStorageKeys {
    Admin,
    CoreState,
}
