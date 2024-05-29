use crate::oracle::{Asset, PriceData};
use crate::storage::core::CoreState;
use soroban_sdk::{Env, Symbol};

pub fn get_currency_rate(e: &Env, core_state: &CoreState, denomination: &Symbol) -> PriceData {
    crate::oracle::Client::new(&e, &core_state.oracle)
        .lastprice(
            &e.current_contract_address(),
            &Asset::Other(denomination.clone()),
        )
        .unwrap()
}
