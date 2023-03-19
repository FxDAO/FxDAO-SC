use soroban_sdk::{contracterror, contracttype, Address, BytesN, Symbol};

#[contracttype]
pub struct CoreState {
    pub colla_tokn: BytesN<32>,
    pub stble_issr: Address,
}

#[contracttype]
pub struct UserVaultDataType {
    pub user: Address,
    pub symbol: Symbol, // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
}

#[contracttype]
pub struct UserVault {
    pub id: Address,
    pub total_debt: i128,
    pub total_col: i128,
}

#[contracttype]
pub struct Currency {
    pub symbol: Symbol, // symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    pub active: bool,
    pub contract: BytesN<32>,
    pub last_updte: u64, // This is the last time the price got updated
    pub rate: i128,      // This is the current price of the collateral in our protocol
}

#[contracttype]
pub struct CurrencyStats {
    pub tot_vaults: i64,
    pub tot_debt: i128,
    pub tot_col: i128,
}

#[contracttype]
pub struct CurrencyVaultsConditions {
    pub mn_col_rte: i128, // Min collateral ratio - ex: 1.10
    pub mn_v_c_amt: i128, // Min vault creation amount - ex: 5000
    pub op_col_rte: i128, // Opening collateral ratio - ex: 1.15
}

#[contracttype]
pub enum DataKeys {
    CoreState,
    Admin,
    UserVault(UserVaultDataType),
    Currency(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    CyStats(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    CyVltCond(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
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
    DepositAmountIsMoreThanTotalDebt = 6,
    CollateralRateUnderMinimum = 7,
    UnsupportedNegativeValue = 8,
    CurrencyAlreadyAdded = 90000,
    CurrencyDoesntExist = 90001,
    CurrencyIsInactive = 90002,
}
