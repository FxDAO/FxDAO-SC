use crate::storage::deposits::{Deposit, DepositsDataKeys};
use soroban_sdk::{token, vec, Address, Env, Vec};

pub fn make_deposit(env: &Env, asset: &Address, depositor: &Address, amount: &u128) {
    token::Client::new(env, asset).transfer(
        depositor,
        &env.current_contract_address(),
        &(amount.clone() as i128),
    );
}

pub fn make_withdrawal(env: &Env, asset: &Address, deposit: &Deposit) {
    token::Client::new(env, asset).transfer(
        &env.current_contract_address(),
        &deposit.depositor,
        &(deposit.amount as i128),
    );
}

pub fn save_deposit(env: &Env, deposit: &Deposit) {
    env.storage().persistent().set(
        &DepositsDataKeys::Deposit(deposit.depositor.clone()),
        deposit,
    );
}

pub fn get_deposit(env: &Env, depositor: &Address) -> Deposit {
    env.storage()
        .persistent()
        .get(&DepositsDataKeys::Deposit(depositor.clone()))
        .unwrap_or(Deposit {
            depositor: depositor.clone(),
            amount: 0,
            deposit_time: env.ledger().timestamp(),
        })
}

pub fn remove_deposit(env: &Env, depositor: &Address) {
    env.storage()
        .persistent()
        .remove(&DepositsDataKeys::Deposit(depositor.clone()));
}

pub fn save_depositors(env: &Env, depositors: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DepositsDataKeys::Depositors, depositors)
}

pub fn get_depositors(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DepositsDataKeys::Depositors)
        .unwrap_or(vec![&env] as Vec<Address>)
}

pub fn is_depositor_listed(records: &Vec<Address>, depositor: &Address) -> bool {
    let mut saved: bool = false;

    for value in records.iter() {
        if depositor == &value {
            saved = true;
        }
    }
    saved
}

pub fn remove_depositor_from_depositors(
    depositors: &Vec<Address>,
    depositor: &Address,
) -> Vec<Address> {
    let mut updated_record = depositors.clone();

    for (i, address) in updated_record.iter().enumerate() {
        if depositor == &address {
            updated_record.remove(i as u32);
        }
    }

    updated_record
}
