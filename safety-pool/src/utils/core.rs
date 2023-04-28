use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStorageKeys};
use soroban_sdk::{panic_with_error, Address, Env};

pub fn can_init_contract(env: &Env) {
    if env.storage().has(&CoreStorageKeys::CoreState) {
        panic_with_error!(&env, SCErrors::ContractAlreadyInitiated);
    }
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().set(&CoreStorageKeys::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().get(&CoreStorageKeys::Admin).unwrap().unwrap()
}

pub fn set_core_state(env: &Env, core_state: &CoreState) {
    env.storage().set(&CoreStorageKeys::CoreState, core_state);
}

pub fn get_core_state(env: &Env) -> CoreState {
    env.storage()
        .get(&CoreStorageKeys::CoreState)
        .unwrap()
        .unwrap()
}
