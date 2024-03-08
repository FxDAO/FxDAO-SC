use soroban_sdk::{contracttype, Address, Env};

pub const PERSISTENT_BUMP_CONSTANT: u32 = 1036800;
pub const PERSISTENT_BUMP_CONSTANT_THRESHOLD: u32 = 518400;

#[contracttype]
pub struct Deposit {
    pub depositor: Address,
    pub shares: u128,
    pub locked: bool,
    pub unlocks_at: u64,

    // This is the snapshot of the factor at the moment of locking this deposit
    pub snapshot: u128,
}

#[contracttype]
pub enum DepositsDataKeys {
    Deposit(Address),
}

pub trait DepositsStorageFunc {
    fn _bump_deposit(&self, depositor: &Address);
    fn _deposit(&self, depositor: &Address) -> Option<Deposit>;
    fn _set_deposit(&self, v: &Deposit);
    fn _remove_deposit(&self, depositor: &Address);
}

impl DepositsStorageFunc for Env {
    fn _bump_deposit(&self, depositor: &Address) {
        self.storage().persistent().extend_ttl(
            &DepositsDataKeys::Deposit(depositor.clone()),
            PERSISTENT_BUMP_CONSTANT_THRESHOLD,
            self.ledger().sequence() + PERSISTENT_BUMP_CONSTANT,
        );
    }

    fn _deposit(&self, depositor: &Address) -> Option<Deposit> {
        self.storage()
            .persistent()
            .get(&DepositsDataKeys::Deposit(depositor.clone()))
    }

    fn _set_deposit(&self, v: &Deposit) {
        self.storage()
            .persistent()
            .set(&DepositsDataKeys::Deposit(v.depositor.clone()), v);
    }

    fn _remove_deposit(&self, depositor: &Address) {
        self.storage()
            .persistent()
            .remove(&DepositsDataKeys::Deposit(depositor.clone()));
    }
}
