#![cfg(test)]

extern crate std;

use crate::storage::vaults::*;
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_allowance, set_initial_state, InitialVariables,
    TestData,
};

use crate::errors::SCErrors;
use crate::utils::indexes::calculate_user_vault_index;
use crate::utils::payments::calc_fee;
use num_integer::div_floor;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, token, Address, Env, IntoVal, Symbol, Vec};

#[test]
fn test_set_vault_conditions() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);

    data.contract_client.init(
        &data.contract_admin,
        &data.oracle_admin,
        &data.protocol_manager,
        &data.collateral_token_client.address,
        &data.stable_token_issuer,
        &data.treasury,
        &data.fee,
    );

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    // Check the admin is the one who call it
    assert_eq!(
        env.auths().first().unwrap(),
        &(
            data.contract_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    Symbol::new(&env, "set_vault_conditions"),
                    (
                        base_variables.min_col_rate.clone(),
                        base_variables.min_debt_creation.clone(),
                        base_variables.opening_col_rate.clone(),
                        data.stable_token_denomination.clone(),
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )
    );

    let vault_info = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(vault_info.min_col_rate, 11000000);
    assert_eq!(vault_info.min_debt_creation, 50000000000);
    assert_eq!(vault_info.opening_col_rate, 11500000);
}

#[test]
fn test_new_vault() {
    let env = Env::default();
    let data = create_base_data(&env);

    data.contract_client.init(
        &data.contract_admin,
        &data.oracle_admin,
        &data.protocol_manager,
        &data.collateral_token_client.address,
        &data.stable_token_issuer,
        &data.treasury,
        &data.fee,
    );

    let currency_price: u128 = 830124; // 0.0830124
    let depositor = Address::generate(&env);
    let initial_debt: u128 = 5_000_0000000; // USD 5000
    let collateral_amount: u128 = 90_347_8867088; // 90,347.8867088 XLM
    let contract_address: Address = data.contract_client.address.clone();

    let min_col_rate: u128 = 1_1000000;
    let min_debt_creation: u128 = 5000_0000000;
    let opening_col_rate: u128 = 1_1500000;

    token::Client::new(&env, &data.stable_token_client.address).approve(
        &data.stable_token_issuer,
        &contract_address,
        &90000000000000000000,
        &200_000,
    );

    token::StellarAssetClient::new(&env, &data.stable_token_client.address)
        .mint(&data.stable_token_issuer, &90000000000000000000);

    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let inactive_currency = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::None,
    //         &depositor,
    //         &initial_debt,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(inactive_currency, SCErrors::CurrencyDoesntExist.into());

    data.contract_client.create_currency(
        &data.stable_token_denomination,
        &data.stable_token_client.address,
    );

    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &currency_price);

    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let inactive_currency = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::None,
    //         &depositor,
    //         &initial_debt,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(inactive_currency, SCErrors::CurrencyIsInactive.into());

    data.contract_client
        .toggle_currency(&data.stable_token_denomination, &true);

    data.collateral_token_admin_client
        .mint(&depositor, &(collateral_amount as i128 * 2));

    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let vaults_info_not_started = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::None,
    //         &depositor,
    //         &initial_debt,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     vaults_info_not_started,
    //     SCErrors::VaultsInfoHasNotStarted.into()
    // );

    data.contract_client.set_vault_conditions(
        &min_col_rate,
        &min_debt_creation,
        &opening_col_rate,
        &data.stable_token_denomination,
    );

    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let invalid_initial_debt = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::None,
    //         &depositor,
    //         &10,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     invalid_initial_debt,
    //     SCErrors::InvalidInitialDebtAmount.into()
    // );

    // data.contract_client.new_vault(
    //     &depositor,
    //     &initial_debt,
    //     &collateral_amount,
    //     &data.stable_token_denomination,
    // );

    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let invalid_opening_col_ratio = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::None,
    //         &depositor,
    //         &collateral_amount,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     invalid_opening_col_ratio,
    //     SCErrors::InvalidOpeningCollateralRatio.into()
    // );

    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let prev_index_is_higher_error = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::Some(VaultKey {
    //             index: u128::MAX,
    //             account: Address::generate(&env),
    //             denomination: symbol_short!("usd"),
    //         }),
    //         &depositor,
    //         &initial_debt,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     prev_index_is_higher_error,
    //     SCErrors::InvalidPrevVaultIndex.into()
    // );

    // Fail if the Vault doesn't exist
    // TODO: UPDATE THIS ONCE SOROBAN FIX IT
    // let vault_doesnt_exist_error = data
    //     .contract_client
    //     .try_get_vault(&depositor, &data.stable_token_denomination)
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(vault_doesnt_exist_error, SCErrors::VaultDoesntExist.into());

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths().first().unwrap(),
        &(
            depositor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    symbol_short!("new_vault"),
                    (
                        OptionalVaultKey::None,
                        depositor.clone(),
                        initial_debt.clone(),
                        collateral_amount.clone(),
                        data.stable_token_denomination.clone(),
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            data.collateral_token_client.address.clone(),
                            symbol_short!("transfer"),
                            (
                                depositor.clone(),
                                data.contract_client.address.clone(),
                                (collateral_amount - calc_fee(&data.fee, &collateral_amount))
                                    .clone() as i128,
                            )
                                .into_val(&env),
                        )),
                        sub_invocations: std::vec![],
                    },
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            data.collateral_token_client.address.clone(),
                            symbol_short!("transfer"),
                            (
                                depositor.clone(),
                                data.treasury.clone(),
                                calc_fee(&data.fee, &collateral_amount) as i128,
                            )
                                .into_val(&env),
                        )),
                        sub_invocations: std::vec![],
                    },
                ],
            }
        )
    );

    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount - calc_fee(&data.fee, &collateral_amount)) as i128
    );

    assert_eq!(
        data.stable_token_client.balance(&depositor),
        initial_debt as i128
    );

    let vault_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    let user_vault: Vault = data
        .contract_client
        .get_vault(&depositor, &data.stable_token_denomination);

    assert_eq!(vault_info.total_vaults, 1);
    assert_eq!(
        vault_info.lowest_key,
        OptionalVaultKey::Some(VaultKey {
            index: user_vault.index.clone(),
            account: user_vault.account.clone(),
            denomination: user_vault.denomination.clone(),
        })
    );
    assert_eq!(vault_info.total_debt, initial_debt);
    assert_eq!(
        vault_info.total_col,
        collateral_amount - calc_fee(&data.fee, &collateral_amount)
    );

    assert_eq!(
        user_vault.index,
        div_floor(
            1000000000 * (collateral_amount - calc_fee(&data.fee, &collateral_amount)),
            initial_debt
        )
    );
    assert_eq!(
        user_vault.total_collateral,
        collateral_amount - calc_fee(&data.fee, &collateral_amount)
    );
    assert_eq!(user_vault.total_debt, initial_debt);

    // Should fail if user tries to create a new vault but already have one
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let vault_already_exist = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::None,
    //         &depositor,
    //         &initial_debt,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     &vault_already_exist,
    //     &SCErrors::UserAlreadyHasDenominationVault.into()
    // );

    let depositor_2 = Address::generate(&env);

    data.collateral_token_admin_client
        .mint(&depositor_2, &(collateral_amount as i128 * 2));

    // If there is already a lowest key, prev key cant not be None
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let prev_cant_be_none_error = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::None,
    //         &depositor_2,
    //         &initial_debt,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     &prev_cant_be_none_error,
    //     &SCErrors::PrevVaultCantBeNone.into()
    // );

    // If prev vault doesn't exist, fail
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let prev_doesnt_exist_error = data
    //     .contract_client
    //     .try_new_vault(
    //         &OptionalVaultKey::Some(VaultKey {
    //             denomination: data.stable_token_denomination.clone(),
    //             index: 1,
    //             account: Address::generate(&env),
    //         }),
    //         &depositor_2,
    //         &initial_debt,
    //         &collateral_amount,
    //         &data.stable_token_denomination,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     &prev_doesnt_exist_error,
    //     &SCErrors::PrevVaultDoesntExist.into()
    // );

    data.contract_client.new_vault(
        &vault_info.lowest_key,
        &depositor_2,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    assert_eq!(
        data.stable_token_client.balance(&depositor_2),
        initial_debt as i128
    );

    let updated_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    let second_user_vault = data
        .contract_client
        .get_vault(&depositor, &data.stable_token_denomination);

    assert_eq!(updated_vaults_info.total_vaults, 2);
    assert_eq!(updated_vaults_info.total_debt, initial_debt * 2);
    assert_eq!(
        updated_vaults_info.total_col,
        (collateral_amount - calc_fee(&data.fee, &collateral_amount)) * 2
    );

    assert_eq!(
        second_user_vault.index,
        div_floor(
            1000000000 * (collateral_amount - calc_fee(&data.fee, &collateral_amount)),
            initial_debt
        )
    );
    assert_eq!(
        second_user_vault.total_collateral,
        collateral_amount - calc_fee(&data.fee, &collateral_amount)
    );
    assert_eq!(second_user_vault.total_debt, initial_debt);
}

