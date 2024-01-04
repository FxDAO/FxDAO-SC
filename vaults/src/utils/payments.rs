use crate::storage::core::CoreState;
use crate::storage::currencies::Currency;
use num_integer::{div_ceil, div_floor};
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol};

pub fn calc_fee(fee: &u128, amount: &u128) -> u128 {
    div_ceil(amount * fee, 1_0000000)
}

pub fn pay_fee(env: &Env, core_state: &CoreState, payer: &Address, fee: i128) {
    token::Client::new(env, &core_state.col_token).transfer(payer, &core_state.treasury, &fee);
}

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
