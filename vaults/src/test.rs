// TODO: specify all the steps in the tests

#![cfg(test)]
extern crate std;
use crate::storage_types::*;
use crate::token;
use crate::VaultsContractClient;

use crate::storage_types::CoreState;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol, Address, Env, IntoVal, Symbol};

fn create_token_contract(e: &Env, admin: &Address) -> token::Client {
    token::Client::new(&e, &e.register_stellar_asset_contract(admin.clone()))
}

struct TestData {
    // Contract data
    contract_admin: Address,
    contract_client: VaultsContractClient,

    // Collateral token data
    collateral_token_admin: Address,
    collateral_token_client: token::Client,

    // Native token data
    // native_token_admin: Address,
    native_token_client: token::Client,

    // Stable token data
    stable_token_denomination: Symbol,
    stable_token_issuer: Address,
    stable_token_client: token::Client,
}

struct InitialVariables {
    currency_price: i128,
    depositor: Address,
    initial_debt: i128,
    collateral_amount: i128,
    contract_address: Address,
    mn_col_rte: i128,
    mn_v_c_amt: i128,
    op_col_rte: i128,
}

fn create_base_data(env: &Env) -> TestData {
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
    let contract_client = VaultsContractClient::new(
        &env,
        &env.register_contract(None, crate::contract::VaultsContract),
    );

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

fn create_base_variables(env: &Env, data: &TestData) -> InitialVariables {
    InitialVariables {
        currency_price: 20000000,
        depositor: Address::random(&env),
        initial_debt: 50000000000,
        collateral_amount: 50000000000,
        contract_address: Address::from_contract_id(&env, &data.contract_client.contract_id),
        mn_col_rte: 11000000,
        mn_v_c_amt: 50000000000,
        op_col_rte: 11500000,
    }
}

fn set_initial_state(env: &Env, data: &TestData, base_variables: &InitialVariables) {
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

    data.contract_client.s_p_state(
        &base_variables.mn_col_rte,
        &base_variables.mn_v_c_amt,
        &base_variables.op_col_rte,
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

#[test]
fn test_set_and_get_core_state() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.init(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.stable_token_issuer,
    );

    let saved_admin: Address = data.contract_client.get_admin();
    let core_state: CoreState = data.contract_client.g_c_state();

    assert_eq!(saved_admin, data.contract_admin);
    assert_eq!(
        core_state.colla_tokn,
        data.collateral_token_client.contract_id
    );
}

#[test]
#[should_panic(expected = "Status(ContractError(0))")]
fn test_init_panic() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.init(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.stable_token_issuer,
    );

    data.contract_client.init(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.stable_token_issuer,
    );
}

#[test]
fn test_set_and_get_protocol_state() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);

    data.contract_client.init(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.stable_token_issuer,
    );

    data.contract_client.s_p_state(
        &base_variables.mn_col_rte,
        &base_variables.mn_v_c_amt,
        &base_variables.op_col_rte,
    );

    // Check the admin is the one who call it
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            data.contract_admin.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            symbol!("s_p_state"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (
                base_variables.mn_col_rte.clone(),
                base_variables.mn_v_c_amt.clone(),
                base_variables.op_col_rte.clone()
            )
                .into_val(&env)
        )]
    );

    // Fail if one value is neative
    assert!(data
        .contract_client
        .try_s_p_state(&base_variables.mn_col_rte, &base_variables.mn_v_c_amt, &-23)
        .is_err());

    let protocol_state = data.contract_client.g_p_state();

    assert_eq!(protocol_state.mn_col_rte, 11000000);
    assert_eq!(protocol_state.mn_v_c_amt, 50000000000);
    assert_eq!(protocol_state.op_col_rte, 11500000);
}

#[test]
fn test_set_and_get_rate() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let rate: i128 = 931953;
    data.contract_client
        .s_cy_rate(&data.stable_token_denomination, &rate);

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            data.contract_admin.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            symbol!("s_cy_rate"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (data.stable_token_denomination.clone(), rate.clone()).into_val(&env.clone())
        )]
    );

    let current_currency_rate: Currency =
        data.contract_client.get_cy(&data.stable_token_denomination);

    // We test that the first update is done correctly
    assert_eq!(&current_currency_rate.rate, &rate);

    let new_rate: i128 = 941953;

    data.contract_client
        .s_cy_rate(&data.stable_token_denomination, &new_rate);

    let new_protocol_rate: Currency = data.contract_client.get_cy(&data.stable_token_denomination);

    // Testing that the state gets updated from the one saved before
    assert_eq!(&new_protocol_rate.rate, &new_rate);
    assert_eq!(
        &current_currency_rate.last_updte,
        &new_protocol_rate.last_updte
    );

    // TODO: test the last update once we have added that logic
    // env.ledger().set(LedgerInfo {
    //   timestamp: 12345,
    //   protocol_version: 1,
    //   sequence_number: 10,
    //   network_id: Default::default(),
    //   base_reserve: 10,
    // });
}

