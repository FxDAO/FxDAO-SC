use crate::storage_types::{UserVaultDataType, VaultsDataKeys, VaultsWithIndexDataType};
use soroban_sdk::{vec, Env, Symbol, Vec};

pub fn get_vaults_data_type_with_index(
    env: &Env,
    denomination: &Symbol,
    index: &i128,
) -> Vec<UserVaultDataType> {
    env.storage()
        .persistent()
        .get(&VaultsDataKeys::VaultsWithIndex(VaultsWithIndexDataType {
            index: index.clone(),
            denomination: denomination.clone(),
        }))
        .unwrap_or(vec![&env] as Vec<UserVaultDataType>)
}

pub fn set_vaults_data_type_with_index(
    env: &Env,
    denomination: &Symbol,
    index: &i128,
    vaults: &Vec<UserVaultDataType>,
) {
    env.storage().persistent().set(
        &VaultsDataKeys::VaultsWithIndex(VaultsWithIndexDataType {
            index: index.clone(),
            denomination: denomination.clone(),
        }),
        vaults,
    );
}

pub fn remove_vaults_data_type_with_index(env: &Env, denomination: &Symbol, index: &i128) {
    env.storage()
        .persistent()
        .remove(&VaultsDataKeys::VaultsWithIndex(VaultsWithIndexDataType {
            index: index.clone(),
            denomination: denomination.clone(),
        }));
}
