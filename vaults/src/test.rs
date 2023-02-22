// TODO: documentate all the steps in the tests

#![cfg(test)]
extern crate std;
use crate::storage_types::*;
use crate::token;
use crate::VaultsContractClient;

use crate::storage_types::CoreState;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol, Address, Env, IntoVal};

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

    // Collateral token data
    // native_token_admin: Address,
    native_token_client: token::Client,

    // Collateral token data
    stable_token_admin: Address,
    stable_token_client: token::Client,
}

struct InitialVariables {
    collateral_price: u128,
    depositor: Address,
    initial_debt: u128,
    collateral_amount: u128,
    contract_address: Address,
    mn_col_rte: u128,
    mn_v_c_amt: u128,
    op_col_rte: u128,
}

fn create_base_data(env: &Env) -> TestData {
    // Set up the collateral token
    let collateral_token_admin = Address::random(&env);
    let collateral_token_client = create_token_contract(&env, &collateral_token_admin);

    // Set up the native token
    let native_token_admin = Address::random(&env);
    let native_token_client = create_token_contract(&env, &native_token_admin);

    // Set up the stable token
    let stable_token_admin = Address::random(&env);
    let stable_token_client = create_token_contract(&env, &stable_token_admin);

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
        stable_token_admin,
        stable_token_client,
    };
}

fn create_base_variables(env: &Env, data: &TestData) -> InitialVariables {
    InitialVariables {
        collateral_price: 20000000,
        depositor: Address::random(&env),
        initial_debt: 50000000000,
        collateral_amount: 50000000000,
        contract_address: Address::from_contract_id(&env, &data.contract_client.contract_id),
        mn_col_rte: 11000000,
        mn_v_c_amt: 50000000000,
        op_col_rte: 11500000,
    }
}

fn set_initial_state(data: &TestData, base_variables: &InitialVariables) {
    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );

    data.contract_client
        .s_p_c_prce(&base_variables.collateral_price);

    data.contract_client.s_p_state(
        &base_variables.mn_col_rte,
        &base_variables.mn_v_c_amt,
        &base_variables.op_col_rte,
    );
}

#[test]
fn test_set_core_state() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );
}

#[test]
#[should_panic(expected = "Status(ContractError(0))")]
fn test_init_panic() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );
}

#[test]
fn test_get_core_state() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );

    let saved_admin: Address = data.contract_client.get_admin();
    let core_state: CoreState = data.contract_client.g_c_state();

    assert_eq!(saved_admin, data.contract_admin);
    assert_eq!(core_state.nativ_tokn, data.native_token_client.contract_id);
    assert_eq!(
        core_state.colla_tokn,
        data.collateral_token_client.contract_id
    );
    assert_eq!(core_state.stble_tokn, data.stable_token_client.contract_id);
}

#[test]
fn test_set_and_get_protocol_state() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );

    let mn_col_rte: u128 = 11000000;
    let mn_v_c_amt: u128 = 50000000000;
    let op_col_rte: u128 = 11500000;

    data.contract_client
        .s_p_state(&mn_col_rte, &mn_v_c_amt, &op_col_rte);

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
            (mn_col_rte.clone(), mn_v_c_amt.clone(), op_col_rte.clone()).into_val(&env)
        )]
    );

    let protocol_state = data.contract_client.g_p_state();

    assert_eq!(protocol_state.mn_col_rte, mn_col_rte);
    assert_eq!(protocol_state.mn_v_c_amt, mn_v_c_amt);
    assert_eq!(protocol_state.op_col_rte, op_col_rte);
}

