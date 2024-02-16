use soroban_sdk::{contracttype, Address, Env};

pub const DAY_IN_LEDGERS: u32 = 17280;
pub const INSTANCE_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 28;
pub const INSTANCE_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 14;

#[contracttype]
pub struct CoreState {
    pub col_token: Address,
    pub stable_issuer: Address,
    pub admin: Address,
    pub protocol_manager: Address,
    pub panic_mode: bool,
    pub treasury: Address,
    pub fee: u128,
    pub oracle: Address,
}

#[contracttype]
pub enum CoreDataKeys {
    CoreState,
}

pub trait CoreFunc {
    fn set_core_state(&self, core_state: &CoreState);
    fn core_state(&self) -> Option<CoreState>;
    fn bump_instance(&self);
}

impl CoreFunc for Env {
    fn set_core_state(&self, core_state: &CoreState) {
        self.storage()
            .instance()
            .set(&CoreDataKeys::CoreState, core_state);
    }

    fn core_state(&self) -> Option<CoreState> {
        self.storage().instance().get(&CoreDataKeys::CoreState)
    }

    fn bump_instance(&self) {
        self.storage().instance().extend_ttl(
            INSTANCE_BUMP_CONSTANT_THRESHOLD,
            self.ledger().sequence() + INSTANCE_BUMP_CONSTANT,
        );
    }
}
