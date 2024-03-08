use soroban_sdk::{token, Address, Env, Vec};

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

pub fn make_withdrawal(env: &Env, depositor: &Address, asset: &Address, amount: &u128) {
    token::Client::new(env, asset).transfer(
        &env.current_contract_address(),
        depositor,
        &(amount.clone() as i128),
    );
}
