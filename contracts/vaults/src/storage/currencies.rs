use soroban_sdk::{contracttype, Address, Env, Symbol};

#[contracttype]
#[derive(Clone)]
pub struct Currency {
    pub denomination: Symbol,
    pub active: bool,
    pub contract: Address,
}

#[contracttype]
pub enum CurrenciesDataKeys {
    Currency(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "USD"
}

pub trait CurrenciesFunc {
    fn currency(&self, denomination: &Symbol) -> Option<Currency>;
    fn set_currency(&self, currency: &Currency);
}

impl CurrenciesFunc for Env {
    fn currency(&self, denomination: &Symbol) -> Option<Currency> {
        self.storage()
            .instance()
            .get(&CurrenciesDataKeys::Currency(denomination.clone()))
    }

    fn set_currency(&self, currency: &Currency) {
        self.storage().instance().set(
            &CurrenciesDataKeys::Currency(currency.denomination.clone()),
            currency,
        );
    }
}