#[test]
fn test_new_vault() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.init(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.stable_token_issuer,
    );

    let currency_price: i128 = 830124; // 0.0830124
    let depositor = Address::random(&env);
    let initial_debt: i128 = 5_000_0000000; // USD 5000
    let collateral_amount: i128 = 90_347_8867088; // 90,347.8867088 XLM
    let contract_address: Address =
        Address::from_contract_id(&env, &data.contract_client.contract_id);

    let mn_col_rte: i128 = 1_1000000;
    let mn_v_c_amt: i128 = 5000_0000000;
    let op_col_rte: i128 = 1_1500000;

    token::Client::new(&env, &data.stable_token_client.contract_id).incr_allow(
        &data.stable_token_issuer,
        &contract_address,
        &90000000000000000000,
    );

    token::Client::new(&env, &data.stable_token_client.contract_id).mint(
        &data.stable_token_issuer,
        &data.stable_token_issuer,
        &90000000000000000000,
    );

    // If the method is called before before the currency is active it should fail
    assert!(data
        .contract_client
        .try_new_vault(
            &depositor,
            &initial_debt,
            &collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.new_cy(
        &data.stable_token_denomination,
        &data.stable_token_client.contract_id,
    );

    data.contract_client
        .s_cy_rate(&data.stable_token_denomination, &currency_price);

    data.contract_client
        .toggle_cy(&data.stable_token_denomination, &true);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor,
        &(collateral_amount * 2),
    );

    // If the method is called before protocol state is set it should fail
    assert!(data
        .contract_client
        .try_new_vault(
            &depositor,
            &initial_debt,
            &collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client
        .s_p_state(&mn_col_rte, &mn_v_c_amt, &op_col_rte);

    data.contract_client.new_vault(
        &depositor,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            depositor.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            symbol!("new_vault"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (
                depositor.clone(),
                initial_debt.clone(),
                collateral_amount.clone(),
                data.stable_token_denomination.clone(),
            )
                .into_val(&env),
        )]
    );

    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount)
    );
    assert_eq!(data.stable_token_client.balance(&depositor), (initial_debt));

    let currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(currency_stats.tot_vaults, 1);
    assert_eq!(currency_stats.tot_debt, initial_debt);
    assert_eq!(currency_stats.tot_col, collateral_amount);

    // Should fail if user tries to create a new vault but already have one
    assert!(data
        .contract_client
        .try_new_vault(
            &depositor,
            &initial_debt,
            &collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    let depositor_2 = Address::random(&env);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor_2,
        &(collateral_amount * 2),
    );

    data.contract_client.new_vault(
        &depositor_2,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    assert_eq!(
        data.stable_token_client.balance(&depositor_2),
        (initial_debt)
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.tot_vaults, 2);
    assert_eq!(updated_currency_stats.tot_debt, initial_debt * 2);
    assert_eq!(updated_currency_stats.tot_col, collateral_amount * 2);
}

#[test]
fn test_increase_collateral() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let depositor = Address::random(&env);
    let initial_debt: i128 = 50000000000;
    let collateral_amount: i128 = 50000000000;
    let contract_address: Address =
        Address::from_contract_id(&env, &data.contract_client.contract_id);

    let mn_col_rte: i128 = 11000000;
    let mn_v_c_amt: i128 = 50000000000;
    let op_col_rte: i128 = 11500000;

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor,
        &(collateral_amount * 2),
    );

    data.stable_token_client.mint(
        &data.stable_token_issuer,
        &contract_address,
        &(initial_debt),
    );

    data.contract_client
        .s_p_state(&mn_col_rte, &mn_v_c_amt, &op_col_rte);

    // It should fail if the user doesn't have a Vault open
    assert!(data
        .contract_client
        .try_incr_col(
            &depositor,
            &collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.new_vault(
        &depositor,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    let current_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(current_currency_stats.tot_vaults, 1);
    assert_eq!(current_currency_stats.tot_debt, initial_debt);
    assert_eq!(current_currency_stats.tot_col, collateral_amount);

    data.contract_client.incr_col(
        &depositor,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            depositor.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            symbol!("incr_col"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (
                depositor.clone(),
                collateral_amount.clone(),
                data.stable_token_denomination.clone()
            )
                .into_val(&env),
        )]
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.tot_vaults, 1);
    assert_eq!(updated_currency_stats.tot_debt, initial_debt);
    assert_eq!(updated_currency_stats.tot_col, collateral_amount * 2);

    assert_eq!(data.collateral_token_client.balance(&depositor), 0);
    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount * 2)
    );
}

