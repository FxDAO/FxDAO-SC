#![cfg(test)]

use crate::contract::VaultsContract;
use crate::oracle::{Asset, AssetsData, Client as OracleClient, CoreData, CustomerQuota};
use crate::storage::vaults::{OptionalVaultKey, Vault, VaultKey};
use crate::utils::payments::calc_fee;
use crate::{oracle, VaultsContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, token, Address, Env, Symbol, Vec};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

use crate::tests::test_utils::InitialVariables;
use crate::tests::test_utils::{create_token_contract, update_oracle_price};

mod stable_liquidity_pool {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/stable_liquidity_pool.wasm"
    );
}
use stable_liquidity_pool::Client as StableLiquidityPoolContractClient;

// --- vaults -----------------------------------------------------------------

pub struct TestDataVaults<'a> {
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
    pub stable_token_issuer: &'a Address,
    pub stable_token_client: &'a TokenClient<'a>,
    pub stable_token_admin_client: &'a TokenAdminClient<'a>,

    pub oracle: Address,
    pub oracle_contract_client: OracleClient<'a>,
    pub oracle_contract_admin: Address,
}

pub fn create_base_data_vaults<'a>(
    env: &Env,
    stable_token_issuer: &'a Address,
    stable_token_client: &'a TokenClient,
    stable_token_admin_client: &'a TokenAdminClient,
) -> TestDataVaults<'a> {
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

    // Create the contract
    let contract_admin: Address = Address::generate(&env);
    let protocol_manager: Address = Address::generate(&env);
    let contract_client =
        VaultsContractClient::new(&env, &env.register_contract(None, VaultsContract));

    // Oracle contract
    let oracle: Address = env.register_contract_wasm(None, oracle::WASM);
    let oracle_contract_client: OracleClient = OracleClient::new(&env, &oracle);
    let oracle_contract_admin: Address = Address::generate(&env);

    return TestDataVaults {
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
    };
}

