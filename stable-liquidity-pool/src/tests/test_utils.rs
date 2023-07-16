#![cfg(test)]
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, vec, Address, Env, Vec};
use token::Client as TokenClient;

use crate::contract::{StableLiquidityPoolContract, StableLiquidityPoolContractClient};

pub struct TestData<'a> {
    pub stable_liquidity_pool_contract_client: StableLiquidityPoolContractClient<'a>,

    pub admin: Address,
    pub manager: Address,
    pub governance_token: Address,

    pub usdc_token_admin: Address,
    pub usdc_token_client: TokenClient<'a>,

    pub usdt_token_admin: Address,
    pub usdt_token_client: TokenClient<'a>,

    pub usdx_token_admin: Address,
    pub usdx_token_client: TokenClient<'a>,

    pub fee_percentage: u128,
    pub treasury: Address,
    pub minted_asset_amount: u128,
}

pub fn create_test_data(env: &Env) -> TestData {
    let admin = Address::random(&env);
    let manager = Address::random(&env);
    let governance_token = Address::random(&env);

    let usdc_token_admin = Address::random(&env);
    let usdc_token_client = TokenClient::new(
        &env,
        &env.register_stellar_asset_contract(usdc_token_admin.clone()),
    );

    let usdt_token_admin = Address::random(&env);
    let usdt_token_client = TokenClient::new(
        &env,
        &env.register_stellar_asset_contract(usdt_token_admin.clone()),
    );

    let usdx_token_admin = Address::random(&env);
    let usdx_token_client = TokenClient::new(
        &env,
        &env.register_stellar_asset_contract(usdx_token_admin.clone()),
    );

    let fee_percentage = 30000;
    let treasury = Address::random(&env);

    TestData {
        stable_liquidity_pool_contract_client: StableLiquidityPoolContractClient::new(
            &env,
            &env.register_contract(None, StableLiquidityPoolContract),
        ),
        admin,
        manager,
        governance_token,
        usdc_token_admin,
        usdc_token_client,
        usdt_token_admin,
        usdt_token_client,
        usdx_token_admin,
        usdx_token_client,
        fee_percentage,
        treasury,
        minted_asset_amount: 10_000_0000000,
    }
}

pub fn init_contract(env: &Env, test_data: &TestData) {
    test_data.stable_liquidity_pool_contract_client.init(
        &test_data.admin,
        &test_data.manager,
        &test_data.governance_token,
        &(vec![
            &env,
            test_data.usdc_token_client.address.clone(),
            test_data.usdt_token_client.address.clone(),
            test_data.usdx_token_client.address.clone(),
        ] as Vec<Address>),
        &test_data.fee_percentage,
        &test_data.treasury,
    );
}

pub fn prepare_test_accounts(test_data: &TestData, accounts: &Vec<Address>) {
    for item in accounts.iter() {
        let account = item.unwrap();
        test_data
            .usdc_token_client
            .mint(&account, &(test_data.minted_asset_amount as i128));
        test_data
            .usdt_token_client
            .mint(&account, &(test_data.minted_asset_amount as i128));
        test_data
            .usdx_token_client
            .mint(&account, &(test_data.minted_asset_amount as i128));
    }
}
