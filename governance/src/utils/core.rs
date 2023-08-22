use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStorageKeys};
use soroban_sdk::{panic_with_error, token, Address, Env, Map, Symbol, Vec};

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

pub fn get_governance_token(env: &Env) -> (Address, token::Client) {
    let core_state: CoreState = get_core_state(&env);

    (
        core_state.governance_token.clone(),
        token::Client::new(&env, &core_state.governance_token),
    )
}

pub fn save_managing_contracts(env: &Env, addresses: &Vec<Address>) {
    env.storage()
        .instance()
        .set(&CoreStorageKeys::ManagingContracts, addresses);
}

pub fn get_managing_contracts(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&CoreStorageKeys::ManagingContracts)
        .unwrap()
}

pub fn save_allowed_contracts_functions(env: &Env, data: &Map<Address, Vec<Symbol>>) {
    env.storage()
        .instance()
        .set(&CoreStorageKeys::AllowedContractsFunctions, data);
}

pub fn get_allowed_contracts_functions(env: &Env) -> Map<Address, Vec<Symbol>> {
    env.storage()
        .instance()
        .get(&CoreStorageKeys::AllowedContractsFunctions)
        .unwrap()
}