#[test]
fn test_set_and_get_rate() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );

    let rate: u128 = 931953;

    data.contract_client.s_p_c_prce(&rate);

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            data.contract_admin.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            symbol!("s_p_c_prce"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (rate.clone(),).into_val(&env.clone())
        )]
    );

    let current_protocol_rate: ProtocolCollateralPrice = data.contract_client.g_p_c_prce();

    // We test that the first update is done correctly
    assert_eq!(&current_protocol_rate.current, &rate);

    let new_rate: u128 = 941953;

    data.contract_client.s_p_c_prce(&new_rate);

    let new_protocol_rate: ProtocolCollateralPrice = data.contract_client.g_p_c_prce();

    // Testing that the state gets updated from the one saved before
    assert_eq!(&new_protocol_rate.current, &new_rate);
    assert_eq!(
        &current_protocol_rate.last_updte,
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

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );

    let collateral_price: u128 = 20000000;
    let depositor = Address::random(&env);
    let initial_debt: u128 = 50000000000;
    let collateral_amount: u128 = 50000000000;
    let contract_address: Address =
        Address::from_contract_id(&env, &data.contract_client.contract_id);

    let mn_col_rte: u128 = 11000000;
    let mn_v_c_amt: u128 = 50000000000;
    let op_col_rte: u128 = 11500000;

    // If the method is called before collateral price is set it should fail
    assert!(data
        .contract_client
        .try_new_vault(&depositor, &initial_debt, &collateral_amount)
        .is_err());

    data.contract_client.s_p_c_prce(&collateral_price);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor,
        &((collateral_amount * 2) as i128),
    );

    data.stable_token_client.mint(
        &data.stable_token_admin,
        &contract_address,
        &((initial_debt * 10) as i128),
    );

    // If the method is called before protocol state is set it should fail
    assert!(data
        .contract_client
        .try_new_vault(&depositor, &initial_debt, &collateral_amount)
        .is_err());

    data.contract_client
        .s_p_state(&mn_col_rte, &mn_v_c_amt, &op_col_rte);

    data.contract_client
        .new_vault(&depositor, &initial_debt, &collateral_amount);

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
            )
                .into_val(&env),
        )]
    );

    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount as i128)
    );
    assert_eq!(
        data.stable_token_client.balance(&depositor),
        (initial_debt as i128)
    );

    let current_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(current_protocol_stats.tot_vaults, 1);
    assert_eq!(current_protocol_stats.tot_debt, initial_debt);
    assert_eq!(current_protocol_stats.tot_col, collateral_amount);

    // Should fail if user tries to create a new vault but already have one
    assert!(data
        .contract_client
        .try_new_vault(&depositor, &initial_debt, &collateral_amount)
        .is_err());

    let depositor_2 = Address::random(&env);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor_2,
        &((collateral_amount * 2) as i128),
    );

    data.contract_client
        .new_vault(&depositor_2, &initial_debt, &collateral_amount);

    assert_eq!(
        data.stable_token_client.balance(&depositor_2),
        (initial_debt as i128)
    );

    let updated_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(updated_protocol_stats.tot_vaults, 2);
    assert_eq!(updated_protocol_stats.tot_debt, initial_debt * 2);
    assert_eq!(updated_protocol_stats.tot_col, collateral_amount * 2);
}

#[test]
fn test_pay_debt() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );

    let collateral_price: u128 = 20000000;
    let depositor = Address::random(&env);
    let initial_debt: u128 = 50000000000;
    let collateral_amount: u128 = 50000000000;
    let contract_address: Address =
        Address::from_contract_id(&env, &data.contract_client.contract_id);

    let mn_col_rte: u128 = 11000000;
    let mn_v_c_amt: u128 = 50000000000;
    let op_col_rte: u128 = 11500000;

    data.contract_client.s_p_c_prce(&collateral_price);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor,
        &(collateral_amount as i128),
    );

    data.stable_token_client.mint(
        &data.stable_token_admin,
        &contract_address,
        &((initial_debt * 10) as i128),
    );

    data.contract_client
        .s_p_state(&mn_col_rte, &mn_v_c_amt, &op_col_rte);

    // It should fail if the user doesn't have a Vault open
    assert!(data
        .contract_client
        .try_pay_debt(&depositor, &(initial_debt / 2))
        .is_err());

    data.contract_client
        .new_vault(&depositor, &initial_debt, &collateral_amount);

    let current_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(current_protocol_stats.tot_vaults, 1);
    assert_eq!(current_protocol_stats.tot_debt, initial_debt);
    assert_eq!(current_protocol_stats.tot_col, collateral_amount);

    data.contract_client
        .pay_debt(&depositor, &(initial_debt / 2));

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
            (depositor.clone(), (initial_debt / 2).clone()).into_val(&env),
        )]
    );

    let updated_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(updated_protocol_stats.tot_vaults, 1);
    assert_eq!(updated_protocol_stats.tot_debt, initial_debt / 2);
    assert_eq!(updated_protocol_stats.tot_col, collateral_amount);

    assert_eq!(
        data.stable_token_client.balance(&depositor),
        (initial_debt / 2) as i128
    );
    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount) as i128
    );

    data.contract_client
        .pay_debt(&depositor, &(initial_debt / 2));

    let final_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(final_protocol_stats.tot_vaults, 0);
    assert_eq!(final_protocol_stats.tot_debt, 0);
    assert_eq!(final_protocol_stats.tot_col, 0);

    assert_eq!(data.stable_token_client.balance(&depositor), 0);
    assert_eq!(data.collateral_token_client.balance(&contract_address), 0);
}

