#![cfg(test)]
use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use crate::oracle;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, token, vec, Address, Env, Symbol, Vec};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

use crate::oracle::{
    Asset, AssetsData, Client as OracleClient, CoreData, CustomerQuota, PriceData,
};

pub fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(&e, &contract_address),
        TokenAdminClient::new(&e, &contract_address),
    )
}

pub struct TestData<'a> {
    pub contract_admin: Address,
    pub vaults_contract: Address,
    pub treasury_contract: Address,
    pub deposit_asset_admin: Address,
    pub deposit_asset_client: TokenClient<'a>,
    pub deposit_asset_client_admin: TokenAdminClient<'a>,
    pub collateral_asset_admin: Address,
    pub collateral_asset_client: TokenClient<'a>,
    pub collateral_asset_client_admin: TokenAdminClient<'a>,
    pub denomination_asset: Symbol,
    pub min_deposit: u128,
    pub contract_client: SafetyPoolContractClient<'a>,
    pub profit_share: Vec<u32>,
    pub liquidator_share: Vec<u32>,
    pub governance_asset_client: TokenClient<'a>,
    pub governance_asset_client_admin: TokenAdminClient<'a>,

    pub oracle: Address,
    pub oracle_contract_client: OracleClient<'a>,
    pub oracle_contract_admin: Address,
}

pub fn create_test_data(env: &Env) -> TestData {
    let contract_admin: Address = Address::generate(&env);
    let vaults_contract: Address = Address::generate(&env);
    let treasury_contract: Address = Address::generate(&env);

    let deposit_asset_admin = Address::generate(&env);
    let (deposit_asset_client, deposit_asset_client_admin) =
        create_token_contract(&env, &deposit_asset_admin);

    let collateral_asset_admin = Address::generate(&env);
    let (collateral_asset_client, collateral_asset_client_admin) =
        create_token_contract(&env, &deposit_asset_admin);

    let governance_asset_admin = Address::generate(&env);
    let (governance_asset_client, governance_asset_client_admin) =
        create_token_contract(&env, &governance_asset_admin);

    let min_deposit: u128 = 1000000000;

    let contract_client =
        SafetyPoolContractClient::new(&env, &env.register_contract(None, SafetyPoolContract));

    // Oracle contract
    let oracle: Address = env.register_contract_wasm(None, oracle::WASM);
    let oracle_contract_client: OracleClient = OracleClient::new(&env, &oracle);
    let oracle_contract_admin: Address = Address::generate(&env);

    TestData {
        contract_admin,
        vaults_contract,
        treasury_contract,
        deposit_asset_admin,
        deposit_asset_client,
        deposit_asset_client_admin,
        collateral_asset_admin,
        collateral_asset_client,
        collateral_asset_client_admin,
        denomination_asset: symbol_short!("usd"),
        min_deposit,
        contract_client,
        profit_share: Vec::from_array(&env, [1u32, 2u32]),
        liquidator_share: Vec::from_array(&env, [1u32, 2u32]),
        governance_asset_client,
        governance_asset_client_admin,

        oracle,
        oracle_contract_client,
        oracle_contract_admin,
    }
}

pub fn init_contract(test_data: &TestData) {
    test_data.contract_client.init(
        &test_data.contract_admin,
        &test_data.vaults_contract,
        &test_data.treasury_contract,
        &test_data.collateral_asset_client.address,
        &test_data.deposit_asset_client.address,
        &test_data.denomination_asset,
        &test_data.min_deposit,
        &test_data.governance_asset_client.address,
        &test_data.oracle,
    );
}
