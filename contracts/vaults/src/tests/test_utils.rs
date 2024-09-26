#![cfg(test)]

use crate::contract::VaultsContract;
use crate::oracle::{
    Asset, AssetsData, Client as OracleClient, CoreData, CustomerQuota, PriceData,
};
use crate::utils::payments::calc_fee;
use crate::{oracle, VaultsContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, token, Address, Env, Symbol, Vec};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

pub fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(e, &contract_address),
        TokenAdminClient::new(e, &contract_address),
    )
}

pub struct TestData<'a> {
    // Contract data
    pub contract_admin: Address,
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

    pub oracle: Address,
    pub oracle_contract_client: OracleClient<'a>,
    pub oracle_contract_admin: Address,
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
    let contract_admin: Address = Address::generate(&env);
    let protocol_manager: Address = Address::generate(&env);
    let contract_client =
        VaultsContractClient::new(&env, &env.register_contract(None, VaultsContract));

    // Oracle contract
    let oracle: Address = env.register_contract_wasm(None, oracle::WASM);
    let oracle_contract_client: OracleClient = OracleClient::new(&env, &oracle);
    let oracle_contract_admin: Address = Address::generate(&env);

    TestData {
        contract_admin,
        protocol_manager,
        contract_client,
        treasury: Address::generate(&env),
        fee: 50000, // 0.5%
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

        oracle,
        oracle_contract_client,
        oracle_contract_admin,
    }
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
    data.contract_client.mock_all_auths().init(
        &data.contract_admin,
        &data.protocol_manager,
        &data.collateral_token_client.address,
        &data.stable_token_issuer,
        &data.treasury,
        &data.fee,
        &data.oracle,
    );

    data.contract_client.mock_all_auths().create_currency(
        &data.stable_token_denomination,
        &data.stable_token_client.address,
    );

    init_oracle_contract(&env, &data, &(base_variables.currency_price as i128));

    data.contract_client
        .mock_all_auths()
        .toggle_currency(&data.stable_token_denomination, &true);

    data.contract_client.mock_all_auths().set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    token::StellarAssetClient::new(&env, &data.stable_token_client.address)
        .mock_all_auths()
        .set_admin(&base_variables.contract_address);

    token::StellarAssetClient::new(&env, &data.stable_token_client.address)
        .mock_all_auths()
        .mint(&data.stable_token_issuer, &90000000000000000000);
}

pub fn init_oracle_contract(env: &Env, data: &TestData, rate: &i128) {
    data.oracle_contract_client.mock_all_auths().init(
        &CoreData {
            adm: data.oracle_contract_admin.clone(),
            tick: 60,
            dp: 7,
        },
        &AssetsData {
            base: Asset::Stellar(data.collateral_token_client.address.clone()),
            assets: Vec::from_array(&env, [Asset::Other(data.stable_token_denomination.clone())]),
        },
    );

    update_oracle_price(
        &env,
        &data.oracle_contract_client,
        &data.stable_token_denomination,
        rate,
    );

    data.oracle_contract_client.mock_all_auths().set_quota(
        &data.contract_client.address,
        &CustomerQuota {
            max: 0,
            current: 0,
            exp: u64::MAX,
        },
    );
}

pub fn update_oracle_price(env: &Env, client: &OracleClient, denomination: &Symbol, price: &i128) {
    client.mock_all_auths().set_records(
        &Vec::from_array(&env, [Asset::Other(denomination.clone())]),
        &Vec::from_array(
            &env,
            [PriceData {
                price: price.clone(),
                timestamp: env.ledger().timestamp(),
            }],
        ),
    );
}
