use soroban_sdk::contracttype;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
#[contracttype]
pub struct Liquidation {
    pub index: u64,
    pub total_deposits: u128,
    pub total_debt_paid: u128,
    pub total_col_liquidated: u128,
    pub col_to_withdraw: u128,
    pub share_price: u128,
    pub total_shares: u128,
    pub shares_redeemed: u128,
}

#[contracttype]
pub enum LiquidationsDataKeys {
    Liquidation(u64),
}