#[test]
fn test_multiple_vaults_same_values() {
    // TODO
}

#[test]
fn test_increase_collateral() {
    let env = Env::default();
    let data = create_base_data(&env);
    let base_variables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);
    let contract_address: Address = data.contract_client.address.clone();

    token::Client::new(&env, &data.stable_token_client.address).approve(
        &data.stable_token_issuer,
        &contract_address,
        &90000000000000000000,
        &200_000,
    );

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    data.contract_client.set_currency_rate(
        &data.stable_token_denomination,
        &base_variables.currency_price,
    );

    let depositor: Address = Address::generate(&env);

    data.collateral_token_admin_client
        .mint(&depositor, &(base_variables.collateral_amount as i128 * 2));

    data.stable_token_admin_client
        .mint(&contract_address, &(base_variables.initial_debt as i128));

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    // It should fail if the user doesn't have a Vault open
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let no_vault_created_error = data
    //     .contract_client
    //     .try_incr_col(
    //         &OptionalVaultKey::None,
    //         &VaultKey {
    //             index: 1,
    //             account: depositor.clone(),
    //             denomination: data.stable_token_denomination.clone(),
    //         },
    //         &OptionalVaultKey::None,
    //         &100_0000000,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(no_vault_created_error, SCErrors::VaultDoesntExist.into());

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor,
        &base_variables.initial_debt,
        &base_variables.collateral_amount,
        &data.stable_token_denomination,
    );

    let current_vault: Vault = data
        .contract_client
        .get_vault(&depositor, &data.stable_token_denomination);

    assert_eq!(
        &current_vault.total_collateral,
        &base_variables.collateral_amount_minus_fee,
    );

    let collateral_to_add: u128 = 100_0000000;

    data.contract_client.increase_collateral(
        &OptionalVaultKey::None,
        &VaultKey {
            index: current_vault.index.clone(),
            account: current_vault.account.clone(),
            denomination: current_vault.denomination.clone(),
        },
        &OptionalVaultKey::None,
        &collateral_to_add,
    );

    assert_eq!(
        env.auths().first().unwrap(),
        &(
            depositor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    Symbol::new(&env, "increase_collateral"),
                    (
                        OptionalVaultKey::None,
                        VaultKey {
                            index: current_vault.index.clone(),
                            account: current_vault.account.clone(),
                            denomination: current_vault.denomination.clone(),
                        },
                        OptionalVaultKey::None,
                        collateral_to_add.clone(),
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            data.collateral_token_client.address.clone(),
                            symbol_short!("transfer"),
                            (
                                depositor.clone(),
                                data.contract_client.address.clone(),
                                (collateral_to_add - calc_fee(&data.fee, &collateral_to_add))
                                    as i128,
                            )
                                .into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            data.collateral_token_client.address.clone(),
                            symbol_short!("transfer"),
                            (
                                depositor.clone(),
                                data.treasury.clone(),
                                calc_fee(&data.fee, &collateral_to_add) as i128,
                            )
                                .into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    }
                ],
            }
        )
    );

    let updated_vault: Vault = data
        .contract_client
        .get_vault(&depositor, &data.stable_token_denomination);

    assert_ne!(&current_vault.index, &updated_vault.index);
    assert_eq!(
        &updated_vault.total_collateral,
        &(current_vault.total_collateral
            + (collateral_to_add - calc_fee(&data.fee, &collateral_to_add)))
    );

    let vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(&vaults_info.total_vaults, &1);
    assert_eq!(&vaults_info.total_debt, &base_variables.initial_debt);
    assert_eq!(
        &vaults_info.total_col,
        &(base_variables.collateral_amount_minus_fee
            + (collateral_to_add - calc_fee(&data.fee, &collateral_to_add)))
    );

    let depositor_2: Address = Address::generate(&env);

    data.collateral_token_admin_client.mint(
        &depositor_2,
        &(base_variables.collateral_amount as i128 * 2),
    );

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_2,
        &base_variables.initial_debt,
        &base_variables.collateral_amount,
        &data.stable_token_denomination,
    );

    let depositor_3: Address = Address::generate(&env);

    data.collateral_token_admin_client.mint(
        &depositor_3,
        &(base_variables.collateral_amount as i128 * 2),
    );

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(VaultKey {
            index: calculate_user_vault_index(
                base_variables.initial_debt,
                base_variables.collateral_amount_minus_fee,
            ),
            denomination: data.stable_token_denomination.clone(),
            account: depositor_2.clone(),
        }),
        &depositor_3,
        &base_variables.initial_debt,
        &base_variables.collateral_amount,
        &data.stable_token_denomination,
    );

    // Currently this is the highest vault
    let vault_1: Vault = data
        .contract_client
        .get_vault(&depositor, &data.stable_token_denomination);

    // Currently this is the lowest vault
    let vault_2: Vault = data
        .contract_client
        .get_vault(&depositor_2, &data.stable_token_denomination);

    // Currently this is the middle vault
    let vault_3: Vault = data
        .contract_client
        .get_vault(&depositor_3, &data.stable_token_denomination);

    // If prev_key is None, the target Vault needs to be the lowest vault otherwise panic
    // TODO: FIX ONCE SOROBAN FIX IT
    // let none_must_be_the_lowest_error = data
    //     .contract_client
    //     .try_increase_collateral(
    //         &OptionalVaultKey::None,
    //         &VaultKey {
    //             index: vault_3.index.clone(),
    //             account: vault_3.account.clone(),
    //             denomination: vault_3.denomination.clone(),
    //         },
    //         &OptionalVaultKey::None,
    //         &collateral_to_add,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     none_must_be_the_lowest_error,
    //     SCErrors::PrevVaultCantBeNone.into(),
    // );

    // If the Next Key of the prev_vault we provide is None, it means is not this one so it panics
    // TODO: FIX ONCE SOROBAN FIX IT
    // let invalid_next_key_none = data
    //     .contract_client
    //     .try_increase_collateral(
    //         &OptionalVaultKey::Some(VaultKey {
    //             index: vault_1.index.clone(),
    //             account: vault_1.account.clone(),
    //             denomination: vault_1.denomination.clone(),
    //         }),
    //         &VaultKey {
    //             index: vault_3.index.clone(),
    //             account: vault_3.account.clone(),
    //             denomination: vault_3.denomination.clone(),
    //         },
    //         &OptionalVaultKey::None,
    //         &collateral_to_add,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     invalid_next_key_none,
    //     SCErrors::PrevVaultNextIndexIsInvalid.into(),
    // );

    // If the Next Key of the prev_vault we provide is not the target vault, it means the prev_vault is not the correct one
    // TODO: FIX ONCE SOROBAN FIX IT
    // let invalid_next_key_wrong = data
    //     .contract_client
    //     .try_increase_collateral(
    //         &OptionalVaultKey::Some(VaultKey {
    //             index: vault_2.index.clone(),
    //             account: vault_2.account.clone(),
    //             denomination: vault_2.denomination.clone(),
    //         }),
    //         &VaultKey {
    //             index: vault_1.index.clone(),
    //             account: vault_1.account.clone(),
    //             denomination: vault_1.denomination.clone(),
    //         },
    //         &OptionalVaultKey::None,
    //         &collateral_to_add,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     invalid_next_key_wrong,
    //     SCErrors::PrevVaultNextIndexIsInvalid.into(),
    // );

    data.contract_client.increase_collateral(
        &OptionalVaultKey::Some(VaultKey {
            index: vault_3.index.clone(),
            account: vault_3.account.clone(),
            denomination: vault_3.denomination.clone(),
        }),
        &VaultKey {
            index: vault_2.index.clone(),
            account: vault_2.account.clone(),
            denomination: vault_2.denomination.clone(),
        },
        &OptionalVaultKey::Some(VaultKey {
            index: vault_1.index.clone(),
            account: vault_1.account.clone(),
            denomination: vault_1.denomination.clone(),
        }),
        &(collateral_to_add * 3),
    );

    let updated_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(&updated_vaults_info.total_vaults, &3);
    assert_eq!(
        &updated_vaults_info.total_debt,
        &(base_variables.initial_debt * 3)
    );
    assert_eq!(
        &updated_vaults_info.total_col,
        &((base_variables.collateral_amount_minus_fee * 3)
            + ((collateral_to_add - calc_fee(&data.fee, &collateral_to_add)) * 4))
    );
}