#[test]
fn test_increase_debt() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &base_variables.depositor,
        &(base_variables.collateral_amount * 5),
    );

    data.stable_token_client.mint(
        &data.stable_token_issuer,
        &base_variables.contract_address,
        &(base_variables.initial_debt * 5),
    );

    // It should fail if the user doesn't have a Vault open
    assert!(data
        .contract_client
        .try_incr_debt(
            &base_variables.depositor,
            &base_variables.collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.new_vault(
        &base_variables.depositor,
        &base_variables.initial_debt,
        &(base_variables.collateral_amount * 2),
        &data.stable_token_denomination,
    );

    let current_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(current_currency_stats.tot_vaults, 1);
    assert_eq!(current_currency_stats.tot_debt, base_variables.initial_debt);
    assert_eq!(
        current_currency_stats.tot_col,
        base_variables.collateral_amount * 2
    );

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        base_variables.initial_debt
    );

    data.contract_client.incr_debt(
        &base_variables.depositor,
        &base_variables.initial_debt,
        &data.stable_token_denomination,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            base_variables.depositor.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            symbol!("incr_debt"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (
                base_variables.depositor.clone(),
                base_variables.initial_debt.clone(),
                data.stable_token_denomination.clone(),
            )
                .into_val(&env),
        )]
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.tot_vaults, 1);
    assert_eq!(
        updated_currency_stats.tot_debt,
        base_variables.initial_debt * 2
    );
    assert_eq!(
        updated_currency_stats.tot_col,
        base_variables.collateral_amount * 2
    );

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        (base_variables.initial_debt * 2)
    );
}

#[test]
fn test_pay_debt() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let currency_price: i128 = 20000000;
    let depositor = Address::random(&env);
    let initial_debt: i128 = 50000000000;
    let collateral_amount: i128 = 50000000000;
    let contract_address: Address =
        Address::from_contract_id(&env, &data.contract_client.contract_id);

    let mn_col_rte: i128 = 11000000;
    let mn_v_c_amt: i128 = 50000000000;
    let op_col_rte: i128 = 11500000;

    data.contract_client
        .s_cy_rate(&data.stable_token_denomination, &currency_price);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor,
        &(collateral_amount),
    );

    data.stable_token_client.mint(
        &data.stable_token_issuer,
        &contract_address,
        &(initial_debt * 10),
    );

    data.contract_client
        .s_p_state(&mn_col_rte, &mn_v_c_amt, &op_col_rte);

    // It should fail if the user doesn't have a Vault open
    assert!(data
        .contract_client
        .try_pay_debt(
            &depositor,
            &(initial_debt / 2),
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.new_vault(
        &depositor,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    let current_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(current_currency_stats.tot_vaults, 1);
    assert_eq!(current_currency_stats.tot_debt, initial_debt);
    assert_eq!(current_currency_stats.tot_col, collateral_amount);

    data.contract_client.pay_debt(
        &depositor,
        &(initial_debt / 2),
        &data.stable_token_denomination,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            depositor.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            symbol!("pay_debt"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (
                depositor.clone(),
                (initial_debt / 2).clone(),
                data.stable_token_denomination.clone()
            )
                .into_val(&env),
        )]
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.tot_vaults, 1);
    assert_eq!(updated_currency_stats.tot_debt, initial_debt / 2);
    assert_eq!(updated_currency_stats.tot_col, collateral_amount);

    assert_eq!(
        data.stable_token_client.balance(&depositor),
        (initial_debt / 2)
    );
    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount)
    );

    data.contract_client.pay_debt(
        &depositor,
        &(initial_debt / 2),
        &data.stable_token_denomination,
    );

    let final_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(final_currency_stats.tot_vaults, 0);
    assert_eq!(final_currency_stats.tot_debt, 0);
    assert_eq!(final_currency_stats.tot_col, 0);

    assert_eq!(data.stable_token_client.balance(&depositor), 0);
    assert_eq!(data.collateral_token_client.balance(&contract_address), 0);
}
