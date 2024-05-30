use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum CoreDataKeys {
    Admin,
    Manager,
    RewardsAsset,
}

pub struct Core {
    pub env: Env,
}

impl Core {
    #[inline(always)]
    pub fn new(e: &Env) -> Core {
        Core { env: e.clone() }
    }

    pub fn address(&self, key: &CoreDataKeys) -> Option<Address> {
        self.env.storage().instance().get(key)
    }

    pub fn set_address(&self, key: &CoreDataKeys, address: &Address) {
        self.env.storage().instance().set(key, address);
    }

    pub fn bump(&self) {
        self.env.storage().instance().extend_ttl(17280, 17280 * 30);
    }
}

pub trait CoreStorageFunc {
    fn _core(&self) -> Core;
}

impl CoreStorageFunc for Env {
    #[inline(always)]
    fn _core(&self) -> Core {
        Core::new(self)
    }
}
