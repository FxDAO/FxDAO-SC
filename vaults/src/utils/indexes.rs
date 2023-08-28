use crate::storage::vaults::*;
use num_integer::div_floor;
use soroban_sdk::{vec, Env, Symbol, Vec};
pub const INDEXES_BUMP_CONSTANT: u32 = 120960;

// pub fn bump_vaults_data_types_with_index(
//     env: &Env,
//     vaults_data_types_with_index_key: &VaultsDataKeys, // VaultsDataKeys::UsersVaultsDataTypesWithIndex
// ) {
//     if env
//         .storage()
//         .persistent()
//         .has(vaults_data_types_with_index_key)
//     {
//         env.storage()
//             .persistent()
//             .bump(vaults_data_types_with_index_key, INDEXES_BUMP_CONSTANT);
//     }
// }
//
// pub fn bump_vaults_indexes_list(
//     env: &Env,
//     vaults_indexes_list_key: &VaultsDataKeys, // VaultsDataKeys::VaultsIndexes
// ) {
//     if env.storage().persistent().has(vaults_indexes_list_key) {
//         env.storage()
//             .persistent()
//             .bump(vaults_indexes_list_key, INDEXES_BUMP_CONSTANT);
//     }
// }
//
// pub fn get_vaults_data_type_with_index(
//     env: &Env,
//     vaults_data_types_with_index_key: &VaultsDataKeys, // &VaultsDataKeys::VaultsWithIndex(VaultsWithIndexDataType)
// ) -> Vec<UserVaultDataType> {
//     env.storage()
//         .persistent()
//         .get(vaults_data_types_with_index_key)
//         .unwrap_or(vec![&env] as Vec<UserVaultDataType>)
// }
//
// pub fn set_vaults_data_types_with_index(
//     env: &Env,
//     vaults_data_types_with_index_key: &VaultsDataKeys,
//     vaults: &Vec<UserVaultDataType>,
// ) {
//     env.storage()
//         .persistent()
//         .set(vaults_data_types_with_index_key, vaults);
// }
//
// // vaults_data_types_with_index_key = VaultsDataKeys::VaultsDataTypesWithIndex
// pub fn remove_vaults_data_type_with_index(
//     env: &Env,
//     vaults_data_types_with_index_key: &VaultsDataKeys,
// ) {
//     env.storage()
//         .persistent()
//         .remove(vaults_data_types_with_index_key);
// }

pub fn calculate_user_vault_index(total_debt: u128, total_collateral: u128) -> u128 {
    div_floor(1000000000 * total_collateral, total_debt)
}

