use crate::storage_types::*;
use crate::token;
use soroban_sdk::{panic_with_error, Address, BytesN, Env};

pub fn check_admin(env: &Env, caller: Address) {
    let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

    if core_state.admin != caller {
        panic_with_error!(&env, SCErrors::Unauthorized);
    }
}

pub fn get_core_state(env: &Env) -> CoreState {
    env.storage().get(&DataKeys::CoreState).unwrap().unwrap()
}

pub fn get_protocol_state(env: &Env) -> ProtocolState {
    env.storage().get(&DataKeys::ProtState).unwrap().unwrap()
}

pub fn get_protocol_collateral_price(env: &Env) -> ProtocolCollateralPrice {
    env.storage()
        .get(&DataKeys::ProtRate)
        .unwrap_or(Ok(ProtocolCollateralPrice {
            last_updte: env.ledger().timestamp(),
            current: 0,
        }))
        .unwrap()
}

pub fn valid_initial_debt(env: &Env, state: &ProtocolState, initial_debt: u128) {
    if state.mn_v_c_amt > initial_debt {
        panic_with_error!(env, SCErrors::InvalidInitialDebtAmount);
    }
}

// TODO: consider remove both deposit_collateral and withdraw_stablecoin
pub fn deposit_collateral(
    env: &Env,
    collateral_token: BytesN<32>,
    depositor: &Address,
    collateral_amount: u128,
) {
    token::Client::new(&env, &collateral_token).xfer(
        &depositor,
        &env.current_contract_address(),
        &(collateral_amount as i128),
    );
}

pub fn withdraw_stablecoin(
    env: &Env,
    contract: BytesN<32>,
    recipient: &Address,
    stablecoin_amount: u128,
) {
    token::Client::new(&env, &contract).xfer(
        &env.current_contract_address(),
        &recipient,
        &(stablecoin_amount as i128),
    );
}

pub fn get_protocol_stats(env: &Env) -> ProtStats {
    env.storage()
        .get(&DataKeys::ProtStats)
        .unwrap_or(Ok(ProtStats {
            tot_vaults: 0,
            tot_debt: 0,
            tot_col: 0,
        }))
        .unwrap()
}

pub fn update_protocol_stats(env: &Env, stats: ProtStats) {
    env.storage().set(&DataKeys::ProtStats, &stats);
}
