use soroban_sdk::{contracttype, Address, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct UserVaultDataType {
    pub user: Address,
    pub denomination: Symbol,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct UserVault {
    pub id: Address,
    pub total_debt: i128,
    pub total_col: i128,
    pub index: i128,
    pub denomination: Symbol,
}

#[derive(Clone)]
#[contracttype]
pub struct VaultsWithIndexDataType {
    pub index: i128,
    pub denomination: Symbol,
}

/// I need to be able to check who is the lowest collateral ratio no matter the currency
/// I need to be able to check the lowest one without needing to load a huge vector of values
/// I need to be able to sort the vec from lower to higher in an efficient way
#[contracttype]
pub enum VaultsDataKeys {
    /// The "UserVault" key is the one that actually holds the information of the user's vault
    /// Everytime this key is updated we need to update both "SortedVlts" and "RatioKey"
    UserVault(UserVaultDataType),
    /// This key host a Vec of i128 which is the index of the vaults, this Vec must be updated every time a Vault is updated
    /// The Vec is sorted by the collateral ratio of the deposit IE the lower go first
    /// The Symbol value is the denomination of the currency
    VaultsIndexes(Symbol),

    /// The result is a Vec<UserVaultDataType>
    VaultsDataTypesWithIndex(VaultsWithIndexDataType),
}
