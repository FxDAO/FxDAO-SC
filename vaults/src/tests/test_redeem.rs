#![cfg(test)]
extern crate std;

use crate::storage_types::{CurrencyStats, UserVault};
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_allowance, set_initial_state, InitialVariables,
    TestData,
};
use crate::utils::vaults::calculate_user_vault_index;
use num_integer::div_floor;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, token, Address, Env, IntoVal, Symbol};

/// It tests the redeem method, this must comply with the next behaviour:
///
/// 1.- The user redeeming the stables must receive the expected collateral
///
/// 2.- The stables must be sent to the issuer (burned in the case of a classic asset)
///
/// 3.- The indexes list must be updated in the correct order
///
/// 4.- vaults with index must be updated for all of the vaults
///
/// 5.- vaults stats must be updated
#[test]
fn test_redeem() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let rate: i128 = 931953;
    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &rate);

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &1000000000,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    // Prepare and test the index of all depositors

    let depositor_1: Address = Address::random(&env);
    let depositor_1_collateral: i128 = 30000000000;
    let depositor_1_debt: i128 = 1000000000;
    let depositor_1_index: i128 = 30000000000;
    let depositor_2: Address = Address::random(&env);
    let depositor_2_collateral: i128 = 30000000000;
    let depositor_2_debt: i128 = 1500000000;
    let depositor_2_index: i128 = 20000000000;
    let depositor_3: Address = Address::random(&env);
    let depositor_3_collateral: i128 = 30000000000;
    let depositor_3_debt: i128 = 1250000000;
    let depositor_3_index: i128 = 24000000000;
    let depositor_4: Address = Address::random(&env);
    let depositor_4_collateral: i128 = 30000000000;
    let depositor_4_debt: i128 = 1200000000;
    let depositor_4_index: i128 = 25000000000;

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_1, &depositor_1_collateral);

    set_allowance(&env, &data, &depositor_1);

    data.contract_client.new_vault(
        &depositor_1,
        &depositor_1_debt,
        &depositor_1_collateral,
        &data.stable_token_denomination,
    );

    let depositor_1_vault: UserVault = data
        .contract_client
        .get_vault(&depositor_1, &data.stable_token_denomination);

    assert_eq!(depositor_1_vault.index, depositor_1_index);

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_2, &depositor_2_collateral);

    set_allowance(&env, &data, &depositor_2);

    data.contract_client.new_vault(
        &depositor_2,
        &depositor_2_debt,
        &depositor_2_collateral,
        &data.stable_token_denomination,
    );

    let depositor_2_vault: UserVault = data
        .contract_client
        .get_vault(&depositor_2, &data.stable_token_denomination);

    assert_eq!(depositor_2_vault.index, depositor_2_index);

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_3, &depositor_3_collateral);

    set_allowance(&env, &data, &depositor_3);

    data.contract_client.new_vault(
        &depositor_3,
        &depositor_3_debt,
        &depositor_3_collateral,
        &data.stable_token_denomination,
    );

    let depositor_3_vault: UserVault = data
        .contract_client
        .get_vault(&depositor_3, &data.stable_token_denomination);

    assert_eq!(depositor_3_vault.index, depositor_3_index);

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_4, &depositor_4_collateral);

    set_allowance(&env, &data, &depositor_4);

    data.contract_client.new_vault(
        &depositor_4,
        &depositor_4_debt,
        &depositor_4_collateral,
        &data.stable_token_denomination,
    );

    let depositor_4_vault: UserVault = data
        .contract_client
        .get_vault(&depositor_4, &data.stable_token_denomination);

    assert_eq!(depositor_4_vault.index, depositor_4_index);

    // Send all stable tokens to one single account which is the one that will redeem collateral

    let redeem_user: Address = Address::random(&env);

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_1,
        &redeem_user,
        &500000000,
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_2,
        &redeem_user,
        &500000000,
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_3,
        &redeem_user,
        &500000000,
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_4,
        &redeem_user,
        &500000000,
    );

    set_allowance(&env, &data, &redeem_user);

    let amount_to_redeem: i128 =
        token::Client::new(&env, &data.stable_token_client.address).balance(&redeem_user);

    assert_eq!(amount_to_redeem, 200_0000000);

    // Before redeeming
    let currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(currency_stats.total_vaults, 4);
    assert_eq!(currency_stats.total_debt, 495_0000000);
    assert_eq!(currency_stats.total_col, 12000_0000000);

    data.contract_client.redeem(
        &redeem_user,
        &amount_to_redeem,
        &data.stable_token_denomination,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths().first().unwrap(),
        &(
            redeem_user.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    symbol_short!("redeem"),
                    (
                        redeem_user.clone(),
                        amount_to_redeem.clone(),
                        data.stable_token_denomination.clone()
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        data.stable_token_client.address.clone(),
                        symbol_short!("transfer"),
                        (
                            redeem_user.clone(),
                            data.stable_token_issuer.clone(),
                            amount_to_redeem.clone(),
                        )
                            .into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )
    );

    // We check the results after redeeming
    let remaining_stable_amount_balance =
        token::Client::new(&env, &data.stable_token_client.address).balance(&redeem_user);
    let collateral_redeemed =
        token::Client::new(&env, &data.collateral_token_client.address).balance(&redeem_user);

    assert_eq!(remaining_stable_amount_balance, 0);
    assert_eq!(
        collateral_redeemed,
        div_floor(amount_to_redeem * 10000000, rate)
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    // Check the currency stats were updated correctly
    assert_eq!(updated_currency_stats.total_vaults, 3);
    assert_eq!(updated_currency_stats.total_debt, 295_0000000);
    assert_eq!(
        updated_currency_stats.total_col,
        12000_0000000 - depositor_2_collateral - div_floor(500000000 * 10000000, rate)
    );

    // Check the depositor whose vault was closed received his extra collateral
    let depositor_2_collateral_balance: i128 =
        token::Client::new(&env, &data.collateral_token_client.address).balance(&depositor_2);

    assert_eq!(
        depositor_2_collateral_balance,
        depositor_2_collateral - div_floor(depositor_2_debt * 10000000, rate)
    );

    // We confirm the depositor_2's vault was removed
    assert!(&data
        .contract_client
        .try_get_vault(&depositor_2, &data.stable_token_denomination)
        .is_err());

    // Check that depositor_3's vault data was updated
    let depositor_3_vault: UserVault = data
        .contract_client
        .get_vault(&depositor_3, &data.stable_token_denomination);

    assert_eq!(depositor_3_vault.total_debt, depositor_3_debt - 500000000);
    assert_eq!(
        depositor_3_vault.total_col,
        depositor_3_collateral - div_floor(500000000 * 10000000, rate)
    );
    assert_eq!(
        depositor_3_vault.index,
        calculate_user_vault_index(depositor_3_vault.total_debt, depositor_3_vault.total_col)
    );

    // TODO: Make sure every case is tested later
}
