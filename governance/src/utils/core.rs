use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStorageKeys};
use soroban_sdk::{panic_with_error, token, Address, BytesN, Env};

pub fn can_init_contract(env: &Env) {
    if env.storage().has(&CoreStorageKeys::CoreState) {
        panic_with_error!(&env, SCErrors::ContractAlreadyInitiated);
    }
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

pub fn get_governance_token(env: &Env) -> (Address, token::Client) {
    let core_state: CoreState = get_core_state(&env);

    (
        core_state.governance_token.clone(),
        token::Client::new(&env, &core_state.governance_token),
    )
}
