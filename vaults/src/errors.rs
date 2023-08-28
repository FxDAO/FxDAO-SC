use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCErrors {
    CoreAlreadySet = 10000,
    // Unauthorized = 10000,
    VaultsInfoHasNotStarted = 20000,
    ThereAreNoVaults = 20001,
    InvalidInitialDebtAmount = 30000,
    InvalidOpeningCollateralRatio = 40000,
    VaultDoesntExist = 50000,
    UserAlreadyHasDenominationVault = 50001,
    UserVaultIndexIsInvalid = 50002,
    UserVaultCantBeLiquidated = 50003,
    InvalidPrevVaultIndex = 50004,
    PrevVaultCantBeNone = 50005,
    PrevVaultDoesntExist = 50006,
    PrevVaultNextIndexIsLowerThanNewVault = 50007,
    PrevVaultNextIndexIsInvalid = 1,
    // DepositAmountIsMoreThanTotalDebt = 60000,
    // CollateralRateUnderMinimum = 70000,
    // UnsupportedNegativeValue = 80000,
    CurrencyAlreadyAdded = 90000,
    CurrencyDoesntExist = 90001,
    CurrencyIsInactive = 90002,
}
