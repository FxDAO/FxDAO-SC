use soroban_sdk::{contracttype, Address, Symbol, Vec};

#[contracttype]
#[derive(PartialEq, Debug, Clone)]
pub struct CoreState {
    pub admin: Address,
    pub vaults_contract: Address,
    pub treasury_contract: Address,
    pub collateral_asset: Address,
    pub deposit_asset: Address,
    pub denomination_asset: Symbol,
    pub min_deposit: u128,
    pub treasury_share: Vec<u32>,
    pub liquidator_share: Vec<u32>,
    pub governance_token: Address,
}

#[contracttype]
pub enum CoreStorageKeys {
    CoreState,                       // CoreState
    LastGovernanceTokenDistribution, // u64
}
