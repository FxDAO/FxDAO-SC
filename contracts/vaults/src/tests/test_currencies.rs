// TODO: specify all the steps in the tests

#![cfg(test)]
extern crate std;
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{Env, IntoVal, Symbol};

use crate::tests::test_utils::*;

#[test]
fn test_create_new_currency() {
    let env = Env::default();
    env.mock_all_auths();
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

    data.contract_client.create_currency(
        &data.stable_token_denomination,
        &data.stable_token_client.address,
    );

    // Check the function is requiring the protocol manager approved this operation
    assert_eq!(
        env.auths().first().unwrap(),
        &(
            data.protocol_manager.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    data.contract_client.address.clone(),
                    Symbol::new(&env, "create_currency"),
                    (
                        data.stable_token_denomination.clone(),
                        data.stable_token_client.address.clone(),
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![],
            }
        )
    );
}
