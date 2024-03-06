use soroban_sdk::{contracttype, Address, Env, Symbol};

pub const DAY_IN_LEDGERS: u32 = 17280;
pub const PERSISTENT_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 28;
pub const PERSISTENT_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 14;

#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum OptionalVaultKey {
    None,
    Some(VaultKey),
}

#[contracttype]
#[derive(Debug, Clone)]
pub struct VaultsInfo {
    pub denomination: Symbol,
    pub total_vaults: u64,
    pub total_debt: u128,
    pub total_col: u128,
    pub lowest_key: OptionalVaultKey,
    pub min_col_rate: u128,
    // Min collateral ratio - ex: 1.10
    pub min_debt_creation: u128,
    // Min vault creation amount - ex: 5000
    pub opening_col_rate: u128, // Opening collateral ratio - ex: 1.15
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VaultKey {
    pub index: u128,
    pub account: Address,
    pub denomination: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Vault {
    pub index: u128,
    pub next_key: OptionalVaultKey,
    pub account: Address,
    pub total_debt: u128,
    pub total_collateral: u128,
    pub denomination: Symbol,
}

#[contracttype]
#[derive(Clone, Debug)]
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

pub trait VaultsFunc {
    fn bump_vault(&self, vault_key: &VaultKey);
    fn bump_vault_index(&self, vault_index_key: &VaultIndexKey);
    fn vaults_info(&self, denomination: &Symbol) -> Option<VaultsInfo>;
    fn set_vaults_info(&self, vaults_info: &VaultsInfo);
    fn vault(&self, vault_key: &VaultKey) -> Option<Vault>;
    fn set_vault(&self, vault: &Vault);
    fn remove_vault(&self, vault_key: &VaultKey);
    fn set_vault_index(&self, vault_key: &VaultKey);
    fn remove_vault_index(&self, vault_index_key: &VaultIndexKey);
    fn vault_index(&self, vault_index_key: &VaultIndexKey) -> Option<u128>;
}

impl VaultsFunc for Env {
    fn bump_vault(&self, vault_key: &VaultKey) {
        self.storage().persistent().extend_ttl(
            &VaultsDataKeys::Vault(vault_key.clone()),
            PERSISTENT_BUMP_CONSTANT_THRESHOLD,
            PERSISTENT_BUMP_CONSTANT,
        );
    }

    fn bump_vault_index(&self, vault_index_key: &VaultIndexKey) {
        self.storage().persistent().extend_ttl(
            &VaultsDataKeys::VaultIndex(vault_index_key.clone()),
            PERSISTENT_BUMP_CONSTANT_THRESHOLD,
            PERSISTENT_BUMP_CONSTANT,
        );
    }

    fn vaults_info(&self, denomination: &Symbol) -> Option<VaultsInfo> {
        self.storage()
            .instance()
            .get(&VaultsDataKeys::VaultsInfo(denomination.clone()))
    }

    fn set_vaults_info(&self, vaults_info: &VaultsInfo) {
        self.storage().instance().set(
            &VaultsDataKeys::VaultsInfo(vaults_info.denomination.clone()),
            vaults_info,
        );
    }

    fn vault(&self, vault_key: &VaultKey) -> Option<Vault> {
        self.storage()
            .persistent()
            .get(&VaultsDataKeys::Vault(vault_key.clone()))
    }

    fn set_vault(&self, vault: &Vault) {
        self.storage().persistent().set(
            &VaultsDataKeys::Vault(VaultKey {
                index: vault.index.clone(),
                account: vault.account.clone(),
                denomination: vault.denomination.clone(),
            }),
            vault,
        );
    }

    fn remove_vault(&self, vault_key: &VaultKey) {
        self.storage()
            .persistent()
            .remove(&VaultsDataKeys::Vault(vault_key.clone()));
    }

    fn set_vault_index(&self, vault_key: &VaultKey) {
        self.storage().persistent().set(
            &VaultsDataKeys::VaultIndex(VaultIndexKey {
                user: vault_key.account.clone(),
                denomination: vault_key.denomination.clone(),
            }),
            &vault_key.index,
        );
    }

    fn remove_vault_index(&self, vault_index_key: &VaultIndexKey) {
        self.storage()
            .persistent()
            .remove(&VaultsDataKeys::VaultIndex(vault_index_key.clone()));
    }

    fn vault_index(&self, vault_index_key: &VaultIndexKey) -> Option<u128> {
        self.storage()
            .persistent()
            .get(&VaultsDataKeys::VaultIndex(vault_index_key.clone()))
    }
}