// pub fn add_new_index_into_indexes_list(indexes_list: &Vec<i128>, index: i128) -> Vec<i128> {
//     let mut updated_indexes_list: Vec<i128> = indexes_list.clone();
//     let first_value: i128 = updated_indexes_list.first().unwrap_or(0);
//     let last_value: i128 = updated_indexes_list.last().unwrap_or(0);
//
//     if first_value > index {
//         updated_indexes_list.push_front(index);
//     } else if last_value < index {
//         updated_indexes_list.push_back(index);
//     } else if last_value != index && first_value != index {
//         match updated_indexes_list.binary_search(index) {
//             Ok(_) => {} // element already in vector @ `pos`
//             Err(pos) => updated_indexes_list.insert(pos, index),
//         }
//     }
//
//     updated_indexes_list
// }
//
// pub fn remove_index_from_indexes_list(indexes_list: &Vec<i128>, index: i128) -> Vec<i128> {
//     let mut updated_indexes_list: Vec<i128> = indexes_list.clone();
//     let first_value: i128 = updated_indexes_list.first().unwrap_or(0);
//     let last_value: i128 = updated_indexes_list.last().unwrap_or(0);
//
//     if first_value == index {
//         updated_indexes_list.pop_front();
//     } else if last_value == index {
//         updated_indexes_list.pop_back();
//     } else {
//         match updated_indexes_list.binary_search(index) {
//             Ok(pos) => {
//                 updated_indexes_list.remove(pos);
//             }
//             Err(_) => {} // If we don't find it we don't use that position
//         }
//     }
//
//     updated_indexes_list
// }
//
// // vaults_indexes_list_key: &VaultsDataKeys::VaultsIndexes
// pub fn get_vaults_indexes_list(env: &Env, vaults_indexes_list_key: &VaultsDataKeys) -> Vec<i128> {
//     env.storage()
//         .persistent()
//         .get(vaults_indexes_list_key)
//         .unwrap_or(vec![env] as Vec<i128>)
// }
//
// pub fn save_vaults_indexes_list(
//     env: &Env,
//     vaults_indexes_list_key: &VaultsDataKeys,
//     indexes_list: &Vec<i128>,
// ) {
//     env.storage()
//         .persistent()
//         .set(vaults_indexes_list_key, indexes_list);
// }
//
// #[cfg(test)]
// mod test {
//     use crate::utils::indexes::{
//         add_new_index_into_indexes_list, calculate_user_vault_index, remove_index_from_indexes_list,
//     };
//     use soroban_sdk::{vec, Env, Vec};
//
//     // TODO: test add_vault_to_vaults_with_index and remove_vault_from_vaults_with_index
//
//     #[test]
//     fn test_calculate_user_vault_index() {
//         // Case 1
//         let total_debt_1 = 5000_0000000;
//         let total_col_1 = 5000_0000000;
//         let result_1 = calculate_user_vault_index(total_debt_1, total_col_1);
//
//         assert_eq!(result_1, 1000000000);
//
//         // Case 2
//         let total_debt_2 = 100_0000000;
//         let total_col_2 = 3000_0000000;
//         let result_2 = calculate_user_vault_index(total_debt_2, total_col_2);
//
//         assert_eq!(result_2, 30000000000);
//
//         // Case 3
//         let total_debt_3 = 3000_0000000;
//         let total_col_3 = 100_0000000;
//         let result_3 = calculate_user_vault_index(total_debt_3, total_col_3);
//
//         assert_eq!(result_3, 33333333);
//
//         // Case 4
//         let total_debt_4 = 29999999999;
//         let total_col_4 = 1000000000;
//         let result_4 = calculate_user_vault_index(total_debt_4, total_col_4);
//
//         assert_eq!(result_4, 33333333);
//
//         // Case 5
//         let total_debt_5 = 1000000000;
//         let total_col_5 = 29999999999;
//         let result_5 = calculate_user_vault_index(total_debt_5, total_col_5);
//
//         assert_eq!(result_5, 29999999999);
//     }
//
//     #[test]
//     fn test_add_new_index_into_indexes_list() {
//         let env = Env::default();
//         let first_index = 1000000000 as i128;
//         let mut original_vector: Vec<i128> = vec![&env] as Vec<i128>;
//
//         original_vector = add_new_index_into_indexes_list(&original_vector, first_index);
//
//         assert_eq!(original_vector, vec![&env, first_index]);
//
//         let second_index: i128 = 30000000000;
//
//         original_vector = add_new_index_into_indexes_list(&original_vector, second_index);
//
//         assert_eq!(original_vector, vec![&env, first_index, second_index]);
//
//         let third_index: i128 = 33333333;
//
//         original_vector = add_new_index_into_indexes_list(&original_vector, third_index);
//
//         assert_eq!(
//             original_vector,
//             vec![&env, third_index, first_index, second_index]
//         );
//
//         let fourth_index: i128 = 33333333;
//
//         original_vector = add_new_index_into_indexes_list(&original_vector, fourth_index);
//
//         assert_eq!(
//             original_vector,
//             vec![&env, third_index, first_index, second_index]
//         );
//
//         let fifth_index: i128 = 29999999999;
//
//         original_vector = add_new_index_into_indexes_list(&original_vector, fifth_index);
//
//         assert_eq!(
//             original_vector,
//             vec![&env, third_index, first_index, fifth_index, second_index]
//         );
//
//         let sixth_index: i128 = 900000000;
//
//         original_vector = add_new_index_into_indexes_list(&original_vector, sixth_index);
//
//         assert_eq!(
//             original_vector,
//             vec![
//                 &env,
//                 third_index,
//                 sixth_index,
//                 first_index,
//                 fifth_index,
//                 second_index
//             ]
//         );
//     }
//
//     #[test]
//     fn test_remove_index_from_indexes_list() {
//         let env = Env::default();
//
//         let first_index: i128 = 1000000000;
//         let second_index: i128 = 30000000000;
//         let third_index: i128 = 33333333;
//         let fourth_index: i128 = 33333333;
//         let fifth_index: i128 = 29999999999;
//         let sixth_index: i128 = 900000000;
//
//         let mut original_vector: Vec<i128> = vec![
//             &env,
//             third_index,
//             sixth_index,
//             first_index,
//             fifth_index,
//             second_index,
//         ] as Vec<i128>;
//
//         original_vector = remove_index_from_indexes_list(&original_vector, third_index);
//
//         assert_eq!(
//             original_vector,
//             vec![&env, sixth_index, first_index, fifth_index, second_index,] as Vec<i128>
//         );
//
//         original_vector = remove_index_from_indexes_list(&original_vector, second_index);
//
//         assert_eq!(
//             original_vector,
//             vec![&env, sixth_index, first_index, fifth_index,] as Vec<i128>
//         );
//
//         original_vector = remove_index_from_indexes_list(&original_vector, first_index);
//
//         assert_eq!(
//             original_vector,
//             vec![&env, sixth_index, fifth_index,] as Vec<i128>
//         );
//
//         original_vector = remove_index_from_indexes_list(&original_vector, fourth_index);
//
//         assert_eq!(
//             original_vector,
//             vec![&env, sixth_index, fifth_index,] as Vec<i128>
//         );
//
//         original_vector = remove_index_from_indexes_list(&original_vector, fifth_index);
//
//         assert_eq!(original_vector, vec![&env, sixth_index,] as Vec<i128>);
//
//         original_vector = remove_index_from_indexes_list(&original_vector, sixth_index);
//
//         assert_eq!(original_vector, vec![&env] as Vec<i128>);
//     }
// }
