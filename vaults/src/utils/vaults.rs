use crate::storage_types::{
    Currency, CurrencyVaultsConditions, SCErrors, UserVault, UserVaultDataType, VaultsDataKeys,
};
use crate::utils::indexes::{
    get_vaults_data_type_with_index, remove_vaults_data_type_with_index,
    set_vaults_data_type_with_index,
};
use num_integer::div_floor;
use soroban_sdk::{panic_with_error, vec, Address, Env, Symbol, Vec};

pub fn get_user_vault(env: &Env, user: &Address, denomination: &Symbol) -> UserVault {
    env.storage()
        .get(&VaultsDataKeys::UserVault(UserVaultDataType {
            user: user.clone(),
            denomination: denomination.clone(),
        }))
        .unwrap()
        .unwrap()
}

pub fn save_new_user_vault(
    env: &Env,
    user: &Address,
    denomination: &Symbol,
    user_vault: &UserVault,
) {
    if user_vault.index <= 0 {
        panic_with_error!(env, &SCErrors::UserVaultIndexIsInvalid);
    }

    set_user_vault(env, user, denomination, user_vault);

    let mut saved_vaults_with_index: Vec<UserVaultDataType> =
        get_vaults_data_type_with_index(env, &denomination, &user_vault.index);

    saved_vaults_with_index =
        add_vault_to_vaults_with_index(&saved_vaults_with_index, &user, &denomination);

    set_vaults_data_type_with_index(
        &env,
        &denomination,
        &user_vault.index,
        &saved_vaults_with_index,
    );

    let mut indexes_list = get_sorted_indexes_list(env, denomination);
    indexes_list = add_new_index_into_indexes_list(&indexes_list.clone(), user_vault.index.clone());

    env.storage().set(
        &VaultsDataKeys::Indexes(denomination.clone()),
        &indexes_list,
    );
}

/// 1 - This method updates the individual user vault with the new values
///
/// 2 - It gets the saved Vaults with the old index and remove the vault from the record
///
/// 2.1 - If the record's Vector is now blank (length == 0), it removes it instead of saving it
///
/// 3 - Adds the user vault data key into the new vaults with index record (the record for the new index)
///
/// 4 - It updates the sorted list of indexes by adding the new index
///
/// 4.1 - If the old record is blank, it removes the old index from the list
pub fn update_user_vault(
    env: &Env,
    user: &Address,
    denomination: &Symbol,
    current_user_vault: &UserVault,
    new_user_vault: &UserVault,
) {
    if new_user_vault.index <= 0 {
        panic_with_error!(env, &SCErrors::UserVaultIndexIsInvalid);
    }

    set_user_vault(env, user, denomination, new_user_vault);

    let mut old_vaults_with_index_record: Vec<UserVaultDataType> =
        get_vaults_data_type_with_index(env, &denomination, &current_user_vault.index);

    old_vaults_with_index_record =
        remove_vault_from_vaults_with_index(&old_vaults_with_index_record, &user, &denomination);

    if old_vaults_with_index_record.len() == 0 {
        remove_vaults_data_type_with_index(&env, &denomination, &current_user_vault.index);
    } else {
        set_vaults_data_type_with_index(
            &env,
            &denomination,
            &current_user_vault.index,
            &old_vaults_with_index_record,
        );
    }

    let mut new_vaults_with_index_record: Vec<UserVaultDataType> =
        get_vaults_data_type_with_index(env, &denomination, &new_user_vault.index);

    new_vaults_with_index_record =
        add_vault_to_vaults_with_index(&new_vaults_with_index_record, &user, &denomination);

    set_vaults_data_type_with_index(
        &env,
        &denomination,
        &new_user_vault.index,
        &new_vaults_with_index_record,
    );

    let mut indexes_list = get_sorted_indexes_list(env, denomination);

    if old_vaults_with_index_record.len() == 0 {
        indexes_list = remove_index_from_indexes_list(&indexes_list, current_user_vault.index);
    }

    indexes_list = add_new_index_into_indexes_list(&indexes_list, new_user_vault.index);

    env.storage().set(
        &VaultsDataKeys::Indexes(denomination.clone()),
        &indexes_list,
    );
}

/// 1 - This method removes the UserVault from the UserVault(UserVaultDataType) key
///
/// 2 - It gets the saved Vaults with the current index and remove the vault data type from the record
///
/// 3 - If the record of the Vector is now blank (length == 0):
///
/// 3.1 - it removes it instead of saving it
///
/// 3.2 - It gets the indexes list and remove the index from it
pub fn remove_user_vault(env: &Env, user: &Address, denomination: &Symbol, user_vault: &UserVault) {
    env.storage()
        .remove(&VaultsDataKeys::UserVault(UserVaultDataType {
            user: user.clone(),
            denomination: denomination.clone(),
        }));

    let mut vaults_with_index_record: Vec<UserVaultDataType> =
        get_vaults_data_type_with_index(env, &denomination, &user_vault.index);

    vaults_with_index_record =
        remove_vault_from_vaults_with_index(&vaults_with_index_record, &user, &denomination);

    if vaults_with_index_record.len() == 0 {
        remove_vaults_data_type_with_index(&env, &denomination, &user_vault.index);
    } else {
        set_vaults_data_type_with_index(
            &env,
            &denomination,
            &user_vault.index,
            &vaults_with_index_record,
        );
    }

    if vaults_with_index_record.len() == 0 {
        let mut indexes_list = get_sorted_indexes_list(env, denomination);
        indexes_list = remove_index_from_indexes_list(&indexes_list, user_vault.index);
        env.storage().set(
            &VaultsDataKeys::Indexes(denomination.clone()),
            &indexes_list,
        );
    }
}

