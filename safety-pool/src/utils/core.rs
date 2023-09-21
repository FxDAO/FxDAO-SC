use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStats, CoreStorageKeys};
use soroban_sdk::{panic_with_error, Env};

pub const DAY_IN_LEDGERS: u32 = 17280;
pub const INSTANCE_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 30;
pub const INSTANCE_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 20;

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .bump(INSTANCE_BUMP_CONSTANT_THRESHOLD, INSTANCE_BUMP_CONSTANT)
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

pub fn get_core_state(env: &Env) -> CoreState {
    env.storage()
        .instance()
        .get(&CoreStorageKeys::CoreState)
        .unwrap()
}

pub fn set_core_stats(env: &Env, core_stats: &CoreStats) {
    env.storage()
        .instance()
        .set(&CoreStorageKeys::CoreStats, core_stats);
}

pub fn get_core_stats(env: &Env) -> CoreStats {
    env.storage()
        .instance()
        .get(&CoreStorageKeys::CoreStats)
        .unwrap()
}

pub fn set_last_governance_token_distribution_time(env: &Env) {
    env.storage().instance().set(
        &CoreStorageKeys::LastGovernanceTokenDistribution,
        &env.ledger().timestamp(),
    )
}

pub fn get_last_governance_token_distribution_time(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&CoreStorageKeys::LastGovernanceTokenDistribution)
        .unwrap_or(0)
}
