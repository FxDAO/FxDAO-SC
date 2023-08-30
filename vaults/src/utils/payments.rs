use crate::storage::core::CoreState;
use crate::storage::currencies::Currency;
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol};

pub fn deposit_collateral(env: &Env, core_state: &CoreState, depositor: &Address, amount: i128) {
    token::Client::new(&env, &core_state.col_token).transfer(
        depositor,
        &env.current_contract_address(),
        &amount,
    );
}

pub fn withdraw_stablecoin(
    env: &Env,
    core_state: &CoreState,
    currency: &Currency,
    recipient: &Address,
    amount: i128,
) {
    token::Client::new(&env, &currency.contract).transfer_from(
        &env.current_contract_address(),
        &core_state.stable_issuer,
        recipient,
        &amount,
    );
}

pub fn withdraw_collateral(env: &Env, core_state: &CoreState, requester: &Address, amount: i128) {
    token::Client::new(&env, &core_state.col_token).transfer(
        &env.current_contract_address(),
        requester,
        &amount,
    );
}

pub fn deposit_stablecoin(
    env: &Env,
    core_state: &CoreState,
    currency: &Currency,
    depositor: &Address,
    amount: i128,
) {
    token::Client::new(&env, &currency.contract).transfer(
        depositor,
        &env.current_contract_address(),
        &amount,
    );

    token::Client::new(&env, &currency.contract).transfer(
        &env.current_contract_address(),
        &core_state.stable_issuer,
        &amount,
    );
}
