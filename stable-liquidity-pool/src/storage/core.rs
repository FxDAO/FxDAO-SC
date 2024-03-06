use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
pub struct CoreState {
    pub admin: Address,
    pub manager: Address,
    pub governance_token: Address,
    pub accepted_assets: Vec<Address>,
    // For example 0.3% = 0.003 = 30000
    pub fee_percentage: u128,
    pub total_deposited: u128,
    pub share_price: u128,
    pub total_shares: u128,
    pub treasury: Address,
}

#[contracttype]
pub struct LockingState {
    // This is the total of shares locked
    pub total: u128,

    // The factor is a value used to know the rewards for each user
    pub factor: u128,
}

#[contracttype]
pub enum CoreStorageKeys {
    CoreState,
    LockingState,
}

pub trait CoreStorageFunc {
    fn _locking_state(&self) -> Option<LockingState>;
    fn _set_locking_state(&self, v: &LockingState);
}

impl CoreStorageFunc for Env {
    fn _locking_state(&self) -> Option<LockingState> {
        self.storage()
            .instance()
            .get(&CoreStorageKeys::LockingState)
    }

    fn _set_locking_state(&self, v: &LockingState) {
        self.storage()
            .instance()
            .set(&CoreStorageKeys::LockingState, v);
    }
}
