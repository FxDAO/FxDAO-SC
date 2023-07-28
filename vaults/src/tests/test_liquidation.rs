#![cfg(test)]
extern crate std;

use crate::storage_types::{CurrencyStats, SCErrors, UserVault};
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_initial_state, InitialVariables, TestData,
};
use crate::utils::indexes::calculate_user_vault_index;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, token, vec, Address, Env, Error, IntoVal, Symbol, Vec};

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
        .set_currency_rate(&data.stable_token_denomination, &first_rate);

    let depositor: Address = Address::random(&env);
    let depositor_debt: i128 = 5_000_0000000;
    let depositor_collateral: i128 = 100_000_0000000;

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor, &depositor_collateral);

    let liquidator: Address = Address::random(&env);
    let liquidator_debt: i128 = 5_000_0000000;
    let liquidator_collateral: i128 = 500_000_0000000;

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&liquidator, &liquidator_collateral);

    token::Client::new(&env, &data.collateral_token_client.address).approve(
        &depositor,
        &data.contract_client.address,
        &9000000000000000,
        &200_000,
    );

    token::Client::new(&env, &data.collateral_token_client.address).approve(
        &liquidator,
        &data.contract_client.address,
        &9000000000000000,
        &200_000,
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
        .unwrap_err()
        .unwrap();

    // TODO: ENABLE THIS LATER
    // assert_eq!(
    //     cant_liquidate_error_result,
    //     SCErrors::UserVaultCantBeLiquidated.into()
    // );

    // We update the collateral price in order to put the depositor's vault below the min collateral ratio
    let second_rate: i128 = 531953;
    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &second_rate);

    token::Client::new(&env, &data.stable_token_client.address).approve(
        &liquidator,
        &data.contract_client.address,
        &9000000000000000,
        &200_000,
    );

    data.contract_client.liquidate(
        &liquidator,
        &data.stable_token_denomination,
        &vec![&env, depositor.clone()],
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths().first().unwrap(),
        &(
            liquidator.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    symbol_short!("liquidate"),
                    (
                        liquidator.clone(),
                        data.stable_token_denomination.clone(),
                        vec![&env, depositor.clone()]
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        data.stable_token_admin_client.address.clone(),
                        symbol_short!("transfer"),
                        (
                            liquidator.clone(),
                            data.stable_token_issuer.clone(),
                            5000_0000000i128
                        )
                            .into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )
    );

    // The depositor's vault should be removed from the protocol
    let vault_doesnt_exist_result = data
        .contract_client
        .try_get_vault(&depositor, &data.stable_token_denomination)
        .unwrap_err();

    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // assert_eq!(
    //     vault_doesnt_exist_result.unwrap(),
    //     SCErrors::UserVaultDoesntExist.into()
    // );

    // The liquidator should now have the collateral from the depositor
    let liquidator_collateral_balance =
        token::Client::new(&env, &data.collateral_token_client.address).balance(&liquidator);

    assert_eq!(liquidator_collateral_balance, depositor_collateral);

    // The liquidator should have 0 stablecoins
    let liquidator_debt_balance =
        token::Client::new(&env, &data.stable_token_client.address).balance(&liquidator);

    assert_eq!(liquidator_debt_balance, 0);

    // check currency stats has been updated correctly
    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.total_col, liquidator_collateral);
    assert_eq!(updated_currency_stats.total_debt, liquidator_debt);
    assert_eq!(updated_currency_stats.total_vaults, 1);

    // Check the only index is the one from the liquidator's vault
    let updated_indexes: Vec<i128> = data
        .contract_client
        .get_indexes(&data.stable_token_denomination);

    assert_eq!(
        updated_indexes,
        vec![
            &env,
            calculate_user_vault_index(liquidator_debt, liquidator_collateral)
        ]
    );
}

#[test]
fn test_vaults_to_liquidate() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let min_collateral_rate: i128 = 1_1000000;
    let opening_debt_amount: i128 = 1_0000000;
    let opening_collateral_rate: i128 = 1_1500000;

    data.contract_client.set_vault_conditions(
        &min_collateral_rate,
        &opening_debt_amount,
        &opening_collateral_rate,
        &data.stable_token_denomination,
    );

    let first_rate: i128 = 0_0958840;
    let second_rate: i128 = 0_0586660;

    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &first_rate);

    let depositor_1: Address = Address::random(&env);
    let depositor_2: Address = Address::random(&env);
    let depositor_3: Address = Address::random(&env);
    let depositor_4: Address = Address::random(&env);
    let depositor_5: Address = Address::random(&env);
    let depositor_collateral: i128 = 3000_0000000;

    for (i, depositor) in [
        depositor_1,
        depositor_2,
        depositor_3,
        depositor_4,
        depositor_5,
    ]
    .iter()
    .enumerate()
    {
        token::AdminClient::new(&env, &data.collateral_token_client.address)
            .mint(&depositor, &depositor_collateral);

        let debt_amount: i128;
        if i < 3 {
            debt_amount = 100_0000000;
        } else {
            debt_amount = 160_0000000;
        }

        token::Client::new(&env, &data.collateral_token_client.address).approve(
            &depositor,
            &data.contract_client.address,
            &9000000000000000,
            &200_000,
        );

        data.contract_client.new_vault(
            &depositor,
            &debt_amount,
            &depositor_collateral,
            &data.stable_token_denomination,
        );
    }

    let mut current_vaults_to_liquidate: Vec<UserVault> = data
        .contract_client
        .vaults_to_liquidate(&data.stable_token_denomination);

    assert_eq!(current_vaults_to_liquidate, vec![&env]);

    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &second_rate);

    current_vaults_to_liquidate = data
        .contract_client
        .vaults_to_liquidate(&data.stable_token_denomination);

    assert_eq!(current_vaults_to_liquidate.len(), 2);
}
