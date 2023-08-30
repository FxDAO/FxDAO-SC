use crate::storage::storage_types::*;
use crate::storage::vaults::*;
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol};

pub fn check_oracle_admin(env: &Env) -> Address {
    let oracle_admin: Address = env
        .storage()
        .instance()
        .get(&DataKeys::OracleAdmin)
        .unwrap();
    oracle_admin.require_auth();
    oracle_admin
}

pub fn check_protocol_manager(env: &Env) -> Address {
    let protocol_manager: Address = env
        .storage()
        .instance()
        .get(&DataKeys::ProtocolManager)
        .unwrap();
    protocol_manager.require_auth();
    protocol_manager
}

pub fn check_positive(env: &Env, value: &i128) {
    if value < &0 {
        panic_with_error!(&env, SCErrors::UnsupportedNegativeValue);
    }
}

/// Vaults utils

/// Currency Vault conditions
pub fn get_currency_vault_conditions(env: &Env, denomination: &Symbol) -> CurrencyVaultsConditions {
    env.storage()
        .instance()
        .get(&DataKeys::CurrencyVaultsConditions(denomination.clone()))
        .unwrap()
}

pub fn set_currency_vault_conditions(
    env: &Env,
    min_col_rate: &i128,
    min_debt_creation: &i128,
    opening_col_rate: &i128,
    denomination: &Symbol,
) {
    env.storage().instance().set(
        &DataKeys::CurrencyVaultsConditions(denomination.clone()),
        &CurrencyVaultsConditions {
            min_col_rate: min_col_rate.clone(),
            min_debt_creation: min_debt_creation.clone(),
            opening_col_rate: opening_col_rate.clone(),
        },
    );
}

/// Currency Stats Utils
pub fn get_currency_stats(env: &Env, denomination: &Symbol) -> CurrencyStats {
    env.storage()
        .instance()
        .get(&DataKeys::CurrencyStats(denomination.clone()))
        .unwrap_or(CurrencyStats {
            total_vaults: 0,
            total_debt: 0,
            total_col: 0,
        })
}

pub fn set_currency_stats(env: &Env, denomination: &Symbol, currency_stats: &CurrencyStats) {
    env.storage().instance().set(
        &DataKeys::CurrencyStats(denomination.clone()),
        currency_stats,
    );
}
