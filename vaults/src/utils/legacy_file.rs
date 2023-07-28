use crate::storage_types::*;
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol};

pub fn check_admin(env: &Env) -> Address {
    let admin: Address = env.storage().instance().get(&DataKeys::Admin).unwrap();
    admin.require_auth();
    admin
}

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

pub fn validate_initial_debt(
    env: &Env,
    currency_vault_conditions: &CurrencyVaultsConditions,
    initial_debt: i128,
) {
    if currency_vault_conditions.min_debt_creation > initial_debt {
        panic_with_error!(env, SCErrors::InvalidInitialDebtAmount);
    }
}

pub fn check_positive(env: &Env, value: &i128) {
    if value < &0 {
        panic_with_error!(&env, SCErrors::UnsupportedNegativeValue);
    }
}

/// Vaults utils
pub fn validate_user_vault(env: &Env, user_vault_data_type: &UserVaultDataType) {
    if !env
        .storage()
        .persistent()
        .has(&VaultsDataKeys::UserVault(user_vault_data_type.clone()))
    {
        panic_with_error!(&env, SCErrors::UserVaultDoesntExist);
    }
}

pub fn vault_spot_available(env: &Env, user: Address, denomination: &Symbol) {
    if env
        .storage()
        .persistent()
        .has(&VaultsDataKeys::UserVault(UserVaultDataType {
            user,
            denomination: denomination.clone(),
        }))
    {
        panic_with_error!(&env, SCErrors::UserAlreadyHasDenominationVault);
    }
}

/// Currency utils
pub fn validate_currency(env: &Env, denomination: &Symbol) {
    if !env
        .storage()
        .instance()
        .has(&DataKeys::Currency(denomination.clone()))
    {
        panic_with_error!(&env, SCErrors::CurrencyDoesntExist);
    }
}

pub fn is_currency_active(env: &Env, denomination: &Symbol) {
    let currency: Currency = env
        .storage()
        .instance()
        .get(&DataKeys::Currency(denomination.clone()))
        .unwrap();

    if !currency.active {
        panic_with_error!(&env, SCErrors::CurrencyIsInactive);
    }
}

pub fn save_currency(env: &Env, currency: &Currency) {
    env.storage()
        .instance()
        .set(&DataKeys::Currency(currency.denomination.clone()), currency);
}

pub fn get_currency(env: &Env, denomination: &Symbol) -> Currency {
    env.storage()
        .instance()
        .get(&DataKeys::Currency(denomination.clone()))
        .unwrap()
}

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

/// Payments Utils
pub fn withdraw_collateral(env: &Env, core_state: &CoreState, requester: &Address, amount: &i128) {
    token::Client::new(&env, &core_state.col_token).transfer(
        &env.current_contract_address(),
        &requester,
        &amount,
    );
}

pub fn deposit_collateral(env: &Env, core_state: &CoreState, depositor: &Address, amount: &i128) {
    token::Client::new(&env, &core_state.col_token).transfer(
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
    token::Client::new(&env, &currency.contract).transfer_from(
        &env.current_contract_address(),
        &core_state.stable_issuer,
        &recipient,
        &amount,
    );
}

pub fn deposit_stablecoin(
    env: &Env,
    core_state: &CoreState,
    currency: &Currency,
    depositor: &Address,
    amount: &i128,
) {
    token::Client::new(&env, &currency.contract).transfer(
        &depositor,
        &core_state.stable_issuer,
        &amount,
    );
}
