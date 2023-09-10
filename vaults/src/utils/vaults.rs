// use crate::storage::storage_types::*;
use crate::storage::vaults::{
    OptionalVaultKey, Vault, VaultIndexKey, VaultKey, VaultsDataKeys, VaultsInfo,
};
// use crate::utils::indexes::{
//     add_new_index_into_indexes_list, get_vaults_data_type_with_index, get_vaults_indexes_list,
//     remove_index_from_indexes_list, remove_vaults_data_type_with_index, save_vaults_indexes_list,
//     set_vaults_data_types_with_index,
// };
use crate::errors::SCErrors;
use crate::storage::currencies::Currency;
use crate::utils::currencies::get_currency;
use num_integer::div_floor;
use soroban_sdk::{panic_with_error, vec, Address, Env, Symbol, Vec};

pub const PERSISTENT_BUMP_CONSTANT: u32 = 1036800;

pub fn bump_vault(env: &Env, vault_key: VaultKey) {
    env.storage()
        .persistent()
        .bump(&VaultsDataKeys::Vault(vault_key), PERSISTENT_BUMP_CONSTANT);
}

pub fn bump_vault_index(env: &Env, vault_index_key: VaultIndexKey) {
    env.storage().persistent().bump(
        &VaultsDataKeys::VaultIndex(vault_index_key),
        PERSISTENT_BUMP_CONSTANT,
    );
}

pub fn is_vaults_info_started(env: &Env, denomination: &Symbol) -> bool {
    env.storage()
        .instance()
        .has(&VaultsDataKeys::VaultsInfo(denomination.clone()))
}

pub fn get_vaults_info(env: &Env, denomination: &Symbol) -> VaultsInfo {
    env.storage()
        .instance()
        .get(&VaultsDataKeys::VaultsInfo(denomination.clone()))
        .unwrap()
}

pub fn set_vaults_info(env: &Env, vaults_info: &VaultsInfo) {
    env.storage().instance().set(
        &VaultsDataKeys::VaultsInfo(vaults_info.denomination.clone()),
        vaults_info,
    );
}

