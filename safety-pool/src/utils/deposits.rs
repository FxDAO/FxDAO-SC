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
    env.storage().set(
        &DepositsDataKeys::Deposit(deposit.depositor.clone()),
        deposit,
    );
}

pub fn get_deposit(env: &Env, depositor: &Address) -> Deposit {
    env.storage()
        .get(&DepositsDataKeys::Deposit(depositor.clone()))
        .unwrap_or(Ok(Deposit {
            depositor: depositor.clone(),
            amount: 0,
            deposit_time: env.ledger().timestamp(),
        }))
        .unwrap()
}

pub fn remove_deposit(env: &Env, depositor: &Address) {
    env.storage()
        .remove(&DepositsDataKeys::Deposit(depositor.clone()));
}

pub fn save_depositors(env: &Env, depositors: &Vec<Address>) {
    env.storage().set(&DepositsDataKeys::Depositors, depositors)
}

pub fn get_depositors(env: &Env) -> Vec<Address> {
    env.storage()
        .get(&DepositsDataKeys::Depositors)
        .unwrap_or(Ok(vec![&env] as Vec<Address>))
        .unwrap()
}

pub fn is_depositor_listed(records: &Vec<Address>, depositor: &Address) -> bool {
    let mut saved: bool = false;

    for item in records.iter() {
        match item {
            Ok(value) => {
                if depositor == &value {
                    saved = true;
                }
            }
            Err(_) => {}
        }
    }
    saved
}

pub fn remove_depositor_from_depositors(
    depositors: &Vec<Address>,
    depositor: &Address,
) -> Vec<Address> {
    let mut updated_record = depositors.clone();

    for (i, el) in updated_record.iter().enumerate() {
        if depositor == &el.unwrap() {
            updated_record.remove(i as u32);
        }
    }

    updated_record
}
