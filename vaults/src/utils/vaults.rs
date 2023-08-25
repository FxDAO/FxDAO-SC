// use crate::storage::storage_types::*;
use crate::storage::vaults::{VaultsDataKeys, VaultsInfo};
// use crate::utils::indexes::{
//     add_new_index_into_indexes_list, get_vaults_data_type_with_index, get_vaults_indexes_list,
//     remove_index_from_indexes_list, remove_vaults_data_type_with_index, save_vaults_indexes_list,
//     set_vaults_data_types_with_index,
// };
use num_integer::div_floor;
use soroban_sdk::{panic_with_error, vec, Address, Env, Symbol, Vec};

pub const PERSISTENT_BUMP_CONSTANT: u32 = 1036800;

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

// pub fn bump_user_vault(env: &Env, user_vault_data_type: UserVaultDataType) {
//     env.storage().persistent().bump(
//         &VaultsDataKeys::UserVault(user_vault_data_type),
//         PERSISTENT_BUMP_CONSTANT,
//     );
// }
//
// pub fn get_user_vault(env: &Env, user_vault_data_type: &UserVaultDataType) -> UserVault {
//     env.storage()
//         .persistent()
//         .get(&VaultsDataKeys::UserVault(user_vault_data_type.clone()))
//         .unwrap()
// }
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
//
// pub fn set_user_vault(env: &Env, user_vault_data_type: &UserVaultDataType, user_vault: &UserVault) {
//     env.storage().persistent().set(
//         &VaultsDataKeys::UserVault(user_vault_data_type.clone()),
//         user_vault,
//     );
// }
//
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
//
// // Validations
// pub fn can_be_liquidated(
//     user_vault: &UserVault,
//     currency: &Currency,
//     currency_vaults_conditions: &CurrencyVaultsConditions,
// ) -> bool {
//     let collateral_value: i128 = currency.rate * user_vault.total_col;
//     let deposit_rate: i128 = div_floor(collateral_value, user_vault.total_debt);
//     deposit_rate < currency_vaults_conditions.min_col_rate
// }
