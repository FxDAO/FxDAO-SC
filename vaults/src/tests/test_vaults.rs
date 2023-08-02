#![cfg(test)]

extern crate std;

use crate::storage::storage_types::*;
use crate::storage::vaults::*;
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_allowance, set_initial_state, InitialVariables,
    TestData,
};

use num_integer::div_floor;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, token, Address, Env, IntoVal, Symbol, Vec};

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
    );

    let currency_price: i128 = 830124; // 0.0830124
    let depositor = Address::random(&env);
    let initial_debt: i128 = 5_000_0000000; // USD 5000
    let collateral_amount: i128 = 90_347_8867088; // 90,347.8867088 XLM
    let contract_address: Address = data.contract_client.address.clone();

    let min_col_rate: i128 = 1_1000000;
    let min_debt_creation: i128 = 5000_0000000;
    let opening_col_rate: i128 = 1_1500000;

    token::Client::new(&env, &data.stable_token_client.address).approve(
        &data.stable_token_issuer,
        &contract_address,
        &90000000000000000000,
        &200_000,
    );

    token::AdminClient::new(&env, &data.stable_token_client.address)
        .mint(&data.stable_token_issuer, &90000000000000000000);

    set_allowance(&env, &data, &depositor);

    // If the method is called before before the currency is active it should fail
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    assert!(data
        .contract_client
        .try_new_vault(
            &depositor,
            &initial_debt,
            &collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.create_currency(
        &data.stable_token_denomination,
        &data.stable_token_client.address,
    );

    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &currency_price);

    data.contract_client
        .toggle_currency(&data.stable_token_denomination, &true);

    data.collateral_token_admin_client
        .mint(&depositor, &(collateral_amount * 2));

    // If the method is called before protocol state is set it should fail
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    assert!(data
        .contract_client
        .try_new_vault(
            &depositor,
            &initial_debt,
            &collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.set_vault_conditions(
        &min_col_rate,
        &min_debt_creation,
        &opening_col_rate,
        &data.stable_token_denomination,
    );

    data.contract_client.new_vault(
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
                        depositor.clone(),
                        initial_debt.clone(),
                        collateral_amount.clone(),
                        data.stable_token_denomination.clone(),
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        data.collateral_token_client.address.clone(),
                        symbol_short!("transfer"),
                        (
                            depositor.clone(),
                            data.contract_client.address.clone(),
                            collateral_amount.clone(),
                        )
                            .into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )
    );

    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount)
    );
    assert_eq!(data.stable_token_client.balance(&depositor), (initial_debt));

    let currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    let user_vault = data
        .contract_client
        .get_vault(&depositor, &data.stable_token_denomination);

    let indexes_list: Vec<i128> = data
        .contract_client
        .get_indexes(&data.stable_token_denomination);

    assert_eq!(currency_stats.total_vaults, 1);
    assert_eq!(currency_stats.total_debt, initial_debt);
    assert_eq!(currency_stats.total_col, collateral_amount);

    assert_eq!(
        user_vault.index,
        div_floor(1000000000 * collateral_amount, initial_debt)
    );
    assert_eq!(user_vault.total_col, collateral_amount);
    assert_eq!(user_vault.total_debt, initial_debt);

    assert_eq!(
        indexes_list.first().unwrap(),
        div_floor(1000000000 * collateral_amount, initial_debt)
    );
    assert_eq!(indexes_list.iter().len(), 1 as usize);

    // Should fail if user tries to create a new vault but already have one
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    assert!(data
        .contract_client
        .try_new_vault(
            &depositor,
            &initial_debt,
            &collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    let depositor_2 = Address::random(&env);

    data.collateral_token_admin_client
        .mint(&depositor_2, &(collateral_amount * 2));

    set_allowance(&env, &data, &depositor_2);

    data.contract_client.new_vault(
        &depositor_2,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    assert_eq!(
        data.stable_token_client.balance(&depositor_2),
        (initial_debt)
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    let second_user_vault = data
        .contract_client
        .get_vault(&depositor, &data.stable_token_denomination);

    let new_indexes_list: Vec<i128> = data
        .contract_client
        .get_indexes(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.total_vaults, 2);
    assert_eq!(updated_currency_stats.total_debt, initial_debt * 2);
    assert_eq!(updated_currency_stats.total_col, collateral_amount * 2);

    assert_eq!(
        second_user_vault.index,
        div_floor(1000000000 * collateral_amount, initial_debt)
    );
    assert_eq!(second_user_vault.total_col, collateral_amount);
    assert_eq!(second_user_vault.total_debt, initial_debt);

    assert_eq!(
        new_indexes_list.first().unwrap(),
        div_floor(1000000000 * collateral_amount, initial_debt)
    );
    assert_eq!(new_indexes_list.iter().len(), 1 as usize);
}

#[test]
fn test_increase_collateral() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let depositor = Address::random(&env);
    let initial_debt: i128 = 50000000000;
    let collateral_amount: i128 = 50000000000;
    let contract_address: Address = data.contract_client.address.clone();

    let min_col_rate: i128 = 11000000;
    let min_debt_creation: i128 = 50000000000;
    let opening_col_rate: i128 = 11500000;

    data.collateral_token_admin_client
        .mint(&depositor, &(collateral_amount * 2));

    data.stable_token_admin_client
        .mint(&contract_address, &(initial_debt));

    set_allowance(&env, &data, &depositor);

    data.contract_client.set_vault_conditions(
        &min_col_rate,
        &min_debt_creation,
        &opening_col_rate,
        &data.stable_token_denomination,
    );

    // It should fail if the user doesn't have a Vault open
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    assert!(data
        .contract_client
        .try_incr_col(
            &depositor,
            &collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.new_vault(
        &depositor,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    let current_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(current_currency_stats.total_vaults, 1);
    assert_eq!(current_currency_stats.total_debt, initial_debt);
    assert_eq!(current_currency_stats.total_col, collateral_amount);

    data.contract_client.incr_col(
        &depositor,
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
                    symbol_short!("incr_col"),
                    (
                        depositor.clone(),
                        collateral_amount.clone(),
                        data.stable_token_denomination.clone()
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        data.collateral_token_client.address.clone(),
                        symbol_short!("transfer"),
                        (
                            depositor.clone(),
                            data.contract_client.address.clone(),
                            collateral_amount.clone(),
                        )
                            .into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.total_vaults, 1);
    assert_eq!(updated_currency_stats.total_debt, initial_debt);
    assert_eq!(updated_currency_stats.total_col, collateral_amount * 2);

    assert_eq!(data.collateral_token_client.balance(&depositor), 0);
    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount * 2)
    );
}

#[test]
fn test_increase_debt() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    data.collateral_token_admin_client.mint(
        &base_variables.depositor,
        &(base_variables.collateral_amount * 5),
    );

    data.stable_token_admin_client.mint(
        &base_variables.contract_address,
        &(base_variables.initial_debt * 5),
    );

    set_allowance(&env, &data, &base_variables.depositor);

    // It should fail if the user doesn't have a Vault open
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    assert!(data
        .contract_client
        .try_incr_debt(
            &base_variables.depositor,
            &base_variables.collateral_amount,
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.new_vault(
        &base_variables.depositor,
        &base_variables.initial_debt,
        &(base_variables.collateral_amount * 2),
        &data.stable_token_denomination,
    );

    let current_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(current_currency_stats.total_vaults, 1);
    assert_eq!(
        current_currency_stats.total_debt,
        base_variables.initial_debt
    );
    assert_eq!(
        current_currency_stats.total_col,
        base_variables.collateral_amount * 2
    );

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        base_variables.initial_debt
    );

    data.contract_client.incr_debt(
        &base_variables.depositor,
        &base_variables.initial_debt,
        &data.stable_token_denomination,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths().first().unwrap(),
        &(
            base_variables.depositor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    symbol_short!("incr_debt"),
                    (
                        base_variables.depositor.clone(),
                        base_variables.initial_debt.clone(),
                        data.stable_token_denomination.clone(),
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        ) // [(
          //     // Address for which auth is performed
          //     base_variables.depositor.clone(),
          //     // Identifier of the called contract
          //     data.contract_client.address.clone(),
          //     // Name of the called function
          //     Symbol::short("incr_debt"),
          //     // Arguments used (converted to the env-managed vector via `into_val`)
          //     (
          //         base_variables.depositor.clone(),
          //         base_variables.initial_debt.clone(),
          //         data.stable_token_denomination.clone(),
          //     )
          //         .into_val(&env),
          // )]
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.total_vaults, 1);
    assert_eq!(
        updated_currency_stats.total_debt,
        base_variables.initial_debt * 2
    );
    assert_eq!(
        updated_currency_stats.total_col,
        base_variables.collateral_amount * 2
    );

    assert_eq!(
        data.stable_token_client.balance(&base_variables.depositor),
        (base_variables.initial_debt * 2)
    );
}

#[test]
fn test_pay_debt() {
    let env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let currency_price: i128 = 20000000;
    let depositor = Address::random(&env);
    let initial_debt: i128 = 50000000000;
    let collateral_amount: i128 = 50000000000;
    let contract_address: Address = data.contract_client.address.clone();

    let min_col_rate: i128 = 11000000;
    let min_debt_creation: i128 = 50000000000;
    let opening_col_rate: i128 = 11500000;

    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &currency_price);

    data.collateral_token_admin_client
        .mint(&depositor, &(collateral_amount));

    set_allowance(&env, &data, &depositor);

    data.stable_token_admin_client
        .mint(&contract_address, &(initial_debt * 10));

    data.contract_client.set_vault_conditions(
        &min_col_rate,
        &min_debt_creation,
        &opening_col_rate,
        &data.stable_token_denomination,
    );

    // It should fail if the user doesn't have a Vault open
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    assert!(data
        .contract_client
        .try_pay_debt(
            &depositor,
            &(initial_debt / 2),
            &data.stable_token_denomination
        )
        .is_err());

    data.contract_client.new_vault(
        &depositor,
        &initial_debt,
        &collateral_amount,
        &data.stable_token_denomination,
    );

    let current_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(current_currency_stats.total_vaults, 1);
    assert_eq!(current_currency_stats.total_debt, initial_debt);
    assert_eq!(current_currency_stats.total_col, collateral_amount);

    data.contract_client.pay_debt(
        &depositor,
        &(initial_debt / 2),
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
                    symbol_short!("pay_debt"),
                    (
                        depositor.clone(),
                        (initial_debt / 2).clone(),
                        data.stable_token_denomination.clone(),
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        data.stable_token_client.address.clone(),
                        symbol_short!("transfer"),
                        (
                            depositor.clone(),
                            data.stable_token_issuer.clone(),
                            (initial_debt / 2).clone(),
                        )
                            .into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )
    );

    let updated_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(updated_currency_stats.total_vaults, 1);
    assert_eq!(updated_currency_stats.total_debt, initial_debt / 2);
    assert_eq!(updated_currency_stats.total_col, collateral_amount);

    assert_eq!(
        data.stable_token_client.balance(&depositor),
        (initial_debt / 2)
    );
    assert_eq!(
        data.collateral_token_client.balance(&contract_address),
        (collateral_amount)
    );

    data.contract_client.pay_debt(
        &depositor,
        &(initial_debt / 2),
        &data.stable_token_denomination,
    );

    let final_currency_stats: CurrencyStats = data
        .contract_client
        .get_currency_stats(&data.stable_token_denomination);

    assert_eq!(final_currency_stats.total_vaults, 0);
    assert_eq!(final_currency_stats.total_debt, 0);
    assert_eq!(final_currency_stats.total_col, 0);

    assert_eq!(data.stable_token_client.balance(&depositor), 0);
    assert_eq!(data.collateral_token_client.balance(&contract_address), 0);

    // We confirm the vault was removed from the storage
    // TODO: UPDATE THIS ONCE SOROBAN IS FIXED
    assert!(data
        .contract_client
        .try_get_vault(&depositor, &data.stable_token_denomination)
        .is_err());
}

// TODO: Test the vault index is always updated after each update
