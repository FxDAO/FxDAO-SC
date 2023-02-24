use soroban_sdk::{contracterror, contracttype, Address, BytesN};

pub const DEFAULT_DECIMAL: u32 = 7;

#[contracttype]
pub struct CoreState {
    pub colla_tokn: BytesN<32>,
    pub nativ_tokn: BytesN<32>,
    pub stble_tokn: BytesN<32>,
}

pub const DEFAULT_MIN_COLLATERAL_RATIO: i128 = 11000000;
pub const DEFAULT_MIN_VAULT_CREATION_AMOUNT: i128 = 50000000000;
pub const DEFAULT_OPENING_COLLATERAL_RATIO: i128 = 11500000;

#[contracttype]
pub struct ProtocolState {
    // Min collateral ratio - ex: 1.10
    pub mn_col_rte: i128,

    // Min vault creation amount - ex: 5000
    pub mn_v_c_amt: i128,

    // Opening collateral ratio - ex: 1.15
    pub op_col_rte: i128,
}

#[contracttype]
pub struct ProtocolCollateralPrice {
    // This is the last time the price got updated
    pub last_updte: u64,

    // This is the current price of the collateral in our protocol
    pub current: i128,
}

#[contracttype]
pub struct ProtStats {
    pub tot_vaults: i64,
    pub tot_debt: i128,
    pub tot_col: i128,
}

#[contracttype]
pub struct UserVault {
    pub id: Address,
    pub total_debt: i128,
    pub total_col: i128,
}

#[contracttype]
pub enum DataKeys {
    CoreState,
    ProtState,
    ProtRate,
    ProtStats,
    Admin,
    UserVault(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCErrors {
    AlreadyInit = 0,
    Unauthorized = 1,
    UserAlreadyHasVault = 2,
    InvalidInitialDebtAmount = 3,
    InvalidOpeningCollateralRatio = 4,
    UserDoesntHaveAVault = 5,
    DepositAmountIsMoreThanTotalDebt = 6,
    CollateralRateUnderMinimun = 7,
    UnsuportedNegativeValue = 8,
}
