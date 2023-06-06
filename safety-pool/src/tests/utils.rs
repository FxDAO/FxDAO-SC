#![cfg(test)]
use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, vec, Address, Env, Symbol, Vec};
use token::Client as TokenClient;

pub fn create_token_contract<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    token::Client::new(&e, &e.register_stellar_asset_contract(admin.clone()))
}

pub struct TestData<'a> {
    pub contract_admin: Address,
    pub vaults_contract: Address,
    pub treasury_contract: Address,
    pub deposit_asset_admin: Address,
    pub deposit_asset: TokenClient<'a>,
    pub collateral_asset_admin: Address,
    pub collateral_asset: TokenClient<'a>,
    pub denomination_asset: Symbol,
    pub min_deposit: u128,
    pub contract_client: SafetyPoolContractClient<'a>,
    pub profit_share: Vec<u32>,
    pub liquidator_share: Vec<u32>,
}

pub fn create_test_data(env: &Env) -> TestData {
    let contract_admin: Address = Address::random(&env);
    let vaults_contract: Address = Address::random(&env);
    let treasury_contract: Address = Address::random(&env);

    let deposit_asset_admin = Address::random(&env);
    let deposit_asset = create_token_contract(&env, &deposit_asset_admin);

    let collateral_asset_admin = Address::random(&env);
    let collateral_asset = create_token_contract(&env, &deposit_asset_admin);

    let min_deposit: u128 = 1000000000;

    let contract_client =
        SafetyPoolContractClient::new(&env, &env.register_contract(None, SafetyPoolContract));

    TestData {
        contract_admin,
        vaults_contract,
        treasury_contract,
        deposit_asset_admin,
        deposit_asset,
        collateral_asset_admin,
        collateral_asset,
        denomination_asset: Symbol::short("usd"),
        min_deposit,
        contract_client,
        profit_share: vec![&env, 1u32, 2u32] as Vec<u32>,
        liquidator_share: vec![&env, 1u32, 2u32] as Vec<u32>,
    }
}

pub fn init_contract(test_data: &TestData) {
    test_data.contract_client.init(
        &test_data.contract_admin,
        &test_data.vaults_contract,
        &test_data.treasury_contract,
        &test_data.collateral_asset.address,
        &test_data.deposit_asset.address,
        &test_data.denomination_asset,
        &test_data.min_deposit,
        &test_data.profit_share,
        &test_data.liquidator_share,
    );
}

pub fn set_allowance(env: &Env, assets: &Vec<Address>, contract: &Address, depositor: &Address) {
    for item in assets.iter() {
        let asset = item.unwrap();
        token::Client::new(&env, &asset).increase_allowance(
            &depositor,
            &contract,
            &9000000000000000,
        );
    }
}
