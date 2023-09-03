pub const INSTANCE_BUMP_CONSTANT: u32 = 507904;

use crate::storage::core::{CoreDataKeys, CoreState};
use soroban_sdk::{Address, Env};

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .bump(env.ledger().sequence() + INSTANCE_BUMP_CONSTANT)
}

pub fn is_core_created(env: &Env) -> bool {
    env.storage().instance().has(&CoreDataKeys::CoreState)
}

pub fn save_core_state(env: &Env, core_state: &CoreState) {
    env.storage()
        .instance()
        .set(&CoreDataKeys::CoreState, core_state);
}

pub fn get_core_state(env: &Env) -> CoreState {
    env.storage()
        .instance()
        .get(&CoreDataKeys::CoreState)
        .unwrap()
}