#[test]
fn test_increase_debt() {
    let env: Env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);
    let contract_address: Address = data.contract_client.address.clone();

    token::Client::new(&env, &data.stable_token_client.address).approve(
        &data.stable_token_issuer,
        &contract_address,
        &90000000000000000000,
        &200_000,
    );

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    data.contract_client.set_currency_rate(
        &data.stable_token_denomination,
        &base_variables.currency_price,
    );

    data.collateral_token_admin_client.mint(
        &base_variables.depositor,
        &(base_variables.collateral_amount as i128 * 5),
    );

    data.stable_token_admin_client.mint(
        &base_variables.contract_address,
        &(base_variables.initial_debt as i128 * 5),
    );

    // It should fail if the user doesn't have a Vault open
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let no_vault_created_error = data
    //     .contract_client
    //     .try_increase_debt(
    //         &OptionalVaultKey::None,
    //         &VaultKey {
    //             index: 1,
    //             account: base_variables.depositor.clone(),
    //             denomination: data.stable_token_denomination.clone(),
    //         },
    //         &OptionalVaultKey::None,
    //         &100_0000000,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(no_vault_created_error, SCErrors::VaultDoesntExist.into());

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &base_variables.depositor,
        &base_variables.initial_debt,
        &(base_variables.collateral_amount * 2),
        &data.stable_token_denomination,
    );

    let current_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(current_vaults_info.total_vaults, 1);
    assert_eq!(current_vaults_info.total_debt, base_variables.initial_debt);
    assert_eq!(
        current_vaults_info.total_col,
        (base_variables.collateral_amount * 2)
            - calc_fee(&data.fee, &(base_variables.collateral_amount * 2))
    );
    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        base_variables.initial_debt as i128
    );

    data.contract_client.increase_debt(
        &OptionalVaultKey::None,
        &VaultKey {
            index: calculate_user_vault_index(
                base_variables.initial_debt.clone(),
                ((base_variables.collateral_amount * 2)
                    - calc_fee(&data.fee, &(base_variables.collateral_amount * 2)))
                .clone(),
            ),
            account: base_variables.depositor.clone(),
            denomination: data.stable_token_denomination.clone(),
        },
        &OptionalVaultKey::None,
        &base_variables.initial_debt,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths().first().unwrap(),
        &(
            base_variables.depositor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    Symbol::new(&env, "increase_debt"),
                    (
                        OptionalVaultKey::None,
                        VaultKey {
                            index: calculate_user_vault_index(
                                base_variables.initial_debt.clone(),
                                ((base_variables.collateral_amount * 2)
                                    - calc_fee(&data.fee, &(base_variables.collateral_amount * 2)))
                                .clone(),
                            ),
                            account: base_variables.depositor.clone(),
                            denomination: data.stable_token_denomination.clone(),
                        },
                        OptionalVaultKey::None,
                        base_variables.initial_debt,
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    let updated_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(updated_vaults_info.total_vaults, 1);
    assert_eq!(
        updated_vaults_info.total_debt,
        base_variables.initial_debt * 2
    );
    assert_eq!(
        updated_vaults_info.total_col,
        (base_variables.collateral_amount * 2)
            - calc_fee(&data.fee, &(base_variables.collateral_amount * 2))
    );

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        base_variables.initial_debt as i128 * 2
    );
}

#[test]
fn test_pay_debt() {
    let env: Env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    data.contract_client.set_currency_rate(
        &data.stable_token_denomination,
        &base_variables.currency_price,
    );

    data.collateral_token_admin_client.mint(
        &base_variables.depositor,
        &(base_variables.collateral_amount as i128 * 5),
    );

    data.stable_token_admin_client.mint(
        &base_variables.contract_address,
        &(base_variables.initial_debt as i128 * 5),
    );

    // It should fail if the user doesn't have a Vault open
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let no_vault_open_error = data
    //     .contract_client
    //     .try_pay_debt(
    //         &OptionalVaultKey::None,
    //         &VaultKey {
    //             index: calculate_user_vault_index(
    //                 base_variables.initial_debt.clone(),
    //                 base_variables.collateral_amount.clone(),
    //             ),
    //             account: base_variables.depositor.clone(),
    //             denomination: data.stable_token_denomination.clone(),
    //         },
    //         &OptionalVaultKey::None,
    //         &base_variables.initial_debt,
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(no_vault_open_error, SCErrors::VaultDoesntExist.into());

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &base_variables.depositor,
        &(base_variables.initial_debt * 2),
        &(base_variables.collateral_amount * 2),
        &data.stable_token_denomination,
    );

    let current_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(current_vaults_info.total_vaults, 1);
    assert_eq!(
        current_vaults_info.total_debt,
        base_variables.initial_debt * 2
    );
    assert_eq!(
        current_vaults_info.total_col,
        (base_variables.collateral_amount * 2)
            - calc_fee(&data.fee, &(base_variables.collateral_amount * 2))
    );
    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        (base_variables.initial_debt * 2) as i128
    );

    let mut vault: Vault = data
        .contract_client
        .get_vault(&base_variables.depositor, &data.stable_token_denomination);

    data.contract_client.pay_debt(
        &OptionalVaultKey::None,
        &VaultKey {
            index: vault.index.clone(),
            account: vault.account.clone(),
            denomination: vault.denomination.clone(),
        },
        &OptionalVaultKey::None,
        &base_variables.initial_debt,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths(),
        std::vec![(
            vault.account.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    symbol_short!("pay_debt"),
                    (
                        OptionalVaultKey::None,
                        VaultKey {
                            index: vault.index.clone(),
                            account: vault.account.clone(),
                            denomination: vault.denomination.clone(),
                        },
                        OptionalVaultKey::None,
                        base_variables.initial_debt.clone(),
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![
                    (AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            data.stable_token_client.address.clone(),
                            symbol_short!("transfer"),
                            (
                                vault.account.clone(),
                                data.contract_client.address.clone(),
                                base_variables.initial_debt.clone() as i128,
                            )
                                .into_val(&env),
                        )),
                        sub_invocations: std::vec![],
                    })
                ],
            }
        ),]
    );

    let updated_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(updated_vaults_info.total_vaults, 1);
    assert_eq!(updated_vaults_info.total_debt, base_variables.initial_debt,);
    assert_eq!(
        updated_vaults_info.total_col,
        (base_variables.collateral_amount * 2)
            - calc_fee(&data.fee, &(base_variables.collateral_amount * 2)),
    );

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        base_variables.initial_debt as i128
    );
    assert_eq!(
        data.collateral_token_client
            .balance(&base_variables.contract_address),
        ((base_variables.collateral_amount * 2)
            - calc_fee(&data.fee, &(base_variables.collateral_amount * 2))) as i128
    );

    vault = data
        .contract_client
        .get_vault(&base_variables.depositor, &data.stable_token_denomination);

    // If the vault will be below the min deb it should fail
    // TODO: FIX THIS ONCE SOROBAN FIX IT
    // let min_debt_invalid_error = data
    //     .contract_client
    //     .try_pay_debt(
    //         &OptionalVaultKey::None,
    //         &VaultKey {
    //             index: vault.index.clone(),
    //             account: vault.account.clone(),
    //             denomination: vault.denomination.clone(),
    //         },
    //         &OptionalVaultKey::None,
    //         &(base_variables.initial_debt / 2),
    //     )
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(
    //     min_debt_invalid_error,
    //     SCErrors::InvalidMinDebtAmount.into()
    // );

    data.contract_client.pay_debt(
        &OptionalVaultKey::None,
        &VaultKey {
            index: vault.index.clone(),
            account: vault.account.clone(),
            denomination: vault.denomination.clone(),
        },
        &OptionalVaultKey::None,
        &base_variables.initial_debt,
    );

    let final_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    assert_eq!(final_vaults_info.total_vaults, 0);
    assert_eq!(final_vaults_info.total_debt, 0);
    assert_eq!(final_vaults_info.total_col, 0);

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        0
    );
    assert_eq!(
        data.collateral_token_client
            .balance(&base_variables.contract_address),
        0
    );

    // We confirm the vault was removed from the storage
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    // let vault_removed_error = data
    //     .contract_client
    //     .try_get_vault(&base_variables.depositor, &data.stable_token_denomination)
    //     .unwrap_err()
    //     .unwrap();
    //
    // assert_eq!(vault_removed_error, SCErrors::VaultDoesntExist.into());
}

