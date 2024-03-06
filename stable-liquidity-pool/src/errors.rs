use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SCErrors {
    // Core Errors
    ContractAlreadyInitiated = 11,

    // Deposits
    InvalidAsset = 21,

    // Withdraws
    NothingToWithdraw = 31,
    LockedPeriodUncompleted = 32,
    NotEnoughSharesToWithdraw = 33,
    InvalidWithdraw = 34,

    // Locking
    AlreadyLocked = 41,
    LockedDeposit = 42,
    CantDistribute = 43,
    NotLockedDeposit = 44,
}
