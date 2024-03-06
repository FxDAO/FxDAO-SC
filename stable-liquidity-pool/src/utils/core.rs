use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStorageKeys};
use soroban_sdk::{panic_with_error, Env};

pub const INSTANCE_BUMP_CONSTANT: u32 = 507904;
pub const INSTANCE_BUMP_CONSTANT_THRESHOLD: u32 = 253952;

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_BUMP_CONSTANT_THRESHOLD, INSTANCE_BUMP_CONSTANT)
}

pub fn get_core_state(env: &Env) -> CoreState {
    env.storage()
        .instance()
        .get(&CoreStorageKeys::CoreState)
        .unwrap()
}

pub fn can_init_contract(env: &Env) {
    if env.storage().instance().has(&CoreStorageKeys::CoreState) {
        panic_with_error!(&env, SCErrors::ContractAlreadyInitiated);
    }
}

pub fn set_core_state(env: &Env, core_state: &CoreState) {
    env.storage()
        .instance()
        .set(&CoreStorageKeys::CoreState, core_state);
}