use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStorageKeys};
use soroban_sdk::{panic_with_error, Env};

pub fn get_core_state(env: &Env) -> CoreState {
    env.storage()
        .get(&CoreStorageKeys::CoreState)
        .unwrap()
        .unwrap()
}

pub fn can_init_contract(env: &Env) {
    if env.storage().has(&CoreStorageKeys::CoreState) {
        panic_with_error!(&env, SCErrors::ContractAlreadyInitiated);
    }
}

pub fn set_core_state(env: &Env, core_state: &CoreState) {
    env.storage().set(&CoreStorageKeys::CoreState, core_state);
}

pub fn get_last_governance_token_distribution_time(env: &Env) -> u64 {
    env.storage()
        .get(&CoreStorageKeys::LastGovernanceTokenDistribution)
        .unwrap_or(Ok(0))
        .unwrap()
}

pub fn set_last_governance_token_distribution_time(env: &Env) {
    env.storage().set(
        &CoreStorageKeys::LastGovernanceTokenDistribution,
        &env.ledger().timestamp(),
    )
}
