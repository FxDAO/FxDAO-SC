use soroban_sdk::{contracttype, Address, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct Currency {
    pub denomination: Symbol, // symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    pub active: bool,
    pub contract: Address,
    pub last_update: u64, // This is the last time the price got updated
    pub rate: u128,       // This is the current price of the collateral in our protocol
}

#[contracttype]
pub enum CurrenciesDataKeys {
    Currency(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
}
