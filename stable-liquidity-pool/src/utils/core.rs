use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStorageKeys};
use soroban_sdk::{panic_with_error, Env};
pub const INSTANCE_BUMP_CONSTANT: u32 = 507904;

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .bump(env.ledger().sequence() + INSTANCE_BUMP_CONSTANT)
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

pub fn get_last_governance_token_distribution_time(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&CoreStorageKeys::LastGovernanceTokenDistribution)
        .unwrap_or(0)
}

pub fn set_last_governance_token_distribution_time(env: &Env) {
    env.storage().instance().set(
        &CoreStorageKeys::LastGovernanceTokenDistribution,
        &env.ledger().timestamp(),
    )
}
