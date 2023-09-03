#![cfg(test)]
extern crate std;

use crate::storage::vaults::*;
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_allowance, set_initial_state, InitialVariables,
    TestData,
};
use crate::utils::indexes::calculate_user_vault_index;
use num_integer::div_floor;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, token, Address, Env, IntoVal, Symbol};

/// It tests the redeem method, this must comply with the next behaviour:
///
/// 1.- The user redeeming the stables must receive the expected collateral
///
/// 2.- The stables must be sent to the issuer (burned in the case of a classic asset)
///
/// 3.- The lowest key must be updated
///
/// 4.- Vault must be removed
///
/// 5.- The owner of the Vault must receive the remaining of the collateral
#[test]
fn test_redeem() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let rate: u128 = 931953;
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
    let depositor_1_collateral: u128 = 30000000000;
    let depositor_1_debt: u128 = 100_0000000;
    let depositor_1_index: u128 = 30000000000;
    let depositor_2: Address = Address::random(&env);
    let depositor_2_collateral: u128 = 30000000000;
    let depositor_2_debt: u128 = 1500000000;
    let depositor_2_index: u128 = 200_00000000;
    let depositor_3: Address = Address::random(&env);
    let depositor_3_collateral: u128 = 30000000000;
    let depositor_3_debt: u128 = 1250000000;
    let depositor_3_index: u128 = 240_00000000;
    let depositor_4: Address = Address::random(&env);
    let depositor_4_collateral: u128 = 30000000000;
    let depositor_4_debt: u128 = 1200000000;
    let depositor_4_index: u128 = 250_00000000;

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_1, &(depositor_1_collateral as i128));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_1,
        &depositor_1_debt,
        &depositor_1_collateral,
        &data.stable_token_denomination,
    );

    let depositor_1_vault: Vault = data
        .contract_client
        .get_vault(&depositor_1, &data.stable_token_denomination);

    assert_eq!(depositor_1_vault.index, depositor_1_index);

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_2, &(depositor_2_collateral as i128));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_2,
        &depositor_2_debt,
        &depositor_2_collateral,
        &data.stable_token_denomination,
    );

    let depositor_2_vault: Vault = data
        .contract_client
        .get_vault(&depositor_2, &data.stable_token_denomination);

    assert_eq!(depositor_2_vault.index, depositor_2_index);

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_3, &(depositor_3_collateral as i128));

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(VaultKey {
            index: depositor_2_index.clone(),
            account: depositor_2.clone(),
            denomination: data.stable_token_denomination.clone(),
        }),
        &depositor_3,
        &depositor_3_debt,
        &depositor_3_collateral,
        &data.stable_token_denomination,
    );

    let depositor_3_vault: Vault = data
        .contract_client
        .get_vault(&depositor_3, &data.stable_token_denomination);

    assert_eq!(depositor_3_vault.index, depositor_3_index);

    token::AdminClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_4, &(depositor_4_collateral as i128));

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(VaultKey {
            index: depositor_3_index.clone(),
            account: depositor_3.clone(),
            denomination: data.stable_token_denomination.clone(),
        }),
        &depositor_4,
        &depositor_4_debt,
        &depositor_4_collateral,
        &data.stable_token_denomination,
    );

    let depositor_4_vault: Vault = data
        .contract_client
        .get_vault(&depositor_4, &data.stable_token_denomination);

    assert_eq!(depositor_4_vault.index, depositor_4_index);

    // Send all stable tokens to one single account which is the one that will redeem collateral

    let redeem_user: Address = Address::random(&env);

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_1,
        &redeem_user,
        &50_0000000,
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_2,
        &redeem_user,
        &50_0000000,
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_3,
        &redeem_user,
        &50_0000000,
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_4,
        &redeem_user,
        &50_0000000,
    );

    // Before redeeming
    let vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(vaults_info.total_vaults, 4);
    assert_eq!(
        vaults_info.total_debt,
        depositor_1_debt + depositor_2_debt + depositor_3_debt + depositor_4_debt
    );
    assert_eq!(
        vaults_info.total_col,
        depositor_1_collateral
            + depositor_2_collateral
            + depositor_3_collateral
            + depositor_4_collateral
    );

    data.contract_client
        .redeem(&redeem_user, &data.stable_token_denomination);

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths().first().unwrap(),
        &(
            redeem_user.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    symbol_short!("redeem"),
                    (redeem_user.clone(), data.stable_token_denomination.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        data.stable_token_client.address.clone(),
                        symbol_short!("transfer"),
                        (
                            redeem_user.clone(),
                            data.contract_client.address.clone(),
                            depositor_2_debt.clone() as i128,
                        )
                            .into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )
    );

    assert_eq!(
        data.stable_token_client.balance(&redeem_user) as u128,
        200_0000000 - depositor_2_debt
    );

    let collateral_withdrew: u128 = div_floor(depositor_2_debt * 10000000, rate);
    assert_eq!(
        data.collateral_token_client.balance(&redeem_user) as u128,
        collateral_withdrew
    );

    assert_eq!(
        data.collateral_token_client.balance(&depositor_2) as u128,
        depositor_2_collateral - collateral_withdrew,
    );

    // After redeeming
    let vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(vaults_info.total_vaults, 3);
    assert_eq!(
        vaults_info.total_debt,
        depositor_1_debt + depositor_3_debt + depositor_4_debt
    );
    assert_eq!(
        vaults_info.total_col,
        depositor_1_collateral + depositor_3_collateral + depositor_4_collateral
    );
}
