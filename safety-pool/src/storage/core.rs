use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec};

#[contracttype]
#[derive(PartialEq, Debug, Clone)]
pub struct CoreState {
    pub admin: Address,
    pub vaults_contract: Address,
    pub treasury_contract: Address,
    pub collateral_asset: BytesN<32>,
    pub deposit_asset: BytesN<32>,
    pub denomination_asset: Symbol,
    pub min_deposit: u128,
    pub treasury_share: Vec<u128>,
}

#[contracttype]
pub enum CoreStorageKeys {
    CoreState,
}