/// Creates and insert a Vault into the storage while updating the prev vault in case it exists.
/// **This function doesn't admit errors IE if something goes wrong it must panic.**
///
/// **Arguments:**
/// - `lowest_key` - This key must come from the current (most recent update) lowest key in the storage. If we are making multiple operations, make sure the lowest key passed to this function is the latest state.
/// - `new_vault_key` - We accept the VaultKey of the new vault because it's something that it already includes basic information like: account, denomination and the calculated index of the new vault.
/// - `prev_key` - This value must be the key of the Vault that comes BEFORE the position this new Vault is going to be inserted
/// - `initial_debt` - The debt the new vault will be opened with
/// - `collateral_amount` - The collateral the new vault will be opened with
///
/// **Returns:**
/// - The new saved Vault
/// - The VaultKey of the new Vault
/// - The VaultKeyIndex of the new Vault
/// - The updated value of the `lowest_key`
pub fn create_and_insert_vault(
    env: &Env,
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
            if new_vault_key.index < current_lowest_key.index {
                // Case 2.1: If new index is lower than the lowest index, we continue like if the list was empty but using the old lowest as the next value for the new Vault
                new_vault_next_key = OptionalVaultKey::Some(current_lowest_key);
                updated_lowest_key = OptionalVaultKey::Some(new_vault_key.clone());
            } else {
                // Case 2.2: New vault is higher than the lowest, we need to consider a few things and do some validations
                // - The prev value can not be None
                // - The prev vault must exist
                let prev_key = match prev_key {
                    OptionalVaultKey::None => {
                        panic_with_error!(&env, &SCErrors::PrevVaultCantBeNone);
                    }
                    OptionalVaultKey::Some(key) => key,
                };

                if !has_vault(&env, prev_key.clone()) {
                    panic_with_error!(&env, &SCErrors::PrevVaultDoesntExist);
                }

                let mut prev_vault: Vault = get_vault(&env, prev_key.clone());

                match prev_vault.next_key {
                    // Case 2.2.1: If the prev vault's next key is None, the new Vault will have None as its next key and the prev vault will now have the new vault key as its next key. The lowest key stays the same.
                    OptionalVaultKey::None => {
                        new_vault_next_key = OptionalVaultKey::None;
                        updated_lowest_key = lowest_key.clone();
                        prev_vault.next_key = OptionalVaultKey::Some(new_vault_key.clone());
                        set_vault(&env, &prev_vault);
                    }
                    // Case 2.2.2: If the prev key isn't None, we make this two flows:
                    // - If the prev next key is lower than the new vault, we panic
                    // - If prev next key is higher than the new vault, the new vault will have the current prev next as its next key and the prev vault will update its next key with the new vault. The lowest key stays the same.
                    OptionalVaultKey::Some(current_prev_next_key) => {
                        if current_prev_next_key.index < new_vault_key.index {
                            // TODO: test this
                            panic_with_error!(
                                &env,
                                &SCErrors::PrevVaultNextIndexIsLowerThanNewVault
                            );
                        }

                        new_vault_next_key = OptionalVaultKey::Some(current_prev_next_key);
                        updated_lowest_key = lowest_key.clone();
                        prev_vault.next_key = OptionalVaultKey::Some(new_vault_key.clone());
                        set_vault(&env, &prev_vault);
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
    set_vault(&env, &new_vault);
    set_vault_index(&env, &new_vault_key);

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
    env: &Env,
    user: &Address,
    denomination: &Symbol,
) -> (Vault, VaultKey, VaultIndexKey) {
    let vault_index_key: VaultIndexKey = VaultIndexKey {
        user: user.clone(),
        denomination: denomination.clone(),
    };

    if !has_vault_index(&env, vault_index_key.clone()) {
        panic_with_error!(&env, &SCErrors::VaultDoesntExist);
    }

    let vault_index: u128 = get_vault_index(&env, vault_index_key.clone());
    let vault_key: VaultKey = VaultKey {
        index: vault_index,
        account: user.clone(),
        denomination: denomination.clone(),
    };

    if !has_vault(&env, vault_key.clone()) {
        panic_with_error!(&env, &SCErrors::VaultDoesntExist);
    }

    let user_vault: Vault = get_vault(&env, vault_key.clone());

    (user_vault, vault_key, vault_index_key)
}

pub fn get_vaults(
    env: &Env,
    currency: &Currency,
    vaults_info: &VaultsInfo,
    total: u32,
    only_to_liquidate: bool,
) -> Vec<Vault> {
    let mut vaults: Vec<Vault> = vec![&env] as Vec<Vault>;

    let mut target_key: VaultKey = match vaults_info.lowest_key.clone() {
        OptionalVaultKey::None => {
            panic_with_error!(&env, &SCErrors::NotEnoughVaultsToLiquidate);
        }
        OptionalVaultKey::Some(key) => key,
    };

    for _ in 0..total {
        let vault = get_vault(&env, target_key.clone());

        if !can_be_liquidated(&vault, &currency, &vaults_info) && only_to_liquidate {
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

pub fn get_redeemable_vaults(
    env: &Env,
    vaults_info: &VaultsInfo,
    total_to_redeem: &u128,
) -> (u128, Vec<Vault>) {
    let mut total_can_be_redeemed: u128 = 0u128;
    let mut vaults: Vec<Vault> = vec![&env] as Vec<Vault>;

    let mut target_key: VaultKey = match vaults_info.lowest_key.clone() {
        OptionalVaultKey::None => {
            panic_with_error!(&env, &SCErrors::NotEnoughVaultsToLiquidate);
        }
        OptionalVaultKey::Some(key) => key,
    };

    for _ in 0..10 {
        let vault = get_vault(&env, target_key.clone());

        vaults.push_back(vault.clone());
        total_can_be_redeemed = total_can_be_redeemed + vault.total_debt;

        if total_to_redeem <= &total_can_be_redeemed {
            break;
        }

        if let OptionalVaultKey::Some(key) = vault.next_key {
            target_key = key
        } else {
            break;
        }
    }

    (total_can_be_redeemed, vaults)
}

pub fn get_vault(env: &Env, vault_key: VaultKey) -> Vault {
    env.storage()
        .persistent()
        .get(&VaultsDataKeys::Vault(vault_key))
        .unwrap()
}

pub fn set_vault(env: &Env, vault: &Vault) {
    env.storage().persistent().set(
        &VaultsDataKeys::Vault(VaultKey {
            index: vault.index.clone(),
            account: vault.account.clone(),
            denomination: vault.denomination.clone(),
        }),
        vault,
    );
}

pub fn remove_vault(env: &Env, vault_key: &VaultKey) {
    env.storage()
        .persistent()
        .remove(&VaultsDataKeys::Vault(vault_key.clone()));
}

pub fn set_vault_index(env: &Env, vault_key: &VaultKey) {
    env.storage().persistent().set(
        &VaultsDataKeys::VaultIndex(VaultIndexKey {
            user: vault_key.account.clone(),
            denomination: vault_key.denomination.clone(),
        }),
        &vault_key.index,
    );
}

pub fn remove_vault_index(env: &Env, vault_index_key: &VaultIndexKey) {
    env.storage()
        .persistent()
        .remove(&VaultsDataKeys::VaultIndex(vault_index_key.clone()));
}

pub fn has_vault_index(env: &Env, vault_index_key: VaultIndexKey) -> bool {
    env.storage()
        .persistent()
        .has(&VaultsDataKeys::VaultIndex(vault_index_key))
}

pub fn get_vault_index(env: &Env, vault_index_key: VaultIndexKey) -> u128 {
    env.storage()
        .persistent()
        .get(&VaultsDataKeys::VaultIndex(vault_index_key))
        .unwrap()
}

/// This function checks and removes the given Vault, it also updates the previous Vault if there is one.
/// **This function doesn't admit errors IE if something goes wrong it must panic.**
///
/// This function doesn't update nor care about the lowest_key of the general contract, that's something the contract needs to handle either before or after calling this function.
///
/// **Arguments:**
/// - `vault` - Target Vault to remove from the storage
/// - `prev_key` - This value must be the key of the Vault that comes BEFORE the position the Vault we are going to remove
pub fn withdraw_vault(env: &Env, vault: &Vault, prev_key: &OptionalVaultKey) {
    let target_vault_key: VaultKey = VaultKey {
        index: vault.index.clone(),
        account: vault.account.clone(),
        denomination: vault.denomination.clone(),
    };

    if let OptionalVaultKey::Some(key) = prev_key {
        let (mut prev_vault, _, _) = search_vault(&env, &key.account, &key.denomination);

        // We check that the Next Key correctly targets the target Vault
        // If the Next key is None, it means the target Vault is not the Vault that comes after this one
        if let OptionalVaultKey::Some(k) = prev_vault.next_key {
            if &k != &target_vault_key {
                panic_with_error!(&env, &SCErrors::PrevVaultNextIndexIsInvalid);
            }
        } else {
            panic_with_error!(&env, &SCErrors::PrevVaultNextIndexIsInvalid);
        }

        prev_vault.next_key = vault.next_key.clone();
        set_vault(&env, &prev_vault);
    }

    remove_vault(&env, &target_vault_key);
    remove_vault_index(
        &env,
        &VaultIndexKey {
            user: vault.account.clone(),
            denomination: vault.denomination.clone(),
        },
    );
}

//
// pub fn save_new_user_vault(
//     env: &Env,
//     user_vault: &UserVault,
//     user_vault_data_type: &UserVaultDataType,
//     vaults_data_types_with_index_key: &VaultsDataKeys,
//     vaults_indexes_list_key: &VaultsDataKeys,
// ) {
//     if user_vault.index <= 0 {
//         panic_with_error!(env, &SCErrors::UserVaultIndexIsInvalid);
//     }
//
//     set_user_vault(env, user_vault_data_type, user_vault);
//
//     let mut saved_vaults_with_index: Vec<UserVaultDataType> =
//         get_vaults_data_type_with_index(env, &vaults_data_types_with_index_key);
//
//     saved_vaults_with_index =
//         add_vault_to_vaults_with_index(&saved_vaults_with_index, &user_vault_data_type);
//
//     set_vaults_data_types_with_index(
//         &env,
//         &vaults_data_types_with_index_key,
//         &saved_vaults_with_index,
//     );
//
//     let mut indexes_list: Vec<i128> = get_vaults_indexes_list(env, &vaults_indexes_list_key);
//     indexes_list = add_new_index_into_indexes_list(&indexes_list.clone(), user_vault.index.clone());
//     save_vaults_indexes_list(&env, &vaults_indexes_list_key, &indexes_list);
// }
//
// /// 1 - This method updates the individual user vault with the new values
// ///
// /// 2 - It gets the saved Vaults with the old index and remove the vault from the record
// ///
// /// 2.1 - If the record's Vector is now blank (length == 0), it removes it instead of saving it
// ///
// /// 3 - Adds the user vault data key into the new vaults with index record (the record for the new index)
// ///
// /// 4 - It updates the sorted list of indexes by adding the new index
// ///
// /// 4.1 - If the old record is blank, it removes the old index from the list
// pub fn update_user_vault(
//     env: &Env,
//     current_user_vault: &UserVault,
//     new_user_vault: &UserVault,
//     user_vault_data_type: &UserVaultDataType,
//     vaults_indexes_list_key: &VaultsDataKeys,
//     current_vaults_data_types_with_index_key: &VaultsDataKeys,
//     new_vaults_data_types_with_index_key: &VaultsDataKeys,
// ) {
//     if new_user_vault.index <= 0 {
//         panic_with_error!(env, &SCErrors::UserVaultIndexIsInvalid);
//     }
//
//     set_user_vault(env, &user_vault_data_type, &new_user_vault);
//
//     let mut old_vaults_with_index_record: Vec<UserVaultDataType> =
//         get_vaults_data_type_with_index(env, &current_vaults_data_types_with_index_key);
//
//     old_vaults_with_index_record = remove_vault_from_vaults_with_index(
//         &old_vaults_with_index_record,
//         &user_vault_data_type.user,
//         &user_vault_data_type.denomination,
//     );
//
//     if old_vaults_with_index_record.len() == 0 {
//         remove_vaults_data_type_with_index(&env, &current_vaults_data_types_with_index_key);
//     } else {
//         set_vaults_data_types_with_index(
//             &env,
//             &current_vaults_data_types_with_index_key,
//             &old_vaults_with_index_record,
//         );
//     }
//
//     let mut new_vaults_with_index_record: Vec<UserVaultDataType> =
//         get_vaults_data_type_with_index(env, &new_vaults_data_types_with_index_key);
//
//     new_vaults_with_index_record =
//         add_vault_to_vaults_with_index(&new_vaults_with_index_record, &user_vault_data_type);
//
//     set_vaults_data_types_with_index(
//         &env,
//         &new_vaults_data_types_with_index_key,
//         &new_vaults_with_index_record,
//     );
//
//     let mut indexes_list = get_vaults_indexes_list(env, &vaults_indexes_list_key);
//
//     if old_vaults_with_index_record.len() == 0 {
//         indexes_list = remove_index_from_indexes_list(&indexes_list, current_user_vault.index);
//     }
//
//     indexes_list = add_new_index_into_indexes_list(&indexes_list, new_user_vault.index);
//     save_vaults_indexes_list(&env, &vaults_indexes_list_key, &indexes_list);
// }
//
// /// 1 - This method removes the UserVault from the UserVault(UserVaultDataType) key
// ///
// /// 2 - It gets the saved Vaults with the current index and remove the vault data type from the record
// ///
// /// 3 - If the record of the Vector is now blank (length == 0):
// ///
// /// 3.1 - it removes it instead of saving it
// ///
// /// 3.2 - It gets the indexes list and remove the index from it
// pub fn remove_user_vault(
//     env: &Env,
//     user_vault: &UserVault,
//     user_vault_data_type: &UserVaultDataType,
//     vaults_data_types_with_index_key: &VaultsDataKeys,
//     vaults_indexes_list_key: &VaultsDataKeys, // VaultsDataKeys::VaultsIndexes
// ) {
//     env.storage()
//         .persistent()
//         .remove(&VaultsDataKeys::UserVault(user_vault_data_type.clone()));
//
//     let mut vaults_with_index_record: Vec<UserVaultDataType> =
//         get_vaults_data_type_with_index(env, &vaults_data_types_with_index_key);
//
//     vaults_with_index_record = remove_vault_from_vaults_with_index(
//         &vaults_with_index_record,
//         &user_vault_data_type.user,
//         &user_vault_data_type.denomination,
//     );
//
//     if vaults_with_index_record.len() == 0 {
//         remove_vaults_data_type_with_index(&env, &vaults_data_types_with_index_key);
//     } else {
//         set_vaults_data_types_with_index(
//             &env,
//             &vaults_data_types_with_index_key,
//             &vaults_with_index_record,
//         );
//     }
//
//     if vaults_with_index_record.len() == 0 {
//         let mut indexes_list = get_vaults_indexes_list(env, &vaults_indexes_list_key);
//         indexes_list = remove_index_from_indexes_list(&indexes_list, user_vault.index);
//         save_vaults_indexes_list(env, vaults_indexes_list_key, &indexes_list);
//     }
// }

// pub fn get_redeemable_vaults(
//     env: &Env,
//     amount: &i128,
//     currency: &Currency,
//     vaults_indexes_list_key: &VaultsDataKeys, // VaultsDataKeys::VaultsIndexes
// ) -> Vec<UserVault> {
//     let sorted_indexes_list: Vec<i128> = get_vaults_indexes_list(env, &vaults_indexes_list_key);
//     let mut users_vaults: Vec<UserVault> = vec![env] as Vec<UserVault>;
//     let mut covered_amount: i128 = 0;
//
//     for index in sorted_indexes_list.iter() {
//         let vaults_with_index: Vec<UserVaultDataType> = get_vaults_data_type_with_index(
//             env,
//             &VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
//                 index,
//                 denomination: currency.denomination.clone(),
//             }),
//         );
//
//         for data_type in vaults_with_index.iter() {
//             let user_vault: UserVault = env
//                 .storage()
//                 .persistent()
//                 .get(&VaultsDataKeys::UserVault(data_type))
//                 .unwrap();
//
//             covered_amount = covered_amount + user_vault.total_debt;
//             users_vaults.push_back(user_vault);
//         }
//
//         if &covered_amount >= amount {
//             break;
//         }
//     }
//
//     users_vaults
// }
//
// // Functional utils
//
// pub fn add_vault_to_vaults_with_index(
//     record: &Vec<UserVaultDataType>,
//     user_vault_data_type: &UserVaultDataType,
// ) -> Vec<UserVaultDataType> {
//     let mut updated_record: Vec<UserVaultDataType> = record.clone();
//     let mut saved: bool = false;
//
//     for vault_data_key in updated_record.iter() {
//         if user_vault_data_type.user == vault_data_key.user
//             && user_vault_data_type.denomination == vault_data_key.denomination
//         {
//             saved = true;
//         }
//     }
//
//     if !saved {
//         updated_record.push_back(user_vault_data_type.clone());
//     }
//
//     updated_record
// }
//
// pub fn remove_vault_from_vaults_with_index(
//     record: &Vec<UserVaultDataType>,
//     user: &Address,
//     denomination: &Symbol,
// ) -> Vec<UserVaultDataType> {
//     let mut updated_record = record.clone();
//
//     for (i, vault_data_key) in updated_record.iter().enumerate() {
//         if &vault_data_key.user == user && &vault_data_key.denomination == denomination {
//             updated_record.remove(i as u32);
//         }
//     }
//
//     updated_record
// }

// Validations
pub fn has_vault(env: &Env, vault_key: VaultKey) -> bool {
    env.storage()
        .persistent()
        .has(&VaultsDataKeys::Vault(vault_key))
}

pub fn vault_spot_available(env: &Env, user: Address, denomination: &Symbol) {
    if env
        .storage()
        .persistent()
        .has(&VaultsDataKeys::VaultIndex(VaultIndexKey {
            user,
            denomination: denomination.clone(),
        }))
    {
        panic_with_error!(&env, &SCErrors::UserAlreadyHasDenominationVault);
    }
}

pub fn validate_user_vault(env: &Env, vault_key: VaultKey) {
    if !env
        .storage()
        .persistent()
        .has(&VaultsDataKeys::Vault(vault_key))
    {
        panic_with_error!(&env, SCErrors::VaultDoesntExist);
    }
}

pub fn can_be_liquidated(
    user_vault: &Vault,
    currency: &Currency,
    vaults_info: &VaultsInfo,
) -> bool {
    let collateral_value: u128 = currency.rate * user_vault.total_collateral;
    let deposit_rate: u128 = div_floor(collateral_value, user_vault.total_debt);
    deposit_rate < vaults_info.min_col_rate
}
