// TODO: specify all the steps in the tests

#![cfg(test)]
extern crate std;
use crate::storage_types::*;

use crate::storage_types::CoreState;
use soroban_sdk::{Address, Env, IntoVal, Symbol};

use crate::tests::test_utils::*;

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
    let core_state: CoreState = data.contract_client.get_core_state();

    assert_eq!(saved_admin, data.contract_admin);
    assert_eq!(
        core_state.col_token,
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
fn test_set_and_get_currency_vault_conditions() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);

    data.contract_client.init(
        &data.contract_admin,
        &data.collateral_token_client.contract_id,
        &data.stable_token_issuer,
    );

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
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
            Symbol::new(&env, "set_vault_conditions"),
            // Arguments used (converted to the, &data.stable_token_denomination env-managed vector via `into_val`)
            (
                base_variables.min_col_rate.clone(),
                base_variables.min_debt_creation.clone(),
                base_variables.opening_col_rate.clone(),
                data.stable_token_denomination.clone()
            )
                .into_val(&env)
        )]
    );

    // Fail if one value is neative
    assert!(data
        .contract_client
        .try_set_vault_conditions(
            &base_variables.min_col_rate,
            &base_variables.min_debt_creation,
            &-23,
            &data.stable_token_denomination,
        )
        .is_err());

    let currency_vault_conditions = data
        .contract_client
        .get_vault_conditions(&data.stable_token_denomination);

    assert_eq!(currency_vault_conditions.min_col_rate, 11000000);
    assert_eq!(currency_vault_conditions.min_debt_creation, 50000000000);
    assert_eq!(currency_vault_conditions.opening_col_rate, 11500000);
}

#[test]
fn test_set_and_get_rate() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let rate: i128 = 931953;
    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &rate);

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            data.contract_admin.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            Symbol::new(&env, "set_currency_rate"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (data.stable_token_denomination.clone(), rate.clone()).into_val(&env.clone())
        )]
    );

    let current_currency_rate: Currency = data
        .contract_client
        .get_currency(&data.stable_token_denomination);

    // We test that the first update is done correctly
    assert_eq!(&current_currency_rate.rate, &rate);

    let new_rate: i128 = 941953;

    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &new_rate);

    let new_protocol_rate: Currency = data
        .contract_client
        .get_currency(&data.stable_token_denomination);

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
