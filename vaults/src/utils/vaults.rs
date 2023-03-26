use crate::storage_types::{SCErrors, UserVault, UserVaultDataType, VaultsDataKeys};
use soroban_sdk::{panic_with_error, vec, Address, Env, Symbol, Vec};

pub fn save_new_user_vault(
    env: &Env,
    user: &Address,
    denomination: &Symbol,
    user_vault: &UserVault,
) {
    set_user_vault(env, user, denomination, user_vault);
    add_vault_ratio(env, user, denomination, &user_vault.index);
    save_new_ratio_into_indexes_list(env, user_vault.index.clone(), denomination.clone());
}

pub fn set_user_vault(env: &Env, user: &Address, denomination: &Symbol, user_vault: &UserVault) {
    env.storage().set(
        &VaultsDataKeys::UserVault(UserVaultDataType {
            user: user.clone(),
            symbol: denomination.clone(),
        }),
        user_vault,
    );
}

pub fn get_vault_ratio(env: &Env, ratio: &i128) -> Vec<UserVaultDataType> {
    env.storage()
        .get(&VaultsDataKeys::UsersRatio(ratio.clone()))
        .unwrap_or(Ok(vec![env] as Vec<UserVaultDataType>))
        .unwrap()
}

pub fn add_vault_ratio(env: &Env, user: &Address, denomination: &Symbol, index: &i128) {
    let mut saved_ratio: Vec<UserVaultDataType> = get_vault_ratio(env, index);

    let mut saved: bool = false;
    for item in saved_ratio.iter() {
        match item {
            Ok(value) => {
                if user.clone() == value.user && denomination.clone() == value.symbol {
                    saved = true;
                }
            }
            Err(_) => {}
        }
    }

    if !saved {
        saved_ratio.push_front(UserVaultDataType {
            user: user.clone(),
            symbol: denomination.clone(),
        });
        env.storage()
            .set(&VaultsDataKeys::UsersRatio(index.clone()), &saved_ratio);
    }
}

pub fn calculate_user_vault_index(total_debt: i128, total_col: i128) -> i128 {
    (total_debt - total_col).abs()
}

pub fn get_sorted_indexes_list(env: &Env, denomination: Symbol) -> Vec<i128> {
    env.storage()
        .get(&VaultsDataKeys::Indexes(denomination))
        .unwrap_or(Ok(vec![env] as Vec<i128>))
        .unwrap()
}

pub fn save_new_ratio_into_indexes_list(env: &Env, index: i128, denomination: Symbol) {
    if index == 0 || index < 0 {
        panic_with_error!(env, &SCErrors::UserVaultRatioIsInvalid);
    }

    let mut indexes_list: Vec<i128> = get_sorted_indexes_list(env, denomination);
    let first_value: i128 = indexes_list.first().unwrap_or(Ok(0)).unwrap();
    let last_value: i128 = indexes_list.last().unwrap_or(Ok(0)).unwrap();

    if first_value > index {
        indexes_list.push_front(index);
        env.storage()
            .set(&VaultsDataKeys::Indexes(denomination), &indexes_list);
    } else if last_value < index {
        indexes_list.push_back(index);
        env.storage()
            .set(&VaultsDataKeys::Indexes(denomination), &indexes_list);
    } else if last_value != index && first_value != index {
        match indexes_list.binary_search(index) {
            Ok(_) => {} // element already in vector @ `pos`
            Err(pos) => indexes_list.insert(pos, index),
        }
        env.storage()
            .set(&VaultsDataKeys::Indexes(denomination), &indexes_list);
    }
}
