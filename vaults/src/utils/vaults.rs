use crate::errors::SCErrors;
use crate::storage::vaults::{
    OptionalVaultKey, Vault, VaultIndexKey, VaultKey, VaultsFunc, VaultsInfo,
};
use num_integer::div_floor;
use soroban_sdk::{panic_with_error, Address, Env, Symbol, Vec};

// Creates and insert a Vault into the storage while updating the prev vault in case it exists.
// **This function doesn't admit errors IE if something goes wrong it must panic.**
//
// **Arguments:**
// - `lowest_key` - This key must come from the current (most recent update) lowest key in the storage. If we are making multiple operations, make sure the lowest key passed to this function is the latest state.
// - `new_vault_key` - We accept the VaultKey of the new vault because it's something that it already includes basic information like: account, denomination and the calculated index of the new vault.
// - `prev_key` - This value must be the key of the Vault that comes BEFORE the position this new Vault is going to be inserted
// - `initial_debt` - The debt the new vault will be opened with
// - `collateral_amount` - The collateral the new vault will be opened with
//
// **Returns:**
// - The new saved Vault
// - The VaultKey of the new Vault
// - The VaultKeyIndex of the new Vault
// - The updated value of the `lowest_key`
pub fn create_and_insert_vault(
    e: &Env,
    lowest_key: &OptionalVaultKey,
    new_vault_key: &VaultKey,
    prev_key: &OptionalVaultKey,
    initial_debt: u128,
    collateral_amount: u128,
) -> (Vault, VaultKey, VaultIndexKey, OptionalVaultKey) {
    let new_vault_next_key: OptionalVaultKey;
    let updated_lowest_key: OptionalVaultKey;
    match lowest_key.clone() {
        // Case 1: If lowest key is None, it means this is the first vault and so we don't need to do any other major validation
        OptionalVaultKey::None => {
            new_vault_next_key = OptionalVaultKey::None;
            updated_lowest_key = OptionalVaultKey::Some(new_vault_key.clone());
        }

        // Case 2: If the lowest key exists, it means the list is not empty and we need to consider some scenarios
        OptionalVaultKey::Some(current_lowest_key) => {
            if new_vault_key.index <= current_lowest_key.index {
                // Case 2.1: If new index is lower or equal than the lowest index, we continue like if the list was empty but using the old lowest as the next value for the new Vault
                new_vault_next_key = OptionalVaultKey::Some(current_lowest_key);
                updated_lowest_key = OptionalVaultKey::Some(new_vault_key.clone());
            } else {
                // Case 2.2: New vault is higher than the lowest, we need to consider a few things and do some validations
                // - The prev value can not be None
                // - The prev vault must exist
                let prev_key = match prev_key {
                    OptionalVaultKey::None => {
                        panic_with_error!(&e, &SCErrors::PrevVaultCantBeNone);
                    }
                    OptionalVaultKey::Some(key) => key,
                };

                let mut prev_vault: Vault = e
                    .vault(&prev_key)
                    .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::PrevVaultDoesntExist));

                match prev_vault.next_key {
                    // Case 2.2.1: If the prev vault's next key is None, the new Vault will have None as its next key and the prev vault will now have the new vault key as its next key. The lowest key stays the same.
                    OptionalVaultKey::None => {
                        new_vault_next_key = OptionalVaultKey::None;
                        updated_lowest_key = lowest_key.clone();
                        prev_vault.next_key = OptionalVaultKey::Some(new_vault_key.clone());
                        e.set_vault(&prev_vault);
                    }
                    // Case 2.2.2: If the prev key isn't None, we make this two flows:
                    // - If the prev next key is lower than the new vault, we panic
                    // - If prev next key is higher than the new vault, the new vault will have the current prev next as its next key and the prev vault will update its next key with the new vault. The lowest key stays the same.
                    OptionalVaultKey::Some(current_prev_next_key) => {
                        if current_prev_next_key.index < new_vault_key.index {
                            // TODO: test this
                            panic_with_error!(&e, &SCErrors::PrevVaultNextIndexIsLowerThanNewVault);
                        }

                        new_vault_next_key = OptionalVaultKey::Some(current_prev_next_key);
                        updated_lowest_key = lowest_key.clone();
                        prev_vault.next_key = OptionalVaultKey::Some(new_vault_key.clone());
                        e.set_vault(&prev_vault);
                    }
                }
            }
        }
    }

    let new_vault: Vault = Vault {
        next_key: new_vault_next_key,
        denomination: new_vault_key.denomination.clone(),
        account: new_vault_key.account.clone(),
        total_debt: initial_debt,
        total_collateral: collateral_amount,
        index: new_vault_key.index.clone(),
    };
    e.set_vault(&new_vault);
    e.set_vault_index(&new_vault_key);

    (
        new_vault,
        new_vault_key.clone(),
        VaultIndexKey {
            user: new_vault_key.account.clone(),
            denomination: new_vault_key.denomination.clone(),
        },
        updated_lowest_key,
    )
}

