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

pub fn get_currency_vault_conditions(env: &Env, denomination: &Symbol) -> CurrencyVaultsConditions {
    env.storage()
        .get(&DataKeys::CyVltCond(denomination.clone()))
        .unwrap()
        .unwrap()
}

pub fn set_currency_vault_conditions(
    env: &Env,
    mn_col_rte: &i128,
    mn_v_c_amt: &i128,
    op_col_rte: &i128,
    denomination: &Symbol,
) {
    env.storage().set(
        &DataKeys::CyVltCond(denomination.clone()),
        &CurrencyVaultsConditions {
            mn_col_rte: mn_col_rte.clone(),
            mn_v_c_amt: mn_v_c_amt.clone(),
            op_col_rte: op_col_rte.clone(),
        },
    );
}

pub fn valid_initial_debt(
    env: &Env,
    currency_vault_conditions: &CurrencyVaultsConditions,
    initial_debt: i128,
) {
    if currency_vault_conditions.mn_v_c_amt > initial_debt {
        panic_with_error!(env, SCErrors::InvalidInitialDebtAmount);
    }
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

/// Currency Stats Utils
pub fn get_currency_stats(env: &Env, denomination: &Symbol) -> CurrencyStats {
    env.storage()
        .get(&DataKeys::CyStats(denomination.clone()))
        .unwrap_or(Ok(CurrencyStats {
            tot_vaults: 0,
            tot_debt: 0,
            tot_col: 0,
        }))
        .unwrap()
}

pub fn set_currency_stats(env: &Env, denomination: &Symbol, currency_stats: &CurrencyStats) {
    env.storage()
        .set(&DataKeys::CyStats(denomination.clone()), currency_stats);
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
