pub const INSTANCE_BUMP_CONSTANT: u32 = 507904;

use crate::storage::storage_types::*;
use soroban_sdk::{Address, Env};

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .bump(env.ledger().sequence() + INSTANCE_BUMP_CONSTANT)
}

pub fn save_core_state(env: &Env, core_state: &CoreState) {
    env.storage()
        .instance()
        .set(&DataKeys::CoreState, core_state);
}

pub fn get_core_state(env: &Env) -> CoreState {
    env.storage().instance().get(&DataKeys::CoreState).unwrap()
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKeys::Admin).unwrap()
}
