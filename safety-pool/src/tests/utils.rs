#![cfg(test)]
use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use crate::token;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, Symbol};

pub fn create_token_contract(e: &Env, admin: &Address) -> token::Client {
    token::Client::new(&e, &e.register_stellar_asset_contract(admin.clone()))
}

pub struct TestData {
    pub contract_admin: Address,
    pub vaults_contract: Address,
    pub treasury_contract: Address,
    pub deposit_asset_admin: Address,
    pub deposit_asset: token::Client,
    pub collateral_asset_admin: Address,
    pub collateral_asset: token::Client,
    pub denomination_asset: Symbol,
    pub min_deposit: u128,
    pub contract_address: Address,
    pub contract_client: SafetyPoolContractClient,
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
        contract_address: Address::from_contract_id(&env, &contract_client.contract_id),
        contract_client,
    }
}

pub fn init_contract(test_data: &TestData) {
    test_data.contract_client.init(
        &test_data.contract_admin,
        &test_data.vaults_contract,
        &test_data.treasury_contract,
        &test_data.collateral_asset.contract_id,
        &test_data.deposit_asset.contract_id,
        &test_data.denomination_asset,
        &test_data.min_deposit,
    );
}
