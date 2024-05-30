use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub struct Pool {
    pub active: bool,
    pub asset: Address,
    pub balance: u128,
    pub deposits: u64,
    pub factor: u128,
    pub lock_period: u64,
    pub min_deposit: u128,
}

#[contracttype]
pub enum PoolDataKeys {
    Pool(Address),
}

pub struct Pools {
    pub env: Env,
}

impl Pools {
    #[inline(always)]
    pub fn new(e: &Env) -> Pools {
        Pools { env: e.clone() }
    }

    pub fn pool(&self, address: &Address) -> Option<Pool> {
        self.env
            .storage()
            .persistent()
            .get(&PoolDataKeys::Pool(address.clone()))
    }

    pub fn set_pool(&self, pool: &Pool) {
        self.env
            .storage()
            .persistent()
            .set(&PoolDataKeys::Pool(pool.asset.clone()), pool);
    }

    pub fn bump_pool(&self, address: &Address) {
        self.env.storage().persistent().extend_ttl(
            &PoolDataKeys::Pool(address.clone()),
            17280,
            17280 * 30,
        );
    }
}

pub trait PoolsDataFunc {
    fn _pools(&self) -> Pools;
}

impl PoolsDataFunc for Env {
    #[inline(always)]
    fn _pools(&self) -> Pools {
        Pools::new(self)
    }
}