pub fn set_user_vault(env: &Env, user: &Address, denomination: &Symbol, user_vault: &UserVault) {
    env.storage().set(
        &VaultsDataKeys::UserVault(UserVaultDataType {
            user: user.clone(),
            denomination: denomination.clone(),
        }),
        user_vault,
    );
}

pub fn get_sorted_indexes_list(env: &Env, denomination: &Symbol) -> Vec<i128> {
    env.storage()
        .get(&VaultsDataKeys::Indexes(denomination.clone()))
        .unwrap_or(Ok(vec![env] as Vec<i128>))
        .unwrap()
}

pub fn get_redeemable_vaults(env: &Env, amount: &i128, currency: &Currency) -> Vec<UserVault> {
    let sorted_indexes_list: Vec<i128> = get_sorted_indexes_list(env, &currency.denomination);
    let mut users_vaults: Vec<UserVault> = vec![env] as Vec<UserVault>;
    let mut covered_amount: i128 = 0;

    for item in sorted_indexes_list.iter() {
        let vaults_with_index: Vec<UserVaultDataType> =
            get_vaults_data_type_with_index(env, &currency.denomination, &item.unwrap());

        for data_type in vaults_with_index.iter() {
            let user_vault: UserVault = env
                .storage()
                .get(&VaultsDataKeys::UserVault(data_type.unwrap()))
                .unwrap()
                .unwrap();

            covered_amount = covered_amount + user_vault.total_debt;
            users_vaults.push_back(user_vault);
        }

        if &covered_amount >= amount {
            break;
        }
    }

    users_vaults
}

// Functional utils

pub fn add_vault_to_vaults_with_index(
    record: &Vec<UserVaultDataType>,
    user: &Address,
    denomination: &Symbol,
) -> Vec<UserVaultDataType> {
    let mut updated_record: Vec<UserVaultDataType> = record.clone();
    let mut saved: bool = false;

    for item in updated_record.iter() {
        let vault_data_key = item.unwrap();
        if user == &vault_data_key.user && denomination == &vault_data_key.denomination {
            saved = true;
        }
    }

    if !saved {
        updated_record.push_back(UserVaultDataType {
            user: user.clone(),
            denomination: denomination.clone(),
        });
    }

    updated_record
}

pub fn remove_vault_from_vaults_with_index(
    record: &Vec<UserVaultDataType>,
    user: &Address,
    denomination: &Symbol,
) -> Vec<UserVaultDataType> {
    let mut updated_record = record.clone();

    for (i, el) in updated_record.iter().enumerate() {
        let vault_data_key = el.unwrap();

        if &vault_data_key.user == user && &vault_data_key.denomination == denomination {
            updated_record.remove(i as u32);
        }
    }

    updated_record
}

pub fn calculate_user_vault_index(total_debt: i128, total_col: i128) -> i128 {
    div_floor(1000000000 * total_col, total_debt)
}

pub fn add_new_index_into_indexes_list(indexes_list: &Vec<i128>, index: i128) -> Vec<i128> {
    let mut updated_indexes_list: Vec<i128> = indexes_list.clone();
    let first_value: i128 = updated_indexes_list.first().unwrap_or(Ok(0)).unwrap();
    let last_value: i128 = updated_indexes_list.last().unwrap_or(Ok(0)).unwrap();

    if first_value > index {
        updated_indexes_list.push_front(index);
    } else if last_value < index {
        updated_indexes_list.push_back(index);
    } else if last_value != index && first_value != index {
        match updated_indexes_list.binary_search(index) {
            Ok(_) => {} // element already in vector @ `pos`
            Err(pos) => updated_indexes_list.insert(pos, index),
        }
    }

    updated_indexes_list
}

pub fn remove_index_from_indexes_list(indexes_list: &Vec<i128>, index: i128) -> Vec<i128> {
    let mut updated_indexes_list: Vec<i128> = indexes_list.clone();
    let first_value: i128 = updated_indexes_list.first().unwrap_or(Ok(0)).unwrap();
    let last_value: i128 = updated_indexes_list.last().unwrap_or(Ok(0)).unwrap();

    if first_value == index {
        updated_indexes_list.pop_front();
    } else if last_value == index {
        updated_indexes_list.pop_back();
    } else {
        match updated_indexes_list.binary_search(index) {
            Ok(pos) => {
                updated_indexes_list.remove(pos);
            }
            Err(_) => {} // If we don't find it we don't use that position
        }
    }

    updated_indexes_list
}

