use crate::oracle::{Asset, PriceData};
use crate::storage::core::CoreState;
use soroban_sdk::{Env, Symbol};

pub fn get_currency_rate(e: &Env, core_state: &CoreState, denomination: &Symbol) -> PriceData {
    // TODO: if the timestamp is too far away (15 minutes or more), turn on the panic mode so only payment of vaults is available
    crate::oracle::Client::new(&e, &core_state.oracle)
        .lastprice(
            &e.current_contract_address(),
            &Asset::Other(denomination.clone()),
        )
        .unwrap()
}
