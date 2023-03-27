#![cfg(test)]

extern crate std;

use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_initial_state, InitialVariables, TestData,
};
use crate::utils::vaults::calculate_user_vault_index;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, Vec};

#[test]
fn test_vault_indexes_logic_around() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let currency_price: i128 = 920330;
    let mn_col_rte: i128 = 11000000;
    let mn_v_c_amt: i128 = 1000000000;
    let op_col_rte: i128 = 11500000;

    data.contract_client.s_c_v_c(
        &mn_col_rte,
        &mn_v_c_amt,
        &op_col_rte,
        &data.stable_token_denomination,
    );

    data.contract_client
        .s_cy_rate(&data.stable_token_denomination, &currency_price);

    // First depositor
    let depositor_1 = Address::random(&env);
    let depositor_1_debt: i128 = 1500000000;
    let depositor_1_collateral_amount: i128 = 30000000000;

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor_1,
        &(depositor_1_collateral_amount * 2),
    );

    data.contract_client.new_vault(
        &depositor_1,
        &depositor_1_debt,
        &depositor_1_collateral_amount,
        &data.stable_token_denomination,
    );

    let mut current_indexes: Vec<i128> = data
        .contract_client
        .g_indexes(&data.stable_token_denomination);

    assert_eq!(current_indexes.len(), 1);
    assert_eq!(
        current_indexes.first().unwrap().unwrap(),
        calculate_user_vault_index(depositor_1_debt, depositor_1_collateral_amount)
    );

    // Second depositor
    let depositor_2 = Address::random(&env);
    let depositor_2_debt: i128 = 1400000000;
    let depositor_2_collateral_amount: i128 = 26000000000;

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor_2,
        &(depositor_2_collateral_amount * 2),
    );

    data.contract_client.new_vault(
        &depositor_2,
        &depositor_2_debt,
        &depositor_2_collateral_amount,
        &data.stable_token_denomination,
    );

    current_indexes = data
        .contract_client
        .g_indexes(&data.stable_token_denomination);

    assert_eq!(current_indexes.len(), 2);
    assert_eq!(
        current_indexes.first().unwrap().unwrap(),
        calculate_user_vault_index(depositor_2_debt, depositor_2_collateral_amount)
    );
    assert_eq!(
        current_indexes.last().unwrap().unwrap(),
        calculate_user_vault_index(depositor_1_debt, depositor_1_collateral_amount)
    );

    // Third depositor
    let depositor_3 = Address::random(&env);
    let depositor_3_debt: i128 = 1000000000;
    let depositor_3_collateral_amount: i128 = 32500000000;

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor_3,
        &(depositor_3_collateral_amount * 2),
    );

    data.contract_client.new_vault(
        &depositor_3,
        &depositor_3_debt,
        &depositor_3_collateral_amount,
        &data.stable_token_denomination,
    );

    current_indexes = data
        .contract_client
        .g_indexes(&data.stable_token_denomination);

    assert_eq!(current_indexes.len(), 3);
    assert_eq!(
        current_indexes.first().unwrap().unwrap(),
        calculate_user_vault_index(depositor_2_debt, depositor_2_collateral_amount)
    );
    assert_eq!(
        current_indexes.last().unwrap().unwrap(),
        calculate_user_vault_index(depositor_3_debt, depositor_3_collateral_amount)
    );

    // fourth depositor
    let depositor_4 = Address::random(&env);
    let depositor_4_debt: i128 = 1000000000;
    let depositor_4_collateral_amount: i128 = 32500000000;

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor_4,
        &(depositor_4_collateral_amount * 2),
    );

    data.contract_client.new_vault(
        &depositor_4,
        &depositor_4_debt,
        &depositor_4_collateral_amount,
        &data.stable_token_denomination,
    );

    current_indexes = data
        .contract_client
        .g_indexes(&data.stable_token_denomination);

    assert_eq!(current_indexes.len(), 3);
    assert_eq!(
        current_indexes.first().unwrap().unwrap(),
        calculate_user_vault_index(depositor_2_debt, depositor_2_collateral_amount)
    );
    assert_eq!(
        current_indexes.last().unwrap().unwrap(),
        calculate_user_vault_index(depositor_3_debt, depositor_3_collateral_amount)
    );

    // fifth depositor
    let depositor_5 = Address::random(&env);
    let depositor_5_debt: i128 = 1400000000;
    let depositor_5_collateral_amount: i128 = 24590000000;

    data.collateral_token_client.mint(
        &data.collateral_token_admin,
        &depositor_5,
        &(depositor_5_collateral_amount * 2),
    );

    data.contract_client.new_vault(
        &depositor_5,
        &depositor_5_debt,
        &depositor_5_collateral_amount,
        &data.stable_token_denomination,
    );

    current_indexes = data
        .contract_client
        .g_indexes(&data.stable_token_denomination);

    assert_eq!(current_indexes.len(), 4);
    assert_eq!(
        current_indexes.first().unwrap().unwrap(),
        calculate_user_vault_index(depositor_5_debt, depositor_5_collateral_amount)
    );
    assert_eq!(
        current_indexes.last().unwrap().unwrap(),
        calculate_user_vault_index(depositor_3_debt, depositor_3_collateral_amount)
    );
}
