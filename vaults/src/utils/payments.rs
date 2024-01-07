use crate::storage::core::CoreState;
use crate::storage::currencies::Currency;
use num_integer::{div_ceil, div_floor};
use soroban_sdk::{self, panic_with_error, token, Address, Env, Symbol};

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

pub fn withdraw_collateral(env: &Env, core_state: &CoreState, requester: &Address, amount: i128) {
    token::Client::new(&env, &core_state.col_token).transfer(
        &env.current_contract_address(),
        requester,
        &amount,
    );
}

pub fn mint_stablecoin(env: &Env, currency: &Currency, recipient: &Address, amount: i128) {
    token::StellarAssetClient::new(&env, &currency.contract).mint(&recipient, &amount);
}

pub fn burn_stablecoin(env: &Env, currency: &Currency, depositor: &Address, amount: i128) {
    token::Client::new(&env, &currency.contract).burn(depositor, &amount);
}
