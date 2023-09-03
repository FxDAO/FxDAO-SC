use crate::errors::SCErrors;
use crate::storage::currencies::{CurrenciesDataKeys, Currency};
use soroban_sdk::{panic_with_error, Env, Symbol};

/// Currency utils
pub fn validate_currency(env: &Env, denomination: &Symbol) {
    if !env
        .storage()
        .instance()
        .has(&CurrenciesDataKeys::Currency(denomination.clone()))
    {
        panic_with_error!(&env, &SCErrors::CurrencyDoesntExist);
    }
}

pub fn is_currency_active(env: &Env, denomination: &Symbol) {
    let currency: Currency = env
        .storage()
        .instance()
        .get(&CurrenciesDataKeys::Currency(denomination.clone()))
        .unwrap();

    if !currency.active {
        panic_with_error!(&env, &SCErrors::CurrencyIsInactive);
    }
}

pub fn save_currency(env: &Env, currency: &Currency) {
    env.storage().instance().set(
        &CurrenciesDataKeys::Currency(currency.denomination.clone()),
        currency,
    );
}

pub fn get_currency(env: &Env, denomination: &Symbol) -> Currency {
    env.storage()
        .instance()
        .get(&CurrenciesDataKeys::Currency(denomination.clone()))
        .unwrap()
}
