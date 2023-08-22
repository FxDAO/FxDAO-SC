use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SCErrors {
    // Core Errors
    ContractAlreadyInitiated = 10001,

    // Deposits
    InvalidAsset = 20001,

    // Withdraws
    NothingToWithdraw = 30001,
    LockedPeriodUncompleted = 30002,
    NotEnoughSharesToWithdraw = 30003,
    InvalidWithdraw = 30004,

    // Distributions
    RecentDistribution = 40001,
}
