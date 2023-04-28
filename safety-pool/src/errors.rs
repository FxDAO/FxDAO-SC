use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SCErrors {
    // Core Errors
    ContractAlreadyInitiated = 00001,

    // Deposits
    BelowMinDeposit = 10001,
}
