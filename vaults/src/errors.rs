use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCErrors {
    PanicModeEnabled = 20,
    UnexpectedError = 50,
    CoreAlreadySet = 100,
    VaultsInfoHasNotStarted = 200,
    ThereAreNoVaults = 201,
    InvalidMinDebtAmount = 300,
    InvalidMinCollateralAmount = 310,
    InvalidOpeningCollateralRatio = 400,
    VaultDoesntExist = 500,
    UserAlreadyHasDenominationVault = 501,
    UserVaultIndexIsInvalid = 502,
    UserVaultCantBeLiquidated = 503,
    InvalidPrevVaultIndex = 504,
    PrevVaultCantBeNone = 505,
    PrevVaultDoesntExist = 506,
    PrevVaultNextIndexIsLowerThanNewVault = 507,
    PrevVaultNextIndexIsInvalid = 508,
    IndexProvidedIsNotTheOneSaved = 509,
    NextPrevVaultShouldBeNone = 510,
    NotEnoughVaultsToLiquidate = 511,
    InvalidPrevKeyDenomination = 512,
    DepositAmountIsMoreThanTotalDebt = 600,
    CollateralRateUnderMinimum = 700,
    NotEnoughFundsToRedeem = 800,
    CurrencyAlreadyAdded = 900,
    CurrencyDoesntExist = 901,
    CurrencyIsInactive = 902,
}