pub fn create_base_variables_vaults(env: &Env, data: &TestDataVaults) -> InitialVariables {
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

pub fn set_initial_state_vaults(
    env: &Env,
    data: &TestDataVaults,
    base_variables: &InitialVariables,
) {
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

    init_oracle_contract_vaults(&env, &data, &(base_variables.currency_price as i128));

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

pub fn init_oracle_contract_vaults(env: &Env, data: &TestDataVaults, rate: &i128) {
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

// --- stable-liquidity-pool --------------------------------------------------

pub struct TestDataLiquidity<'a> {
    pub stable_liquidity_pool_contract_client: StableLiquidityPoolContractClient<'a>,

    pub admin: Address,
    pub manager: Address,
    pub governance_token_admin: Address,
    pub governance_token_client: TokenClient<'a>,
    pub governance_token_admin_client: TokenAdminClient<'a>,

    pub usdc_token_admin: Address,
    pub usdc_token_client: TokenClient<'a>,
    pub usdc_token_admin_client: TokenAdminClient<'a>,

    pub usdt_token_admin: Address,
    pub usdt_token_client: TokenClient<'a>,
    pub usdt_token_admin_client: TokenAdminClient<'a>,

    pub usdx_token_admin: Address,
    pub usdx_token_client: TokenClient<'a>,
    pub usdx_token_admin_client: TokenAdminClient<'a>,

    pub fee_percentage: u128,
    pub treasury: Address,
    pub minted_asset_amount: u128,
}

pub fn create_test_data_liquidity(env: &Env) -> TestDataLiquidity {
    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let governance_token_admin = Address::generate(&env);
    let (governance_token_client, governance_token_admin_client) =
        create_token_contract(&env, &governance_token_admin);

    let usdc_token_admin = Address::generate(&env);
    let (usdc_token_client, usdc_token_admin_client) =
        create_token_contract(&env, &usdc_token_admin);

    let usdt_token_admin = Address::generate(&env);
    let (usdt_token_client, usdt_token_admin_client) =
        create_token_contract(&env, &usdt_token_admin);

    let usdx_token_admin = Address::generate(&env);
    let (usdx_token_client, usdx_token_admin_client) =
        create_token_contract(&env, &usdx_token_admin);

    let fee_percentage = 30000; // 0.3%
    let treasury = Address::generate(&env);

    // Stable Liquidity Pool
    let oracle: Address = env.register_contract_wasm(None, stable_liquidity_pool::WASM);
    let stable_liquidity_pool_contract_client: StableLiquidityPoolContractClient =
        StableLiquidityPoolContractClient::new(&env, &oracle);

    TestDataLiquidity {
        stable_liquidity_pool_contract_client,
        admin,
        manager,
        governance_token_admin,
        governance_token_client,
        governance_token_admin_client,
        usdc_token_admin,
        usdc_token_client,
        usdc_token_admin_client,
        usdt_token_admin,
        usdt_token_client,
        usdt_token_admin_client,
        usdx_token_admin,
        usdx_token_client,
        usdx_token_admin_client,
        fee_percentage,
        treasury,
        minted_asset_amount: 10_000_0000000,
    }
}

pub fn init_contract_liquidity(env: &Env, test_data: &TestDataLiquidity) {
    test_data.stable_liquidity_pool_contract_client.init(
        &test_data.admin,
        &test_data.manager,
        &test_data.governance_token_client.address,
        &(Vec::from_array(
            &env,
            [
                test_data.usdc_token_client.address.clone(),
                test_data.usdt_token_client.address.clone(),
                test_data.usdx_token_client.address.clone(),
            ],
        )),
        &test_data.fee_percentage,
        &test_data.treasury,
    );
}

pub fn prepare_test_accounts_liquidity(test_data: &TestDataLiquidity, accounts: &Vec<Address>) {
    for account in accounts.iter() {
        test_data
            .usdc_token_admin_client
            .mint(&account, &(test_data.minted_asset_amount as i128));
        test_data
            .usdt_token_admin_client
            .mint(&account, &(test_data.minted_asset_amount as i128));
        test_data
            .usdx_token_admin_client
            .mint(&account, &(test_data.minted_asset_amount as i128));
    }
}

// --- Setup Functions --------------------------------------------------------

pub fn setup_liquidity_pools(
    env: &Env,
    test_data_stable: &TestDataLiquidity,
    depositors: &[Address],
) {
    env.mock_all_auths();

    init_contract_liquidity(&env, &test_data_stable);

    let deposit_amount: u128 = 1000_0000000;

    prepare_test_accounts_liquidity(&test_data_stable, &Vec::from_slice(&env, depositors));

    // Each address deposits 1000 USDx
    depositors.iter().for_each(|depositor| {
        test_data_stable
            .stable_liquidity_pool_contract_client
            .deposit(
                depositor,
                &test_data_stable.usdx_token_client.address,
                &deposit_amount,
            )
    });

    assert_eq!(
        (depositors.len() * 1000_0000000) as i128,
        test_data_stable.usdx_token_client.balance(
            &test_data_stable
                .stable_liquidity_pool_contract_client
                .address
        )
    );

    // Each address deposits 1000 USDc
    depositors.iter().for_each(|depositor| {
        test_data_stable
            .stable_liquidity_pool_contract_client
            .deposit(
                depositor,
                &test_data_stable.usdc_token_client.address,
                &deposit_amount,
            )
    });

    assert_eq!(
        (depositors.len() * 1000_0000000) as i128,
        test_data_stable.usdc_token_client.balance(
            &test_data_stable
                .stable_liquidity_pool_contract_client
                .address
        )
    );

    // Each address deposits 1000 USDt
    depositors.iter().for_each(|depositor| {
        test_data_stable
            .stable_liquidity_pool_contract_client
            .deposit(
                depositor,
                &test_data_stable.usdt_token_client.address,
                &deposit_amount,
            )
    });

    assert_eq!(
        (depositors.len() * 1000_0000000) as i128,
        test_data_stable.usdt_token_client.balance(
            &test_data_stable
                .stable_liquidity_pool_contract_client
                .address
        )
    );
}

pub fn setup_vaults(env: &Env, data: &TestDataVaults, depositors: &[&Address]) {
    // let data= create_base_data(&env);

    env.mock_all_auths();
    let base_variables = create_base_variables_vaults(&env, &data);
    set_initial_state_vaults(&env, &data, &base_variables);

    let currency_price: u128 = 920330;
    let min_debt_creation: u128 = 1000000000;

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,     // No diff at 11000000 OR 110%
        &min_debt_creation,               // Dropping from 5000 to 100
        &base_variables.opening_col_rate, // No diff at 11000000 OR 115%
        &data.stable_token_denomination,
    );

    update_oracle_price(
        &env,
        &data.oracle_contract_client,
        &data.stable_token_denomination,
        &(currency_price as i128),
    );

    // First: This deposit should have an index of: 3233_7500000
    let depositor_1 = depositors[0];
    let depositor_1_debt: u128 = 100_0000000;
    let depositor_1_collateral_amount: u128 = 3250_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_1, &(depositor_1_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_1,
        &depositor_1_debt,
        &depositor_1_collateral_amount,
        &data.stable_token_denomination,
    );

    let depositor_1_vault: Vault = data
        .contract_client
        .get_vault(&depositor_1, &data.stable_token_denomination);

    // Second: This deposit should have an index of: 3233_7500000
    let depositor_2 = depositors[1];
    let depositor_2_debt: u128 = 100_0000000;
    let depositor_2_collateral_amount: u128 = 3250_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_2, &(depositor_2_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(VaultKey {
            index: depositor_1_vault.index.clone(),
            account: depositor_1_vault.account.clone(),
            denomination: data.stable_token_denomination.clone(),
        }),
        &depositor_2,
        &depositor_2_debt,
        &depositor_2_collateral_amount,
        &data.stable_token_denomination,
    );

    // Third: deposit should have an index of: 1747_6464285
    let depositor_3 = depositors[2];
    let depositor_3_debt: u128 = 140_0000000;
    let depositor_3_collateral_amount: u128 = 2459_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_3, &(depositor_3_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_3,
        &depositor_3_debt,
        &depositor_3_collateral_amount,
        &data.stable_token_denomination,
    );

    // Fourth: This deposit should have an index of: 5970_0000000
    let depositor_4 = depositors[3];
    let depositor_4_debt: u128 = 150_000_0000000;
    let depositor_4_collateral_amount: u128 = 9_000_000_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_4, &(depositor_4_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(VaultKey {
            index: depositor_1_vault.index.clone(),
            account: depositor_1_vault.account.clone(),
            denomination: data.stable_token_denomination.clone(),
        }),
        &depositor_4,
        &depositor_4_debt,
        &depositor_4_collateral_amount,
        &data.stable_token_denomination,
    );
}
