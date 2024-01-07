#![cfg(test)]
use crate::contract::VaultsContract;
use crate::utils::payments::calc_fee;
use crate::VaultsContractClient;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, token, Address, Env, IntoVal, Symbol};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(e, &contract_address),
        TokenAdminClient::new(e, &contract_address),
    )
}

pub struct TestData<'a> {
    // Contract data
    pub contract_admin: Address,
    pub oracle_admin: Address,
    pub protocol_manager: Address,
    pub contract_client: VaultsContractClient<'a>,
    pub treasury: Address,
    pub fee: u128,

    // Collateral token data
    pub collateral_token_admin: Address,
    pub collateral_token_client: TokenClient<'a>,
    pub collateral_token_admin_client: TokenAdminClient<'a>,

    // Native token data
    // native_token_admin: Address,
    pub native_token_client: TokenClient<'a>,
    pub native_token_admin_client: TokenAdminClient<'a>,

    // Stable token data
    pub stable_token_denomination: Symbol,
    pub stable_token_issuer: Address,
    pub stable_token_client: TokenClient<'a>,
    pub stable_token_admin_client: TokenAdminClient<'a>,
}

pub struct InitialVariables {
    pub currency_price: u128,
    pub depositor: Address,
    pub initial_debt: u128,
    pub collateral_amount: u128,
    pub collateral_amount_minus_fee: u128,
    pub contract_address: Address,
    pub min_col_rate: u128,
    pub min_debt_creation: u128,
    pub opening_col_rate: u128,
}

pub fn create_base_data(env: &Env) -> TestData {
    env.mock_all_auths();

    // Set up the collateral token
    let collateral_token_admin = Address::generate(&env);
    let (collateral_token_client, collateral_token_admin_client) =
        create_token_contract(&env, &collateral_token_admin);

    // Set up the native token
    let native_token_admin = Address::generate(&env);
    let (native_token_client, native_token_admin_client) =
        create_token_contract(&env, &native_token_admin);

    // Set up the stable token
    let stable_token_denomination: Symbol = symbol_short!("usd");
    let stable_token_issuer = Address::generate(&env);
    let (stable_token_client, stable_token_admin_client) =
        create_token_contract(&env, &stable_token_issuer);

    // Create the contract
    let contract_admin = Address::generate(&env);
    let oracle_admin = Address::generate(&env);
    let protocol_manager = Address::generate(&env);
    let contract_client =
        VaultsContractClient::new(&env, &env.register_contract(None, VaultsContract));

    return TestData {
        contract_admin,
        oracle_admin,
        protocol_manager,
        contract_client,
        treasury: Address::generate(&env),
        fee: 50000,
        collateral_token_admin,
        collateral_token_client,
        collateral_token_admin_client,
        // native_token_admin,
        native_token_client,
        native_token_admin_client,
        stable_token_denomination,
        stable_token_issuer,
        stable_token_client,
        stable_token_admin_client,
    };
}

pub fn create_base_variables(env: &Env, data: &TestData) -> InitialVariables {
    InitialVariables {
        currency_price: 830124,
        depositor: Address::generate(&env),
        initial_debt: 5000_0000000,
        collateral_amount: 90_347_8867088,
        collateral_amount_minus_fee: 90_347_8867088 - calc_fee(&data.fee, &90_347_8867088),
        contract_address: data.contract_client.address.clone(),
        min_col_rate: 1_1000000,
        min_debt_creation: 5000_0000000,
        opening_col_rate: 1_1500000,
    }
}

pub fn set_initial_state(env: &Env, data: &TestData, base_variables: &InitialVariables) {
    data.contract_client.init(
        &data.contract_admin,
        &data.oracle_admin,
        &data.protocol_manager,
        &data.collateral_token_client.address,
        &data.stable_token_issuer,
        &data.treasury,
        &data.fee,
    );

    data.contract_client.create_currency(
        &data.stable_token_denomination,
        &data.stable_token_client.address,
    );

    data.contract_client.set_currency_rate(
        &data.stable_token_denomination,
        &base_variables.currency_price,
    );

    data.contract_client
        .toggle_currency(&data.stable_token_denomination, &true);

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    token::StellarAssetClient::new(&env, &data.stable_token_client.address)
        .set_admin(&base_variables.contract_address);

    token::StellarAssetClient::new(&env, &data.stable_token_client.address)
        .mint(&data.stable_token_issuer, &90000000000000000000);
}
