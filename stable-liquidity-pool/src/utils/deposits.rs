use crate::storage::deposits::{Deposit, DepositsDataKeys};
use soroban_sdk::{token, Address, Env, Vec};

pub const PERSISTENT_BUMP_CONSTANT: u32 = 1036800;
pub const PERSISTENT_BUMP_CONSTANT_THRESHOLD: u32 = 518400;

pub fn bump_deposit(env: &Env, depositor: Address) {
    env.storage().persistent().extend_ttl(
        &DepositsDataKeys::Deposit(depositor),
        PERSISTENT_BUMP_CONSTANT_THRESHOLD,
        PERSISTENT_BUMP_CONSTANT,
    );
}

pub fn validate_deposit_asset(accepted_assets: &Vec<Address>, asset: &Address) -> bool {
    for accepted_asset in accepted_assets.iter() {
        if &accepted_asset == asset {
            return true;
        }
    }

    false
}

pub fn make_deposit(env: &Env, depositor: &Address, asset: &Address, amount: &u128) {
    token::Client::new(env, asset).transfer(
        depositor,
        &env.current_contract_address(),
        &(amount.clone() as i128),
    );
}

pub fn has_deposit(env: &Env, depositor: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DepositsDataKeys::Deposit(depositor.clone()))
}

pub fn get_deposit(env: &Env, depositor: &Address) -> Deposit {
    env.storage()
        .persistent()
        .get(&DepositsDataKeys::Deposit(depositor.clone()))
        .unwrap_or(Deposit {
            depositor: depositor.clone(),
            shares: 0,
            locked: false,
            unlocks_at: 0,
            snapshot: 0,
        })
}

pub fn save_deposit(env: &Env, deposit: &Deposit) {
    env.storage().persistent().set(
        &DepositsDataKeys::Deposit(deposit.depositor.clone()),
        deposit,
    );
}

pub fn remove_deposit(env: &Env, depositor: &Address) {
    env.storage()
        .persistent()
        .remove(&DepositsDataKeys::Deposit(depositor.clone()));
}

pub fn make_withdrawal(env: &Env, depositor: &Address, asset: &Address, amount: &u128) {
    token::Client::new(env, asset).transfer(
        &env.current_contract_address(),
        depositor,
        &(amount.clone() as i128),
    );
}
