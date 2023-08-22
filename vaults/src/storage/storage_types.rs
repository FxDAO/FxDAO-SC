use soroban_sdk::{contracterror, contracttype, Address, Symbol};

#[contracttype]
pub struct CoreState {
    pub col_token: Address,
    pub stable_issuer: Address,
}

#[contracttype]
pub struct Currency {
    pub denomination: Symbol, // symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    pub active: bool,
    pub contract: Address,
    pub last_updte: u64, // This is the last time the price got updated
    pub rate: i128,      // This is the current price of the collateral in our protocol
}

#[contracttype]
pub struct CurrencyStats {
    pub total_vaults: i64,
    pub total_debt: i128,
    pub total_col: i128,
}

#[contracttype]
pub struct CurrencyVaultsConditions {
    pub min_col_rate: i128,      // Min collateral ratio - ex: 1.10
    pub min_debt_creation: i128, // Min vault creation amount - ex: 5000
    pub opening_col_rate: i128,  // Opening collateral ratio - ex: 1.15
}

#[contracttype]
pub enum DataKeys {
    CoreState,
    Admin,
    OracleAdmin,
    ProtocolManager,
    Currency(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    CurrencyStats(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    CurrencyVaultsConditions(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    PanicMode,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCErrors {
    AlreadyInit = 0,
    Unauthorized = 1,
    UserAlreadyHasVault = 2,
    InvalidInitialDebtAmount = 3,
    InvalidOpeningCollateralRatio = 4,
    UserVaultDoesntExist = 50000,
    UserAlreadyHasDenominationVault = 50001,
    UserVaultIndexIsInvalid = 50002,
    UserVaultCantBeLiquidated = 50003,
    DepositAmountIsMoreThanTotalDebt = 6,
    CollateralRateUnderMinimum = 7,
    UnsupportedNegativeValue = 8,
    CurrencyAlreadyAdded = 90000,
    CurrencyDoesntExist = 90001,
    CurrencyIsInactive = 90002,
}
