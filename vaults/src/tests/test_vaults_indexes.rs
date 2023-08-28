#![cfg(test)]

extern crate std;

use crate::storage::vaults::{OptionalVaultKey, Vault, VaultKey, VaultsInfo};
use crate::tests::test_utils::{create_base_data, create_base_variables, set_initial_state};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{panic_with_error, token, vec, Address, Env, Vec};

#[test]
fn test_indexes_orders() {
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

    let currency_price: u128 = 920330;
    let min_col_rate: u128 = 11000000;
    let min_debt_creation: u128 = 1000000000;
    let opening_col_rate: u128 = 11500000;

    data.contract_client.set_vault_conditions(
        &min_col_rate,
        &min_debt_creation,
        &opening_col_rate,
        &data.stable_token_denomination,
    );

    data.contract_client
        .set_currency_rate(&data.stable_token_denomination, &currency_price);

    // 1st Set of tests
    // This section includes and checks that every time we create a new vault the values are updated

    // First deposit
    // This deposit should have an index of: 2000_0000000
    let depositor_1 = Address::random(&env);
    let depositor_1_debt: u128 = 150_0000000;
    let depositor_1_collateral_amount: u128 = 3000_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_1, &(depositor_1_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_1,
        &depositor_1_debt,
        &depositor_1_collateral_amount,
        &data.stable_token_denomination,
    );

    // Second depositor
    // This deposit should have an index of: 1857_1428571
    let depositor_2 = Address::random(&env);
    let depositor_2_debt: u128 = 140_0000000;
    let depositor_2_collateral_amount: u128 = 2600_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_2, &(depositor_2_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_2,
        &depositor_2_debt,
        &depositor_2_collateral_amount,
        &data.stable_token_denomination,
    );

    // Third depositor
    // This deposit should have an index of: 3250_0000000
    let depositor_3 = Address::random(&env);
    let depositor_3_debt: u128 = 100_0000000;
    let depositor_3_collateral_amount: u128 = 3250_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_3, &(depositor_3_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(VaultKey {
            index: 2000_0000000,
            account: depositor_1.clone(),
            denomination: data.stable_token_denomination.clone(),
        }),
        &depositor_3,
        &depositor_3_debt,
        &depositor_3_collateral_amount,
        &data.stable_token_denomination,
    );

    // fourth depositor
    // This deposit should have an index of: 3250_0000000
    let depositor_4 = Address::random(&env);
    let depositor_4_debt: u128 = 100_0000000;
    let depositor_4_collateral_amount: u128 = 3250_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_4, &(depositor_4_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(VaultKey {
            index: 2000_0000000,
            account: depositor_1.clone(),
            denomination: data.stable_token_denomination.clone(),
        }),
        &depositor_4,
        &depositor_4_debt,
        &depositor_4_collateral_amount,
        &data.stable_token_denomination,
    );

    // fifth depositor
    // This deposit should have an index of: 1756_4285710
    let depositor_5 = Address::random(&env);
    let depositor_5_debt: u128 = 140_0000000;
    let depositor_5_collateral_amount: u128 = 2459_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_5, &(depositor_5_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        &depositor_5,
        &depositor_5_debt,
        &depositor_5_collateral_amount,
        &data.stable_token_denomination,
    );

    // Sixth depositor
    // This deposit should have an index of: 6000_0000000
    let depositor_6 = Address::random(&env);
    let depositor_6_debt: u128 = 150_0000000;
    let depositor_6_collateral_amount: u128 = 9000_0000000;

    data.collateral_token_admin_client
        .mint(&depositor_6, &(depositor_6_collateral_amount as i128 * 2));

    data.contract_client.new_vault(
        &OptionalVaultKey::Some(VaultKey {
            index: 3250_0000000,
            account: depositor_3.clone(),
            denomination: data.stable_token_denomination.clone(),
        }),
        &depositor_6,
        &depositor_6_debt,
        &depositor_6_collateral_amount,
        &data.stable_token_denomination,
    );

    // 2nd part of the test
    // We are going to get the lowest vault and we should be able to go from lowest to higher
    // ----------------------------------------
    env.budget().reset_default();

    let latest_vaults_info: VaultsInfo = data
        .contract_client
        .get_vaults_info(&data.stable_token_denomination);

    let lowest_key = match latest_vaults_info.lowest_key {
        OptionalVaultKey::None => panic!("We don't reach this point"),
        OptionalVaultKey::Some(data) => data,
    };

    let first_vault: Vault = data
        .contract_client
        .get_vault(&lowest_key.account, &lowest_key.denomination);

    let first_lowest_key = match first_vault.next_key {
        OptionalVaultKey::None => panic!("We don't reach this point"),
        OptionalVaultKey::Some(data) => data,
    };

    assert_eq!(first_vault.index, 1756_4285714);
    assert_eq!(first_vault.account, depositor_5);

    let second_vault: Vault = data
        .contract_client
        .get_vault(&first_lowest_key.account, &first_lowest_key.denomination);

    let second_lowest_key = match second_vault.next_key {
        OptionalVaultKey::None => panic!("We don't reach this point"),
        OptionalVaultKey::Some(data) => data,
    };

    assert_eq!(second_vault.index, 1857_1428571);
    assert_eq!(second_vault.account, depositor_2);

    let third_vault: Vault = data
        .contract_client
        .get_vault(&second_lowest_key.account, &second_lowest_key.denomination);

    let third_lowest_key = match third_vault.next_key {
        OptionalVaultKey::None => panic!("We don't reach this point"),
        OptionalVaultKey::Some(data) => data,
    };

    assert_eq!(third_vault.index, 2000_0000000);
    assert_eq!(third_vault.account, depositor_1);

    let fourth_vault: Vault = data
        .contract_client
        .get_vault(&third_lowest_key.account, &third_lowest_key.denomination);

    let fourth_lowest_key = match fourth_vault.next_key {
        OptionalVaultKey::None => panic!("We don't reach this point"),
        OptionalVaultKey::Some(data) => data,
    };

    assert_eq!(fourth_vault.index, 3250_0000000);
    assert_eq!(fourth_vault.account, depositor_4);

    let fifth_vault: Vault = data
        .contract_client
        .get_vault(&fourth_lowest_key.account, &fourth_lowest_key.denomination);

    let fifth_lowest_key = match fifth_vault.next_key {
        OptionalVaultKey::None => panic!("We don't reach this point"),
        OptionalVaultKey::Some(data) => data,
    };

    assert_eq!(fifth_vault.index, 3250_0000000);
    assert_eq!(fifth_vault.account, depositor_3);

    let sixth_vault: Vault = data
        .contract_client
        .get_vault(&fifth_lowest_key.account, &fifth_lowest_key.denomination);

    match sixth_vault.next_key {
        OptionalVaultKey::None => {}
        OptionalVaultKey::Some(_) => panic!("We don't reach this point"),
    };

    assert_eq!(sixth_vault.index, 6000_0000000);
    assert_eq!(sixth_vault.account, depositor_6);

    // 3rd phase of the test
    // We are going to start increasing the collateral and increasing/paying the debt
    // ----------------------------------------
    env.budget().reset_default();
}
