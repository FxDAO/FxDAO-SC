#![cfg(test)]

use crate::errors::ContractErrors;
use crate::storage::pools::{Pool, PoolsDataFunc};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

#[test]
fn test_distribute_and_withdraw() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    test_data.contract_client.mock_all_auths().set_pool(
        &test_data.staking_asset_client.address,
        &test_data.lock_period,
        &test_data.min_deposit,
    );

    test_data
        .contract_client
        .mock_all_auths()
        .toggle_pool(&test_data.staking_asset_client.address, &true);

    let depositor: Address = Address::generate(&e);
    test_data
        .staking_asset_stellar
        .mock_all_auths()
        .mint(&depositor, &(test_data.min_deposit as i128));

    test_data
        .rewards_asset_stellar
        .mock_all_auths()
        .mint(&test_data.manager, &(test_data.min_deposit as i128));

    assert!(test_data
        .contract_client
        .try_distribute(
            &test_data.staking_asset_client.address,
            &test_data.min_deposit
        )
        .is_err());

    let cant_distribute_error = test_data
        .contract_client
        .mock_all_auths()
        .try_distribute(
            &test_data.staking_asset_client.address,
            &test_data.min_deposit,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        cant_distribute_error,
        ContractErrors::CantDistributeReward.into()
    );

    test_data.contract_client.mock_all_auths().deposit(
        &test_data.staking_asset_client.address,
        &depositor,
        &test_data.min_deposit,
    );

    let rewards_deposit_failed_error = test_data
        .contract_client
        .mock_all_auths()
        .try_distribute(
            &test_data.staking_asset_client.address,
            &(test_data.min_deposit * 10),
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        rewards_deposit_failed_error,
        ContractErrors::RewardsDepositFailed.into()
    );

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.manager,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "distribute",
                args: (
                    test_data.staking_asset_client.address.clone(),
                    test_data.min_deposit.clone(),
                )
                    .into_val(&e),
                sub_invokes: &[MockAuthInvoke {
                    contract: &test_data.rewards_asset_client.address,
                    fn_name: "transfer",
                    args: (
                        test_data.manager.clone(),
                        test_data.contract_client.address.clone(),
                        test_data.min_deposit as i128,
                    )
                        .into_val(&e),
                    sub_invokes: &[],
                }],
            },
        }])
        .distribute(
            &test_data.staking_asset_client.address,
            &test_data.min_deposit,
        );

    e.as_contract(&test_data.contract_client.address, || {
        let pool: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();

        assert_eq!(
            pool.factor,
            (test_data.min_deposit * 1_0000000) / pool.balance
        );
    });

    assert_eq!(
        test_data.min_deposit as i128,
        test_data
            .rewards_asset_client
            .balance(&test_data.contract_client.address),
    );

    e.ledger().set(LedgerInfo {
        timestamp: test_data.lock_period * 3,
        protocol_version: 20,
        sequence_number: e.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    test_data
        .contract_client
        .mock_all_auths()
        .withdraw(&test_data.staking_asset_client.address, &depositor);

    e.as_contract(&test_data.contract_client.address, || {
        let pool: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();
        assert_eq!(pool.factor, 0);
    });

    assert_eq!(
        0,
        test_data
            .rewards_asset_client
            .balance(&test_data.contract_client.address),
    );

    assert_eq!(
        test_data.min_deposit as i128,
        test_data.rewards_asset_client.balance(&depositor),
    );
}

// TODO: test with multiple deposits and multiple distributions
