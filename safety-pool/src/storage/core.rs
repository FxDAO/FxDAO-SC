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
#[derive(PartialEq, Debug, Clone)]
pub struct CoreStats {
    /// total amount of value deposited since inception
    pub lifetime_deposited: u128,
    pub current_deposited: u128,

    /// collateral profited since inception (value between the amount paid for and the amount received)
    pub lifetime_profit: u128,

    /// collateral liquidated overtime since inception
    pub lifetime_liquidated: u128,
    pub current_liquidated: u128,

    /// The collateral factor is the value used to keep track of the collateral each depositor
    /// owns and should be able to withdraw.
    pub collateral_factor: u128,

    /// The deposit factor is used to keep a record of relation between the amounts deposited and
    /// that what has been already used in liquidations, this helps to calculate the amount each
    /// depositor should be able to withdraw. (This is basically a shares price)
    pub deposit_factor: u128,
}

#[contracttype]
pub enum CoreStorageKeys {
    CoreState,                       // CoreState
    CoreStats,                       // CoreStats
    LastGovernanceTokenDistribution, // u64
}
