#![cfg(test)]
extern crate std;

use crate::errors::SCErrors;
use crate::storage::vaults::*;
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_initial_state, update_oracle_price,
    InitialVariables, TestData,
};
use crate::utils::indexes::calculate_user_vault_index;
use crate::utils::payments::calc_fee;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, token, vec, Address, Env, Error, IntoVal, Symbol, Vec};

/// It test a simple liquidation
/// The vault must be removed and the collateral sent to the liquidator
/// Currency stats must be updated
#[test]
fn test_liquidation() {
    let env = Env::default();
    env.mock_all_auths();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let first_rate: u128 = 931953;
    update_oracle_price(
        &env,
        &data.oracle_contract_client,
        &data.stable_token_denomination,
        &(first_rate as i128),
    );

    let depositor: Address = Address::generate(&env);
    let depositor_debt: u128 = 5_000_0000000;
    let depositor_collateral: u128 = 100_000_0000000;

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor, &(depositor_collateral as i128));

    let liquidator: Address = Address::generate(&env);
    let liquidator_debt: u128 = 5_000_0000000;
    let liquidator_collateral: u128 = 500_000_0000000;

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
        .mint(&liquidator, &(liquidator_collateral as i128));

    // Create both vaults

    let depositor_key: VaultKey = VaultKey {
        index: calculate_user_vault_index(
            depositor_debt.clone(),
            (depositor_collateral - calc_fee(&data.fee, &depositor_collateral)).clone(),
        ),
        account: depositor.clone(),
        denomination: data.stable_token_denomination.clone(),
    };
    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor,
        &depositor_debt,
        &depositor_collateral,
        &data.stable_token_denomination,
    );
    let depositor_vault: Vault = data
        .contract_client
        .get_vault(&depositor, &data.stable_token_denomination);

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(depositor_key.clone()),
        &liquidator,
        &liquidator_debt,
        &liquidator_collateral,
        &data.stable_token_denomination,
    );

    // It should throw an error because the vault can't be liquidated yet
    let cant_liquidate_error_result = data
        .contract_client
        .try_liquidate(&liquidator, &data.stable_token_denomination, &1)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        cant_liquidate_error_result,
        SCErrors::NotEnoughVaultsToLiquidate.into()
    );

    // We update the collateral price in order to put the depositor's vault below the min collateral ratio
    let second_rate: u128 = 531953;
    update_oracle_price(
        &env,
        &data.oracle_contract_client,
        &data.stable_token_denomination,
        &(second_rate as i128),
    );

    let current_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(&current_vaults_info.total_vaults, &2);

    data.contract_client
        .liquidate(&liquidator, &data.stable_token_denomination, &1u32);

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths(),
        std::vec![(
            liquidator.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    symbol_short!("liquidate"),
                    (
                        liquidator.clone(),
                        data.stable_token_denomination.clone(),
                        1u32
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        data.stable_token_admin_client.address.clone(),
                        symbol_short!("burn"),
                        (liquidator.clone(), depositor_debt as i128).into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )]
    );

    // The depositor's vault should be removed from the protocol
    let vault_doesnt_exist_result = data
        .contract_client
        .try_get_vault(&depositor, &data.stable_token_denomination)
        .unwrap_err();

    assert_eq!(
        vault_doesnt_exist_result.unwrap(),
        SCErrors::VaultDoesntExist.into()
    );

    // The liquidator should now have the collateral from the depositor
    let liquidator_collateral_balance =
        token::Client::new(&env, &data.collateral_token_client.address).balance(&liquidator)
            as u128;

    let deposited_collateral: u128 =
        depositor_collateral - calc_fee(&data.fee, &depositor_collateral);

    assert_eq!(
        liquidator_collateral_balance,
        deposited_collateral - calc_fee(&data.fee, &deposited_collateral)
    );

    // The liquidator should have 0 stablecoins
    let liquidator_debt_balance =
        token::Client::new(&env, &data.stable_token_client.address).balance(&liquidator);

    assert_eq!(liquidator_debt_balance, 0);

    // check currency stats has been updated correctly
    let updated_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(
        updated_vaults_info.total_col,
        liquidator_collateral - calc_fee(&data.fee, &liquidator_collateral)
    );
    assert_eq!(updated_vaults_info.total_debt, liquidator_debt);
    assert_eq!(updated_vaults_info.total_vaults, 1);

    // Check the only index is the one from the liquidator's vault
    let current_vaults: Vec<Vault> = data.contract_client.get_vaults(
        &OptionalVaultKey::None,
        &data.stable_token_denomination,
        &2u32,
        &true,
    );

    assert_eq!(current_vaults, Vec::new(&env));
}

