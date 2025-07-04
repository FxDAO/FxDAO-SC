#![cfg(test)]
extern crate std;

use crate::errors::SCErrors;
use crate::storage::vaults::*;
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_initial_state, update_oracle_price,
    InitialVariables, TestData,
};
use crate::utils::payments::calc_fee;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, token, Address, Env, IntoVal};

// It tests the redeem method, this must comply with the next behaviour:
//
// 1.- The user redeeming the stables must receive the expected collateral
//
// 2.- The stables must be sent to the issuer (burned in the case of a classic asset)
//
// 3.- The lowest key must be updated
//
// 4.- Vault must be removed
//
// 5.- The owner of the Vault must receive the remaining of the collateral
#[test]
fn test_redeem() {
    let env = Env::default();
    env.mock_all_auths();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let rate: u128 = 931953;
    update_oracle_price(
        &env,
        &data.oracle_contract_client,
        &data.stable_token_denomination,
        &(rate as i128),
    );

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &1000000000,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    // Prepare and test the index of all depositors

    let depositor_1: Address = Address::generate(&env);
    let depositor_1_collateral: u128 = 3000_0000000;
    let depositor_1_debt: u128 = 100_0000000;
    let depositor_1_index: u128 = 2985_0000000;
    let depositor_2: Address = Address::generate(&env);
    let depositor_2_collateral: u128 = 3000_0000000;
    let depositor_2_debt: u128 = 150_0000000;
    let depositor_2_index: u128 = 1990_0000000;
    let depositor_3: Address = Address::generate(&env);
    let depositor_3_collateral: u128 = 3000_0000000;
    let depositor_3_debt: u128 = 125_0000000;
    let depositor_3_index: u128 = 2388_0000000;
    let depositor_4: Address = Address::generate(&env);
    let depositor_4_collateral: u128 = 3000_0000000;
    let depositor_4_debt: u128 = 1200000000;
    let depositor_4_index: u128 = 2487_5000000;

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
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

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
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

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
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

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
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

    let redeem_user: Address = Address::generate(&env);

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_1,
        &redeem_user,
        &(depositor_1_debt as i128),
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_2,
        &redeem_user,
        &(depositor_2_debt as i128),
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_3,
        &redeem_user,
        &(depositor_3_debt as i128),
    );

    token::Client::new(&env, &data.stable_token_client.address).transfer(
        &depositor_4,
        &redeem_user,
        &(depositor_4_debt as i128),
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
        (depositor_1_collateral - calc_fee(&data.fee, &depositor_1_collateral))
            + (depositor_2_collateral - calc_fee(&data.fee, &depositor_2_collateral))
            + (depositor_3_collateral - calc_fee(&data.fee, &depositor_3_collateral))
            + (depositor_4_collateral - calc_fee(&data.fee, &depositor_4_collateral))
    );

    data.contract_client.redeem(
        &redeem_user,
        &data.stable_token_denomination,
        &OptionalVaultKey::None,
        &depositor_2_debt,
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
                        data.stable_token_denomination.clone(),
                        OptionalVaultKey::None,
                        depositor_2_debt.clone()
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        data.stable_token_client.address.clone(),
                        symbol_short!("burn"),
                        (redeem_user.clone(), depositor_2_debt.clone() as i128,).into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )
    );

    assert_eq!(
        data.stable_token_client.balance(&redeem_user) as u128,
        depositor_1_debt + depositor_3_debt + depositor_4_debt
    );

    let collateral_withdrew: u128 = (depositor_2_debt * 1_0000000) / rate;

    assert_eq!(
        data.collateral_token_client.balance(&redeem_user) as u128,
        collateral_withdrew - calc_fee(&100000, &collateral_withdrew)
    );

    assert_eq!(
        data.collateral_token_client.balance(&depositor_2) as u128,
        (depositor_2_collateral - calc_fee(&data.fee, &depositor_2_collateral))
            - collateral_withdrew
            + (calc_fee(&100000, &collateral_withdrew) / 2),
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
        (depositor_1_collateral - calc_fee(&data.fee, &depositor_1_collateral))
            + (depositor_3_collateral - calc_fee(&data.fee, &depositor_3_collateral))
            + (depositor_4_collateral - calc_fee(&data.fee, &depositor_4_collateral))
    );

    let invalid_min_debt_amount_error = data
        .contract_client
        .try_redeem(
            &redeem_user,
            &data.stable_token_denomination,
            &OptionalVaultKey::None,
            &(depositor_3_debt - 1),
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        invalid_min_debt_amount_error,
        SCErrors::InvalidMinDebtAmount.into()
    );

    // We are going to redeem 25_0000000 from depositor_3's vault
    // this will move the lowest key to the second place in the list after depositor_4's vault

    data.contract_client.redeem(
        &redeem_user,
        &data.stable_token_denomination,
        &OptionalVaultKey::Some(VaultKey {
            index: depositor_4_index,
            account: depositor_4,
            denomination: data.stable_token_denomination.clone(),
        }),
        &25_0000000,
    );

    // We check if the lowest vault is now the 4's depositor and its updated values
    let lowest_vault = data
        .contract_client
        .get_vaults(
            &OptionalVaultKey::None,
            &data.stable_token_denomination,
            &1,
            &false,
        )
        .get(0)
        .unwrap();

    assert_eq!(lowest_vault.account, depositor_4_vault.account);

    let updated_depositor_3_vault: Vault = data
        .contract_client
        .get_vault(&depositor_3, &data.stable_token_denomination);

    assert_eq!(
        updated_depositor_3_vault.total_debt,
        depositor_3_vault.total_debt - 25_0000000,
    );

    assert_eq!(
        updated_depositor_3_vault.total_collateral,
        depositor_3_vault.total_collateral - ((25_0000000 * 1_0000000) / rate),
    );

    assert_eq!(
        data.contract_client
            .get_vaults_info(&data.stable_token_denomination)
            .total_col,
        data.collateral_token_client
            .balance(&data.contract_client.address) as u128,
    );
}
