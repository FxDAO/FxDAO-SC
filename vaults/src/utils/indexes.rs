use crate::storage_types::{UserVaultDataType, VaultsDataKeys, VaultsWithIndexDataType};
use soroban_sdk::{vec, Env, Symbol, Vec};

pub fn get_vaults_data_type_with_index(
    env: &Env,
    denomination: &Symbol,
    index: &i128,
) -> Vec<UserVaultDataType> {
    env.storage()
        .get(&VaultsDataKeys::VaultsWithIndex(VaultsWithIndexDataType {
            index: index.clone(),
            denomination: denomination.clone(),
        }))
        .unwrap_or(Ok(vec![&env] as Vec<UserVaultDataType>))
        .unwrap()
}

pub fn set_vaults_data_type_with_index(
    env: &Env,
    denomination: &Symbol,
    index: &i128,
    vaults: &Vec<UserVaultDataType>,
) {
    env.storage().set(
        &VaultsDataKeys::VaultsWithIndex(VaultsWithIndexDataType {
            index: index.clone(),
            denomination: denomination.clone(),
        }),
        vaults,
    );
}

pub fn remove_vaults_data_type_with_index(env: &Env, denomination: &Symbol, index: &i128) {
    env.storage()
        .remove(&VaultsDataKeys::VaultsWithIndex(VaultsWithIndexDataType {
            index: index.clone(),
            denomination: denomination.clone(),
        }));
}
