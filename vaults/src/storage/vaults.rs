use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
pub struct VaultsInfo {
    pub denomination: Symbol,
    pub total_vaults: u64,
    pub total_debt: u128,
    pub total_col: u128,
    pub lowest_index: u128,
    pub min_col_rate: u128,      // Min collateral ratio - ex: 1.10
    pub min_debt_creation: u128, // Min vault creation amount - ex: 5000
    pub opening_col_rate: u128,  // Opening collateral ratio - ex: 1.15
}

#[contracttype]
pub struct VaultKey {
    pub index: u128,
    pub denomination: Symbol,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Vault {
    pub index: i128,
    pub next_index: i128,
    pub account: Address,
    pub total_debt: i128,
    pub total_collateral: i128,
    pub denomination: Symbol,
}

#[contracttype]
pub struct VaultIndexKey {
    pub user: Address,
    pub denomination: Symbol,
}

#[contracttype]
pub enum VaultsDataKeys {
    /// General information by currency.
    /// Symbol is the denomination, not the asset code.
    VaultsInfo(Symbol),

    /// By using the index and denomination (VaultKey) we can get a Vault, all Vaults' indexes are unique.
    /// In cases where the index (collateral / debt) is the same as one already created, we add 1 to it until is unique
    Vault(VaultKey),

    /// By using the combination of the denomination and the address (VaultIndexKey) we can get
    /// the index of the vault so the user doesn't need to know the index of its own vault at all time
    VaultIndex(VaultIndexKey),
}
