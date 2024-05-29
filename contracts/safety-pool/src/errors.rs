use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SCErrors {
    UnexpectedError = 9,

    // Core Errors
    ContractAlreadyInitiated = 10001,

    // Deposits
    BelowMinDeposit = 20001,
    DepositDoesntExist = 20002,
    DepositAlreadyCreated = 20003,
    NothingToWithdraw = 20004,
    LockedPeriodUncompleted = 20005,

    // Liquidations
    CantLiquidateVaults = 30000,
    LiquidationDoesntExist = 31000,
    CantGetMoreThanTenLiquidations = 32000,
    NoCollateralAvailable = 35000,
    CollateralAvailable = 36000,

    // Distributions
    RecentDistribution = 40000,
}
