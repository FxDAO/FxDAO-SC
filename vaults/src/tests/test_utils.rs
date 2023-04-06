#![cfg(test)]
use crate::contract::VaultsContract;
use crate::token;
use crate::VaultsContractClient;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol, Address, Env, Symbol};

pub fn create_token_contract(e: &Env, admin: &Address) -> token::Client {
    token::Client::new(&e, &e.register_stellar_asset_contract(admin.clone()))
}

pub struct TestData {
    // Contract data
    pub contract_admin: Address,
    pub contract_client: VaultsContractClient,

    // Collateral token data
    pub collateral_token_admin: Address,
    pub collateral_token_client: token::Client,

    // Native token data
    // native_token_admin: Address,
    pub native_token_client: token::Client,

    // Stable token data
    pub stable_token_denomination: Symbol,
    pub stable_token_issuer: Address,
    pub stable_token_client: token::Client,
}

pub struct InitialVariables {
    pub currency_price: i128,
    pub depositor: Address,
    pub initial_debt: i128,
    pub collateral_amount: i128,
    pub contract_address: Address,
    pub mn_col_rte: i128,
    pub mn_v_c_amt: i128,
    pub op_col_rte: i128,
}

pub fn create_base_data(env: &Env) -> TestData {
    // Set up the collateral token
    let collateral_token_admin = Address::random(&env);
    let collateral_token_client = create_token_contract(&env, &collateral_token_admin);

    // Set up the native token
    let native_token_admin = Address::random(&env);
    let native_token_client = create_token_contract(&env, &native_token_admin);

    // Set up the stable token
    let stable_token_denomination: Symbol = symbol!("usd");
    let stable_token_issuer = Address::random(&env);
    let stable_token_client = create_token_contract(&env, &stable_token_issuer);

    // Create the contract
    let contract_admin = Address::random(&env);
    let contract_client =
        VaultsContractClient::new(&env, &env.register_contract(None, VaultsContract));

    return TestData {
        contract_admin,
        contract_client,
        collateral_token_admin,
        collateral_token_client,
        // native_token_admin,
        native_token_client,
        stable_token_denomination,
        stable_token_issuer,
        stable_token_client,
    };
}

pub fn create_base_variables(env: &Env, data: &TestData) -> InitialVariables {
    InitialVariables {
        currency_price: 20000000,
        depositor: Address::random(&env),
        initial_debt: 50000000000,
        collateral_amount: 50000000000,
        contract_address: Address::from_contract_id(&env, &data.contract_client.contract_id),
        mn_col_rte: 1_1000000,
        mn_v_c_amt: 5000_0000000,
        op_col_rte: 1_1500000,
    }
}

pub fn set_initial_state(env: &Env, data: &TestData, base_variables: &InitialVariables) {
    data.contract_client.init(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.stable_token_issuer,
    );

    data.contract_client.new_cy(
        &data.stable_token_denomination,
        &data.stable_token_client.contract_id,
    );

    data.contract_client.s_cy_rate(
        &data.stable_token_denomination,
        &base_variables.currency_price,
    );

    data.contract_client
        .toggle_cy(&data.stable_token_denomination, &true);

    data.contract_client.s_c_v_c(
        &base_variables.mn_col_rte,
        &base_variables.mn_v_c_amt,
        &base_variables.op_col_rte,
        &data.stable_token_denomination,
    );

    token::Client::new(&env, &data.stable_token_client.contract_id).incr_allow(
        &data.stable_token_issuer,
        &Address::from_contract_id(&env, &data.contract_client.contract_id),
        &9000000000000000,
    );

    token::Client::new(&env, &data.stable_token_client.contract_id).mint(
        &data.stable_token_issuer,
        &data.stable_token_issuer,
        &90000000000000000000,
    );
}