#[test]
fn test_increase_collateral() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.s_c_state(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.native_token_client.contract_id,
        &data.stable_token_client.contract_id,
    );

    let collateral_price: u128 = 20000000;
    let depositor = Address::random(&env);
    let initial_debt: u128 = 50000000000;
    let collateral_amount: u128 = 50000000000;
    let contract_address: Address =
        Address::from_contract_id(&env, &data.contract_client.contract_id);

    let mn_col_rte: u128 = 11000000;
    let mn_v_c_amt: u128 = 50000000000;
    let op_col_rte: u128 = 11500000;

    data.contract_client.s_p_c_prce(&collateral_price);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor,
        &((collateral_amount * 2) as i128),
    );

    data.stable_token_client.mint(
        &data.stable_token_admin,
        &contract_address,
        &(initial_debt as i128),
    );

    data.contract_client
        .s_p_state(&mn_col_rte, &mn_v_c_amt, &op_col_rte);

    // It should fail if the user doesn't have a Vault open
    assert!(data
        .contract_client
        .try_incr_col(&depositor, &collateral_amount)
        .is_err());

    data.contract_client
        .new_vault(&depositor, &initial_debt, &collateral_amount);

    let current_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(current_protocol_stats.tot_vaults, 1);
    assert_eq!(current_protocol_stats.tot_debt, initial_debt);
    assert_eq!(current_protocol_stats.tot_col, collateral_amount);

    data.contract_client
        .incr_col(&depositor, &collateral_amount);

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
            (depositor.clone(), collateral_amount.clone()).into_val(&env),
        )]
    );

    let updated_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(updated_protocol_stats.tot_vaults, 1);
    assert_eq!(updated_protocol_stats.tot_debt, initial_debt);
    assert_eq!(updated_protocol_stats.tot_col, collateral_amount * 2);

    assert_eq!(data.collateral_token_client.balance(&depositor), 0);
    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount * 2) as i128
    );
}

#[test]
fn test_increase_debt() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&data, &base_variables);

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &base_variables.depositor,
        &((base_variables.collateral_amount * 5) as i128),
    );

    data.stable_token_client.mint(
        &data.stable_token_admin,
        &base_variables.contract_address,
        &((base_variables.initial_debt * 5) as i128),
    );

    // It should fail if the user doesn't have a Vault open
    assert!(data
        .contract_client
        .try_incr_debt(&base_variables.depositor, &base_variables.collateral_amount)
        .is_err());

    data.contract_client.new_vault(
        &base_variables.depositor,
        &base_variables.initial_debt,
        &(base_variables.collateral_amount * 2),
    );

    let current_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(current_protocol_stats.tot_vaults, 1);
    assert_eq!(current_protocol_stats.tot_debt, base_variables.initial_debt);
    assert_eq!(
        current_protocol_stats.tot_col,
        base_variables.collateral_amount * 2
    );

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        base_variables.initial_debt as i128
    );

    data.contract_client
        .incr_debt(&base_variables.depositor, &base_variables.initial_debt);

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
                base_variables.initial_debt.clone()
            )
                .into_val(&env),
        )]
    );

    let updated_protocol_stats: ProtStats = data.contract_client.g_p_stats();

    assert_eq!(updated_protocol_stats.tot_vaults, 1);
    assert_eq!(
        updated_protocol_stats.tot_debt,
        base_variables.initial_debt * 2
    );
    assert_eq!(
        updated_protocol_stats.tot_col,
        base_variables.collateral_amount * 2
    );

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        (base_variables.initial_debt * 2) as i128
    );
}
