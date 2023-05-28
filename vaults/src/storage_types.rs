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
    Currency(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    CurrencyStats(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    CurrencyVaultConditions(Symbol), // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
    PanicMode,
}

#[derive(Clone)]
#[contracttype]
pub struct UserVaultDataType {
    pub user: Address,
    pub denomination: Symbol, // Symbol is the denomination, not the asset code. For example for xUSD the symbol should be "usd"
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct UserVault {
    pub id: Address,
    pub total_debt: i128,
    pub total_col: i128,
    pub index: i128,
    pub denomination: Symbol,
}

#[derive(Clone)]
#[contracttype]
pub struct VaultsWithIndexDataType {
    pub index: i128,
    pub denomination: Symbol,
}

/// I need to be able to check who is the lowest collateral ratio no matter the currency
/// I need to be able to check the lowest one without needing to load a huge vector of values
/// I need to be able to sort the vec from lower to higher in an efficient way
#[contracttype]
pub enum VaultsDataKeys {
    /// The "UserVault" key is the one that actually holds the information of the user's vault
    /// Everytime this key is updated we need to update both "SortedVlts" and "RatioKey"
    UserVault(UserVaultDataType),
    /// This key host a Vec of i128 which is the index of the vaults, this Vec must be updated every time a Vault is updated
    /// The Vec is sorted by the collateral ratio of the deposit IE the lower go first
    /// The Symbol value is the denomination of the currency
    Indexes(Symbol),

    /// The result is a Vec<UserVaultDataType>
    VaultsWithIndex(VaultsWithIndexDataType),
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
