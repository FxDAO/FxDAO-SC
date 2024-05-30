use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractErrors {
    NotStarted = 0,
    PoolDoesntExist = 1,
    InvalidDepositAmount = 2,
    DepositAlreadyExists = 3,
    FundsDepositFailed = 4,
    DepositDoesntExist = 5,
    DepositIsStillLocked = 6,
    RewardsWithdrawFailed = 7,
    FundsWithdrawFailed = 8,
    CantDistributeReward = 9,
    RewardsDepositFailed = 10,
    PoolDoesntAcceptDeposits = 11,
}
