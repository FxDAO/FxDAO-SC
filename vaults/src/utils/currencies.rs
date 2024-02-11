use crate::errors::SCErrors;
use crate::oracle::{Asset, PriceData};
use crate::storage::core::CoreState;
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

pub fn get_currency_rate(e: &Env, core_state: &CoreState, denomination: &Symbol) -> PriceData {
    // TODO: if the timestamp is too far away (15 minutes or more), turn on the panic mode so only payment of vaults is available
    crate::oracle::Client::new(&e, &core_state.oracle)
        .lastprice(
            &e.current_contract_address(),
            &Asset::Other(denomination.clone()),
        )
        .unwrap()
}
