#![cfg(test)]

use crate::contract::{LockingPoolContract, LockingPoolContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env};

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(e, &contract_address),
        token::StellarAssetClient::new(e, &contract_address),
    )
}

pub struct TestData<'a> {
    pub admin: Address,
    pub manager: Address,
    pub contract_client: LockingPoolContractClient<'a>,

    pub lock_period: u64,
    pub min_deposit: u128,

    pub staking_asset_admin: Address,
    pub staking_asset_client: token::Client<'a>,
    pub staking_asset_stellar: token::StellarAssetClient<'a>,

    pub rewards_asset_admin: Address,
    pub rewards_asset_client: token::Client<'a>,
    pub rewards_asset_stellar: token::StellarAssetClient<'a>,
}

pub fn create_test_data<'a>(e: &Env) -> TestData<'a> {
    let admin: Address = Address::generate(&e);
    let manager: Address = Address::generate(&e);

    let contract_id: Address = e.register_contract(None, LockingPoolContract);
    let contract_client: LockingPoolContractClient<'a> =
        LockingPoolContractClient::new(&e, &contract_id);

    let lock_period: u64 = 3600 * 24 * 7;
    let min_deposit: u128 = 100_0000000;

    let staking_asset_admin: Address = Address::generate(&e);
    let (staking_asset_client, staking_asset_stellar) =
        create_token_contract(&e, &staking_asset_admin);

    let rewards_asset_admin: Address = Address::generate(&e);
    let (rewards_asset_client, rewards_asset_stellar) =
        create_token_contract(&e, &rewards_asset_admin);

    TestData {
        admin,
        manager,
        contract_client,
        lock_period,
        min_deposit,
        staking_asset_admin,
        staking_asset_client,
        staking_asset_stellar,
        rewards_asset_admin,
        rewards_asset_client,
        rewards_asset_stellar,
    }
}

pub fn init_contract(test_data: &TestData) {
    test_data.contract_client.set_admin(&test_data.admin);
    test_data.contract_client.set_manager(&test_data.manager);
    test_data
        .contract_client
        .set_rewards_asset(&test_data.rewards_asset_client.address);
}
