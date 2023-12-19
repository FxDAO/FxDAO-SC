pub const DAY_IN_LEDGERS: u32 = 17280;
pub const INSTANCE_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 28;
pub const INSTANCE_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 14;

use crate::storage::core::{CoreDataKeys, CoreState};
use soroban_sdk::Env;

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_BUMP_CONSTANT_THRESHOLD, INSTANCE_BUMP_CONSTANT);
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
