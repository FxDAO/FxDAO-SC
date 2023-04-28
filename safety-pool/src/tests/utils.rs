use crate::contract::SafetyPoolContract;
use crate::token;
use crate::SafetyPoolContractClient;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

pub fn create_token_contract(e: &Env, admin: &Address) -> token::Client {
    token::Client::new(&e, &e.register_stellar_asset_contract(admin.clone()))
}

pub struct TestData {
    pub contract_admin: Address,
    pub vaults_contract: Address,
    pub deposit_asset_admin: Address,
    pub deposit_asset: token::Client,
    pub min_deposit: u128,
    pub contract_client: SafetyPoolContractClient,
}

pub fn create_test_data(env: &Env) -> TestData {
    let contract_admin: Address = Address::random(&env);
    let vaults_contract: Address = Address::random(&env);

    let deposit_asset_admin = Address::random(&env);
    let deposit_asset = create_token_contract(&env, &deposit_asset_admin);

    let min_deposit: u128 = 1000000000;

    let contract_client =
        SafetyPoolContractClient::new(&env, &env.register_contract(None, SafetyPoolContract));

    TestData {
        contract_admin,
        vaults_contract,
        deposit_asset_admin,
        deposit_asset,
        min_deposit,
        contract_client,
    }
}

pub fn init_contract(test_data: &TestData) {
    test_data.contract_client.init(
        &test_data.contract_admin,
        &test_data.vaults_contract,
        &test_data.deposit_asset.contract_id,
        &test_data.min_deposit,
    );
}
