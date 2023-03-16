use crate::storage_types::*;
use crate::token;
use soroban_sdk::{panic_with_error, Address, BytesN, Env, Symbol};

pub fn check_admin(env: &Env) {
    let admin: Address = env.storage().get(&DataKeys::Admin).unwrap().unwrap();
    admin.require_auth();
}

pub fn get_core_state(env: &Env) -> CoreState {
    env.storage().get(&DataKeys::CoreState).unwrap().unwrap()
}

pub fn get_protocol_state(env: &Env) -> ProtocolState {
    env.storage().get(&DataKeys::ProtState).unwrap().unwrap()
}

pub fn valid_initial_debt(env: &Env, state: &ProtocolState, initial_debt: i128) {
    if state.mn_v_c_amt > initial_debt {
        panic_with_error!(env, SCErrors::InvalidInitialDebtAmount);
    }
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

pub fn check_positive(env: &Env, value: &i128) {
    if value < &0 {
        panic_with_error!(&env, SCErrors::UnsupportedNegativeValue);
    }
}

/// Vaults utils
pub fn validate_user_vault(env: &Env, user: Address, denomination: Symbol) {
    if !env.storage().has(&UserVaultDataType {
        user,
        symbol: denomination,
    }) {
        panic_with_error!(&env, SCErrors::UserVaultDoesntExist);
    }
}

pub fn vault_spot_available(env: &Env, user: Address, denomination: Symbol) {
    if env.storage().has(&UserVaultDataType {
        user,
        symbol: denomination,
    }) {
        panic_with_error!(&env, SCErrors::UserAlreadyHasDenominationVault);
    }
}

pub fn set_user_vault(env: &Env, user: &Address, denomination: &Symbol, user_vault: &UserVault) {
    env.storage().set(
        &UserVaultDataType {
            user: user.clone(),
            symbol: denomination.clone(),
        },
        user_vault,
    );
}

pub fn remove_user_vault(env: &Env, user: &Address, denomination: &Symbol) {
    env.storage().remove(&UserVaultDataType {
        user: user.clone(),
        symbol: denomination.clone(),
    });
}

pub fn get_user_vault(env: &Env, user: Address, denomination: Symbol) -> UserVault {
    env.storage()
        .get(&UserVaultDataType {
            user,
            symbol: denomination,
        })
        .unwrap()
        .unwrap()
}

/// Currency utils
pub fn validate_currency(env: &Env, denomination: Symbol) {
    if !env.storage().has(&DataKeys::Currency(denomination)) {
        panic_with_error!(&env, SCErrors::CurrencyDoesntExist);
    }
}

pub fn is_currency_active(env: &Env, denomination: Symbol) {
    let currency: Currency = env
        .storage()
        .get(&DataKeys::Currency(denomination))
        .unwrap()
        .unwrap();

    if !currency.active {
        panic_with_error!(&env, SCErrors::CurrencyIsInactive);
    }
}

pub fn save_currency(env: &Env, currency: Currency) {
    env.storage()
        .set(&DataKeys::Currency(currency.symbol), &currency);
}

pub fn get_currency(env: &Env, denomination: Symbol) -> Currency {
    env.storage()
        .get(&DataKeys::Currency(denomination))
        .unwrap()
        .unwrap()
}

/// Payments Utils
pub fn deposit_collateral(env: &Env, core_state: &CoreState, depositor: &Address, amount: &i128) {
    token::Client::new(&env, &core_state.colla_tokn).xfer(
        &depositor,
        &env.current_contract_address(),
        &amount,
    );
}

pub fn withdraw_stablecoin(
    env: &Env,
    core_state: &CoreState,
    currency: &Currency,
    recipient: &Address,
    amount: &i128,
) {
    token::Client::new(&env, &currency.contract).xfer_from(
        &env.current_contract_address(),
        &core_state.stble_issr,
        &recipient,
        &amount,
    );
}

pub fn deposit_stablecoin(env: &Env, currency: &Currency, depositor: &Address, amount: &i128) {
    token::Client::new(&env, &currency.contract).xfer(
        &depositor,
        &env.current_contract_address(),
        &amount,
    );
}
