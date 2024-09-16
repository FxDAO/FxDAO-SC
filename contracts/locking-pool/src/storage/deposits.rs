use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub struct Deposit {
    pub amount: u128,
    pub unlocks_at: u64,
    pub snapshot: u128,
}

#[contracttype]
pub enum DepositsStorageKeys {
    Deposit((Address, Address)),
}

pub struct Deposits {
    pub env: Env,
}

impl Deposits {
    #[inline(always)]
    fn new(e: &Env) -> Deposits {
        Deposits { env: e.clone() }
    }

    pub fn get(&self, deposit_address: &Address, depositor: &Address) -> Option<Deposit> {
        self.env
            .storage()
            .persistent()
            .get(&DepositsStorageKeys::Deposit((
                deposit_address.clone(),
                depositor.clone(),
            )))
    }

    pub fn set(&self, deposit_address: &Address, depositor: &Address, deposit: &Deposit) {
        self.env.storage().persistent().set(
            &DepositsStorageKeys::Deposit((deposit_address.clone(), depositor.clone())),
            deposit,
        );
    }

    pub fn bump(&self, deposit_address: &Address, depositor: &Address) {
        self.env.storage().persistent().extend_ttl(
            &DepositsStorageKeys::Deposit((deposit_address.clone(), depositor.clone())),
            17280,
            17280 * 30,
        );
    }

    pub fn remove(&self, deposit_address: &Address, depositor: &Address) {
        self.env
            .storage()
            .persistent()
            .remove(&DepositsStorageKeys::Deposit((
                deposit_address.clone(),
                depositor.clone(),
            )))
    }
}

pub trait DepositsStorageFunc {
    fn _deposits(&self) -> Deposits;
}

impl DepositsStorageFunc for Env {
    fn _deposits(&self) -> Deposits {
        Deposits::new(self)
    }
}
