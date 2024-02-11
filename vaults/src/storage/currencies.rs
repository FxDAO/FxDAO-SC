use soroban_sdk::{contracttype, Address, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct Currency {
    pub denomination: Symbol, // symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    pub active: bool,
    pub contract: Address,
}

#[contracttype]
pub enum CurrenciesDataKeys {
    Currency(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
}
