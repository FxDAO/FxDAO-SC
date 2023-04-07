#![cfg(test)]
extern crate std;

use crate::storage_types::CurrencyStats;
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_initial_state, InitialVariables, TestData,
};
use crate::token;
use crate::utils::vaults::calculate_user_vault_index;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Env, IntoVal, Status, Symbol, Vec};

/// It test a simple liquidation
/// The vault must be removed and the collateral sent to the liquidator
/// Currency stats must be updated
#[test]
fn test_liquidation() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let first_rate: i128 = 931953;
    data.contract_client
        .s_cy_rate(&data.stable_token_denomination, &first_rate);

    let depositor: Address = Address::random(&env);
    let depositor_debt: i128 = 5_000_0000000;
    let depositor_collateral: i128 = 100_000_0000000;

    token::Client::new(&env, &data.collateral_token_client.contract_id).mint(
        &data.collateral_token_admin,
        &depositor,
        &depositor_collateral,
    );

    let liquidator: Address = Address::random(&env);
    let liquidator_debt: i128 = 5_000_0000000;
    let liquidator_collateral: i128 = 500_000_0000000;

    token::Client::new(&env, &data.collateral_token_client.contract_id).mint(
        &data.collateral_token_admin,
        &liquidator,
        &liquidator_collateral,
    );

    // Create both vaults
    data.contract_client.new_vault(
        &depositor,
        &depositor_debt,
        &depositor_collateral,
        &data.stable_token_denomination,
    );

    data.contract_client.new_vault(
        &liquidator,
        &liquidator_debt,
        &liquidator_collateral,
        &data.stable_token_denomination,
    );

    // It should throw an error because the vault can't be liquidated yet
    let cant_liquidate_error_result = data
        .contract_client
        .try_liquidate(
            &liquidator,
            &data.stable_token_denomination,
            &vec![&env, depositor.clone()],
        )
        .unwrap_err();

    assert_eq!(
        cant_liquidate_error_result,
        Ok(Status::from_contract_error(50003))
    );

    // We update the collateral price in order to put the depositor's vault below the min collateral ratio
    let second_rate: i128 = 531953;
    data.contract_client
        .s_cy_rate(&data.stable_token_denomination, &second_rate);

    data.contract_client.liquidate(
        &liquidator,
        &data.stable_token_denomination,
        &vec![&env, depositor.clone()],
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            // Address for which auth is performed
            liquidator.clone(),
            // Identifier of the called contract
            data.contract_client.contract_id.clone(),
            // Name of the called function
            Symbol::short("liquidate"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (
                liquidator.clone(),
                data.stable_token_denomination.clone(),
                vec![&env, depositor.clone()]
            )
                .into_val(&env),
        )]
    );

    // The depositor's vault should be removed from the protocol
    let vault_doesnt_exist_result = data
        .contract_client
        .try_get_vault(&depositor, &data.stable_token_denomination)
        .unwrap_err();

    assert_eq!(
        vault_doesnt_exist_result,
        Ok(Status::from_contract_error(50000))
    );

    // The liquidator should now have the collateral from the depositor
    let liquidator_collateral_balance =
        token::Client::new(&env, &data.collateral_token_client.contract_id).balance(&liquidator);

    assert_eq!(liquidator_collateral_balance, depositor_collateral);

    // The liquidator should have 0 stablecoins
    let liquidator_debt_balance =
        token::Client::new(&env, &data.stable_token_client.contract_id).balance(&liquidator);

    assert_eq!(liquidator_debt_balance, 0);

    // check currency stats has been updated correctly
    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .g_cy_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.tot_col, liquidator_collateral);
    assert_eq!(updated_currency_stats.tot_debt, liquidator_debt);
    assert_eq!(updated_currency_stats.tot_vaults, 1);

    // Check the only index is the one from the liquidator's vault
    let updated_indexes: Vec<i128> = data
        .contract_client
        .g_indexes(&data.stable_token_denomination);

    assert_eq!(
        updated_indexes,
        vec![
            &env,
            calculate_user_vault_index(liquidator_debt, liquidator_collateral)
        ]
    );
}
