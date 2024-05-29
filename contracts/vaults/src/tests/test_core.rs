#![cfg(test)]

extern crate std;

use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{Address, Env, IntoVal, Symbol};

use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::tests::test_utils::{create_base_data, TestData};

#[test]
fn test_init() {
    let env: Env = Env::default();

    // Create the contract
    let data: TestData = create_base_data(&env);

    data.contract_client.init(
        &data.contract_admin,
        &data.protocol_manager,
        &data.collateral_token_client.address,
        &data.stable_token_issuer,
        &data.treasury,
        &data.fee,
        &data.oracle,
    );

    let core_state: CoreState = data.contract_client.get_core_state();

    assert_eq!(&core_state.col_token, &data.collateral_token_client.address);
    assert_eq!(&core_state.oracle, &data.oracle);
    assert_eq!(&core_state.protocol_manager, &data.protocol_manager);
    assert_eq!(&core_state.admin, &data.contract_admin);
    assert_eq!(&core_state.stable_issuer, &data.stable_token_issuer);
    assert_eq!(&core_state.panic_mode, &false);

    let init_error = data
        .contract_client
        .try_init(
            &data.contract_admin,
            &data.protocol_manager,
            &data.collateral_token_client.address,
            &data.stable_token_issuer,
            &data.treasury,
            &data.fee,
            &data.oracle,
        )
        .unwrap_err();

    assert_eq!(init_error.unwrap(), SCErrors::CoreAlreadySet.into());
}

#[test]
fn test_site_updates() {
    let env: Env = Env::default();
    env.mock_all_auths();

    // Create the contract
    let data: TestData = create_base_data(&env);

    data.contract_client.init(
        &data.contract_admin,
        &data.protocol_manager,
        &data.collateral_token_client.address,
        &data.stable_token_issuer,
        &data.treasury,
        &data.fee,
        &data.oracle,
    );

    let core_state: CoreState = data.contract_client.get_core_state();
    assert_eq!(&core_state.admin, &data.contract_admin);

    let new_admin: Address = Address::generate(&env);
    data.contract_client.set_address(&0, &new_admin);

    assert_eq!(
        env.auths().first().unwrap(),
        &(
            data.contract_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    Symbol::new(&env, "set_address"),
                    (0, new_admin.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )
    );

    let updated_core_state: CoreState = data.contract_client.get_core_state();
    assert_eq!(&updated_core_state.admin, &new_admin);

    let new_protocol_manager: Address = Address::generate(&env);
    data.contract_client.set_address(&1, &new_protocol_manager);

    assert_eq!(
        env.auths().first().unwrap(),
        &(
            data.protocol_manager.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    Symbol::new(&env, "set_address"),
                    (1, new_protocol_manager.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )
    );

    let updated_core_state: CoreState = data.contract_client.get_core_state();
    assert_eq!(&updated_core_state.protocol_manager, &new_protocol_manager);
}

#[test]
fn test_invalid_protocol_fee() {
    let env: Env = Env::default();
    env.mock_all_auths();

    // Create the contract
    let data: TestData = create_base_data(&env);

    let fee_error = data
        .contract_client
        .try_init(
            &data.contract_admin,
            &data.protocol_manager,
            &data.collateral_token_client.address,
            &data.stable_token_issuer,
            &data.treasury,
            &0_0200000,
            &data.oracle,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(fee_error, SCErrors::InvalidFee.into());

    // TODO: in the future if we allow updating this fee, we should test it here too
}
