#![cfg(test)]

use crate::errors::SCErrors;
use crate::storage::vaults::{OptionalVaultKey, Vault, VaultKey};
use crate::tests::test_utils::{
    create_base_data, create_base_variables, set_initial_state, update_oracle_price,
    InitialVariables, TestData,
};

use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

#[test]
fn test_transfer_debt() {
    let env: Env = Env::default();
    let data: TestData = create_base_data(&env);
    let base_variables: InitialVariables = create_base_variables(&env, &data);
    set_initial_state(&env, &data, &base_variables);

    let currency_price: u128 = 920330;
    let min_col_rate: u128 = 11000000;
    let min_debt_creation: u128 = 1000000000;
    let opening_col_rate: u128 = 11500000;

    data.contract_client.mock_all_auths().set_vault_conditions(
        &min_col_rate,
        &min_debt_creation,
        &opening_col_rate,
        &data.stable_token_denomination,
    );

    update_oracle_price(
        &env,
        &data.oracle_contract_client,
        &data.stable_token_denomination,
        &(currency_price as i128),
    );

    // First deposit
    // This deposit should have an index of: 2000_0000000 - fee
    let depositor_1: Address = Address::generate(&env);
    let depositor_1_debt: u128 = 150_0000000;
    let depositor_1_collateral_amount: u128 = 3000_0000000;

    // Second deposit
    // This deposit should have an index of: 2000_0000000 - fee
    let depositor_2: Address = Address::generate(&env);
    let depositor_2_debt: u128 = 150_0000000;
    let depositor_2_collateral_amount: u128 = 5000_0000000;

    data.collateral_token_admin_client
        .mock_all_auths()
        .mint(&depositor_1, &(depositor_1_collateral_amount as i128 * 2));

    data.collateral_token_admin_client
        .mock_all_auths()
        .mint(&depositor_2, &(depositor_2_collateral_amount as i128 * 2));

    data.contract_client.mock_all_auths().new_vault(
        &OptionalVaultKey::None,
        &depositor_2,
        &depositor_2_debt,
        &depositor_2_collateral_amount,
        &data.stable_token_denomination,
    );

    data.contract_client.mock_all_auths().new_vault(
        &OptionalVaultKey::None,
        &depositor_1,
        &depositor_1_debt,
        &depositor_1_collateral_amount,
        &data.stable_token_denomination,
    );

    let depositor_1_vault: Vault = data
        .contract_client
        .get_vault(&depositor_1, &data.stable_token_denomination);

    let depositor_2_vault: Vault = data
        .contract_client
        .get_vault(&depositor_2, &data.stable_token_denomination);

    let new_owner: Address = Address::generate(&env);

    // Should fail because of no authorization provided
    assert!(data
        .contract_client
        .try_transfer_debt(
            &OptionalVaultKey::None,
            &VaultKey {
                index: depositor_1_vault.index.clone(),
                account: depositor_1_vault.account.clone(),
                denomination: data.stable_token_denomination.clone(),
            },
            &new_owner,
        )
        .is_err());

    let already_has_vault_error = data
        .contract_client
        .mock_all_auths()
        .try_transfer_debt(
            &OptionalVaultKey::Some(VaultKey {
                index: depositor_1_vault.index.clone(),
                account: depositor_1_vault.account.clone(),
                denomination: data.stable_token_denomination.clone(),
            }),
            &VaultKey {
                index: depositor_2_vault.index.clone(),
                account: depositor_2_vault.account.clone(),
                denomination: data.stable_token_denomination.clone(),
            },
            &depositor_2,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        already_has_vault_error,
        SCErrors::UserAlreadyHasDenominationVault.into(),
    );

    data.contract_client
        .mock_auths(&[MockAuth {
            address: &depositor_1,
            invoke: &MockAuthInvoke {
                contract: &data.contract_client.address,
                fn_name: "transfer_debt",
                args: (
                    OptionalVaultKey::None,
                    VaultKey {
                        index: depositor_1_vault.index.clone(),
                        account: depositor_1_vault.account.clone(),
                        denomination: data.stable_token_denomination.clone(),
                    },
                    new_owner.clone(),
                )
                    .into_val(&env),
                sub_invokes: &[],
            },
        }])
        .transfer_debt(
            &OptionalVaultKey::None,
            &VaultKey {
                index: depositor_1_vault.index.clone(),
                account: depositor_1_vault.account.clone(),
                denomination: data.stable_token_denomination.clone(),
            },
            &new_owner,
        );

    let new_vault = data
        .contract_client
        .get_vault(&new_owner, &data.stable_token_denomination);

    assert_eq!(new_owner, new_vault.account);
    assert_eq!(depositor_1_vault.total_debt, new_vault.total_debt);
    assert_eq!(
        depositor_1_vault.total_collateral,
        new_vault.total_collateral
    );
    assert_eq!(depositor_1_vault.index, new_vault.index);
    assert_eq!(depositor_1_vault.next_key, new_vault.next_key);

    data.contract_client.get_vaults(
        &OptionalVaultKey::None,
        &data.stable_token_denomination,
        &10,
        &false,
    );

    data.contract_client
        .mock_all_auths()
        .set_lowest_key(&data.stable_token_denomination, &new_vault.next_key);
}