#[test]
fn test_vaults_to_liquidate() {
    let env = Env::default();
    env.mock_all_auths();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let min_collateral_rate: u128 = 1_1000000;
    let opening_debt_amount: u128 = 1_0000000;
    let opening_collateral_rate: u128 = 1_1500000;

    data.contract_client.set_vault_conditions(
        &min_collateral_rate,
        &opening_debt_amount,
        &opening_collateral_rate,
        &data.stable_token_denomination,
    );

    let first_rate: u128 = 0_0958840;
    let second_rate: u128 = 0_0586660;

    update_oracle_price(
        &env,
        &data.oracle_contract_client,
        &data.stable_token_denomination,
        &(first_rate as i128),
    );

    let depositor_1: Address = Address::generate(&env);
    let depositor_2: Address = Address::generate(&env);
    let depositor_3: Address = Address::generate(&env);
    let depositor_4: Address = Address::generate(&env);
    let depositor_5: Address = Address::generate(&env);
    let depositor_collateral: u128 = 3000_0000000;
    let first_debt_amount: u128 = 100_0000000;
    let second_debt_amount: u128 = 160_0000000;

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_1, &(depositor_collateral as i128));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_1,
        &first_debt_amount,
        &depositor_collateral,
        &data.stable_token_denomination,
    );

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_2, &(depositor_collateral as i128));

    data.contract_client.new_vault(
        &data
            .contract_client
            .get_vaults_info(&data.stable_token_denomination)
            .lowest_key,
        &depositor_2,
        &first_debt_amount,
        &depositor_collateral,
        &data.stable_token_denomination,
    );

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_3, &(depositor_collateral as i128));

    data.contract_client.new_vault(
        &data
            .contract_client
            .get_vaults_info(&data.stable_token_denomination)
            .lowest_key,
        &depositor_3,
        &first_debt_amount,
        &depositor_collateral,
        &data.stable_token_denomination,
    );

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_4, &(depositor_collateral as i128));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_4,
        &second_debt_amount,
        &depositor_collateral,
        &data.stable_token_denomination,
    );

    token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
        .mint(&depositor_5, &(depositor_collateral as i128));

    data.contract_client.new_vault(
        &data
            .contract_client
            .get_vaults_info(&data.stable_token_denomination)
            .lowest_key,
        &depositor_5,
        &second_debt_amount,
        &depositor_collateral,
        &data.stable_token_denomination,
    );

    let mut current_vaults_to_liquidate: Vec<Vault> = data.contract_client.get_vaults(
        &OptionalVaultKey::None,
        &data.stable_token_denomination,
        &5u32,
        &true,
    );

    assert_eq!(current_vaults_to_liquidate, vec![&env]);

    update_oracle_price(
        &env,
        &data.oracle_contract_client,
        &data.stable_token_denomination,
        &(second_rate as i128),
    );

    current_vaults_to_liquidate = data.contract_client.get_vaults(
        &OptionalVaultKey::None,
        &data.stable_token_denomination,
        &5u32,
        &true,
    );

    assert_eq!(current_vaults_to_liquidate.len(), 2);
}
