#![cfg(test)]

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

    data.collateral_token_admin_client
        .mock_all_auths()
        .mint(&depositor_1, &(depositor_1_collateral_amount as i128 * 2));

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

    let new_owner: Address = Address::generate(&env);

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
}
