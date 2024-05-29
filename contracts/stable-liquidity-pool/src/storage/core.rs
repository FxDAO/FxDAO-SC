use soroban_sdk::{contracttype, Address, Env, Vec};

const INSTANCE_BUMP_CONSTANT: u32 = 507904;
const INSTANCE_BUMP_CONSTANT_THRESHOLD: u32 = 253952;

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
    fn _bump_instance(&self);
    fn _core_state(&self) -> Option<CoreState>;
    fn _set_core(&self, v: &CoreState);
    fn _locking_state(&self) -> Option<LockingState>;
    fn _set_locking_state(&self, v: &LockingState);
}

impl CoreStorageFunc for Env {
    fn _bump_instance(&self) {
        self.storage().instance().extend_ttl(
            INSTANCE_BUMP_CONSTANT_THRESHOLD,
            self.ledger().sequence() + INSTANCE_BUMP_CONSTANT,
        );
    }

    fn _core_state(&self) -> Option<CoreState> {
        self.storage().instance().get(&CoreStorageKeys::CoreState)
    }

    fn _set_core(&self, v: &CoreState) {
        self.storage()
            .instance()
            .set(&CoreStorageKeys::CoreState, v);
    }

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