// Validations
pub fn can_be_liquidated(
    user_vault: &UserVault,
    currency: &Currency,
    currency_vaults_conditions: &CurrencyVaultsConditions,
) -> bool {
    let collateral_value: i128 = currency.rate * user_vault.total_col;
    let deposit_rate: i128 = div_floor(collateral_value, user_vault.total_debt);
    deposit_rate < currency_vaults_conditions.min_col_rate
}

#[cfg(test)]
mod test {
    use crate::utils::vaults::{
        add_new_index_into_indexes_list, calculate_user_vault_index, remove_index_from_indexes_list,
    };
    use soroban_sdk::{vec, Env, Vec};

    // TODO: test add_vault_to_vaults_with_index and remove_vault_from_vaults_with_index

    #[test]
    fn test_calculate_user_vault_index() {
        // Case 1
        let total_debt_1 = 5000_0000000;
        let total_col_1 = 5000_0000000;
        let result_1 = calculate_user_vault_index(total_debt_1, total_col_1);

        assert_eq!(result_1, 1000000000);

        // Case 2
        let total_debt_2 = 100_0000000;
        let total_col_2 = 3000_0000000;
        let result_2 = calculate_user_vault_index(total_debt_2, total_col_2);

        assert_eq!(result_2, 30000000000);

        // Case 3
        let total_debt_3 = 3000_0000000;
        let total_col_3 = 100_0000000;
        let result_3 = calculate_user_vault_index(total_debt_3, total_col_3);

        assert_eq!(result_3, 33333333);

        // Case 4
        let total_debt_4 = 29999999999;
        let total_col_4 = 1000000000;
        let result_4 = calculate_user_vault_index(total_debt_4, total_col_4);

        assert_eq!(result_4, 33333333);

        // Case 5
        let total_debt_5 = 1000000000;
        let total_col_5 = 29999999999;
        let result_5 = calculate_user_vault_index(total_debt_5, total_col_5);

        assert_eq!(result_5, 29999999999);
    }

    #[test]
    fn test_add_new_index_into_indexes_list() {
        let env = Env::default();
        let first_index = 1000000000 as i128;
        let mut original_vector: Vec<i128> = vec![&env] as Vec<i128>;

        original_vector = add_new_index_into_indexes_list(&original_vector, first_index);

        assert_eq!(original_vector, vec![&env, first_index]);

        let second_index: i128 = 30000000000;

        original_vector = add_new_index_into_indexes_list(&original_vector, second_index);

        assert_eq!(original_vector, vec![&env, first_index, second_index]);

        let third_index: i128 = 33333333;

        original_vector = add_new_index_into_indexes_list(&original_vector, third_index);

        assert_eq!(
            original_vector,
            vec![&env, third_index, first_index, second_index]
        );

        let fourth_index: i128 = 33333333;

        original_vector = add_new_index_into_indexes_list(&original_vector, fourth_index);

        assert_eq!(
            original_vector,
            vec![&env, third_index, first_index, second_index]
        );

        let fifth_index: i128 = 29999999999;

        original_vector = add_new_index_into_indexes_list(&original_vector, fifth_index);

        assert_eq!(
            original_vector,
            vec![&env, third_index, first_index, fifth_index, second_index]
        );

        let sixth_index: i128 = 900000000;

        original_vector = add_new_index_into_indexes_list(&original_vector, sixth_index);

        assert_eq!(
            original_vector,
            vec![
                &env,
                third_index,
                sixth_index,
                first_index,
                fifth_index,
                second_index
            ]
        );
    }

    #[test]
    fn test_remove_index_from_indexes_list() {
        let env = Env::default();

        let first_index: i128 = 1000000000;
        let second_index: i128 = 30000000000;
        let third_index: i128 = 33333333;
        let fourth_index: i128 = 33333333;
        let fifth_index: i128 = 29999999999;
        let sixth_index: i128 = 900000000;

        let mut original_vector: Vec<i128> = vec![
            &env,
            third_index,
            sixth_index,
            first_index,
            fifth_index,
            second_index,
        ] as Vec<i128>;

        original_vector = remove_index_from_indexes_list(&original_vector, third_index);

        assert_eq!(
            original_vector,
            vec![&env, sixth_index, first_index, fifth_index, second_index,] as Vec<i128>
        );

        original_vector = remove_index_from_indexes_list(&original_vector, second_index);

        assert_eq!(
            original_vector,
            vec![&env, sixth_index, first_index, fifth_index,] as Vec<i128>
        );

        original_vector = remove_index_from_indexes_list(&original_vector, first_index);

        assert_eq!(
            original_vector,
            vec![&env, sixth_index, fifth_index,] as Vec<i128>
        );

        original_vector = remove_index_from_indexes_list(&original_vector, fourth_index);

        assert_eq!(
            original_vector,
            vec![&env, sixth_index, fifth_index,] as Vec<i128>
        );

        original_vector = remove_index_from_indexes_list(&original_vector, fifth_index);

        assert_eq!(original_vector, vec![&env, sixth_index,] as Vec<i128>);

        original_vector = remove_index_from_indexes_list(&original_vector, sixth_index);

        assert_eq!(original_vector, vec![&env] as Vec<i128>);
    }
}