#[test]
fn get_vaults() {
    let env: Env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    data.contract_client.set_vault_conditions(
        &base_variables.min_col_rate,
        &base_variables.min_debt_creation,
        &base_variables.opening_col_rate,
        &data.stable_token_denomination,
    );

    data.contract_client.set_currency_rate(
        &data.stable_token_denomination,
        &base_variables.currency_price,
    );

    data.collateral_token_admin_client.mint(
        &base_variables.depositor,
        &(base_variables.collateral_amount as i128 * 5),
    );

    data.stable_token_admin_client.mint(
        &base_variables.contract_address,
        &(base_variables.initial_debt as i128 * 5),
    );

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &base_variables.depositor,
        &(base_variables.initial_debt * 2),
        &(base_variables.collateral_amount * 2),
        &data.stable_token_denomination,
    );

    let vault_to_validate: Vault = Vault {
        index: calculate_user_vault_index(
            base_variables.initial_debt * 2,
            (base_variables.collateral_amount * 2)
                - calc_fee(&data.fee, &(base_variables.collateral_amount * 2)),
        ),
        next_key: OptionalVaultKey::None,
        account: base_variables.depositor.clone(),
        total_debt: base_variables.initial_debt * 2,
        total_collateral: (base_variables.collateral_amount * 2)
            - calc_fee(&data.fee, &(base_variables.collateral_amount * 2)),
        denomination: data.stable_token_denomination.clone(),
    };

    let vault_from_basic: Vault = data
        .contract_client
        .get_vault(&base_variables.depositor, &data.stable_token_denomination);

    assert_eq!(&vault_from_basic, &vault_to_validate);

    let vault_from_key: Vault = data.contract_client.get_vault_from_key(&VaultKey {
        index: vault_from_basic.index.clone(),
        account: vault_from_basic.account.clone(),
        denomination: vault_from_basic.denomination.clone(),
    });

    assert_eq!(&vault_from_key, &vault_to_validate);

    let vaults: Vec<Vault> = data.contract_client.get_vaults(
        &OptionalVaultKey::None,
        &data.stable_token_denomination,
        &1,
        &false,
    );

    assert_eq!(&vaults.first().unwrap(), &vault_from_basic);
}
