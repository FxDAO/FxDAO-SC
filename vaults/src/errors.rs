use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCErrors {
    CoreAlreadySet = 10000,
    // Unauthorized = 1,
    // UserAlreadyHasVault = 2,
    // InvalidInitialDebtAmount = 3,
    // InvalidOpeningCollateralRatio = 4,
    // UserVaultDoesntExist = 50000,
    // UserAlreadyHasDenominationVault = 50001,
    // UserVaultIndexIsInvalid = 50002,
    // UserVaultCantBeLiquidated = 50003,
    // DepositAmountIsMoreThanTotalDebt = 6,
    // CollateralRateUnderMinimum = 7,
    // UnsupportedNegativeValue = 8,
    CurrencyAlreadyAdded = 90000,
    CurrencyDoesntExist = 90001,
    CurrencyIsInactive = 90002,
}
