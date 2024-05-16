#![cfg(test)]

use crate::errors::SCErrors;
use crate::tests::test_utils::{create_test_data, TestData};
use soroban_sdk::{Env, Vec};

#[test]
pub fn test_invalid_fee() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let mut test_data: TestData = create_test_data(&env);
    test_data.fee_percentage = 0_0600000;
    let error = test_data
        .stable_liquidity_pool_contract_client
        .try_init(
            &test_data.admin,
            &test_data.manager,
            &test_data.governance_token_client.address,
            &(Vec::from_array(
                &env,
                [
                    test_data.usdc_token_client.address.clone(),
                    test_data.usdt_token_client.address.clone(),
                    test_data.usdx_token_client.address.clone(),
                ],
            )),
            &test_data.fee_percentage,
            &test_data.treasury,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(error, SCErrors::InvalidFee.into());
}
