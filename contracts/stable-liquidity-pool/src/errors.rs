use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SCErrors {
    // Core Errors
    ContractAlreadyInitiated = 11,
    InvalidFee = 12,

    // Deposits
    InvalidAsset = 21,
    MissingDeposit = 22,
    InvalidDepositAmount = 23,

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