pub fn search_vault(
    e: &Env,
    user: &Address,
    denomination: &Symbol,
) -> (Vault, VaultKey, VaultIndexKey) {
    let vault_index_key: VaultIndexKey = VaultIndexKey {
        user: user.clone(),
        denomination: denomination.clone(),
    };

    let vault_index: u128 = e
        .vault_index(&vault_index_key)
        .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::VaultDoesntExist));

    let vault_key: VaultKey = VaultKey {
        index: vault_index,
        account: user.clone(),
        denomination: denomination.clone(),
    };

    let user_vault: Vault = e
        .vault(&vault_key)
        .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::VaultDoesntExist));

    (user_vault, vault_key, vault_index_key)
}

pub fn get_vaults(
    e: &Env,
    prev_key: &OptionalVaultKey,
    vaults_info: &VaultsInfo,
    total: u32,
    only_to_liquidate: bool,
    rate: u128,
) -> Vec<Vault> {
    let mut vaults: Vec<Vault> = Vec::new(&e);

    let mut target_key: VaultKey;
    if let OptionalVaultKey::Some(vault_key) = prev_key {
        target_key = vault_key.clone();
    } else {
        target_key = match vaults_info.lowest_key.clone() {
            OptionalVaultKey::None => {
                // We can not pass a OptionalVaultKey::None to this function
                panic_with_error!(&e, &SCErrors::UnexpectedError);
            }
            OptionalVaultKey::Some(key) => key,
        };
    }

    for _ in 0..total {
        let vault: Vault = e.vault(&target_key).unwrap();

        if !can_be_liquidated(&vault, &vaults_info, &rate) && only_to_liquidate {
            break;
        }

        vaults.push_back(vault.clone());

        if let OptionalVaultKey::Some(key) = vault.next_key {
            target_key = key
        } else {
            break;
        }
    }

    vaults
}

// This function checks and removes the given Vault, it also updates the previous Vault if there is one.
// **This function doesn't admit errors IE if something goes wrong it must panic.**
//
// This function doesn't update nor care about the lowest_key of the general contract, that's something the contract needs to handle either before or after calling this function.
//
// **Arguments:**
// - `vault` - Target Vault to remove from the storage
// - `prev_key` - This value must be the key of the Vault that comes BEFORE the position the Vault we are going to remove
pub fn withdraw_vault(e: &Env, vault: &Vault, prev_key: &OptionalVaultKey) {
    let target_vault_key: VaultKey = VaultKey {
        index: vault.index.clone(),
        account: vault.account.clone(),
        denomination: vault.denomination.clone(),
    };

    if let OptionalVaultKey::Some(key) = prev_key {
        let (mut prev_vault, _, _) = search_vault(&e, &key.account, &key.denomination);

        // We check that the Next Key correctly targets the target Vault
        // If the Next key is None, it means the target Vault is not the Vault that comes after this one
        if let OptionalVaultKey::Some(k) = prev_vault.next_key {
            if &k != &target_vault_key {
                panic_with_error!(&e, &SCErrors::PrevVaultNextIndexIsInvalid);
            }
        } else {
            panic_with_error!(&e, &SCErrors::PrevVaultNextIndexIsInvalid);
        }

        prev_vault.next_key = vault.next_key.clone();
        e.set_vault(&prev_vault);
    }

    e.remove_vault(&target_vault_key);
    e.remove_vault_index(&VaultIndexKey {
        user: vault.account.clone(),
        denomination: vault.denomination.clone(),
    });
}

pub fn calculate_deposit_ratio(currency_rate: &u128, collateral: &u128, debt: &u128) -> u128 {
    div_floor(currency_rate * collateral, debt.clone())
}

pub fn can_be_liquidated(user_vault: &Vault, vaults_info: &VaultsInfo, rate: &u128) -> bool {
    let collateral_value: u128 = rate * user_vault.total_collateral;
    let deposit_rate: u128 = div_floor(collateral_value, user_vault.total_debt);
    deposit_rate < vaults_info.min_col_rate
}
