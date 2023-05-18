use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SCErrors {
    // Core Errors
    ContractAlreadyInitiated = 10001,

    // Deposits
    BelowMinDeposit = 20001,
    NothingToWithdraw = 20002,

    // Liquidations
    CantLiquidateVaults = 30000,
}
