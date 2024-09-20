#![cfg(test)]

use crate::storage::deposits::DepositsStorageFunc;
use crate::storage::pools::{Pool, PoolsDataFunc};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal, Vec};

#[test]
fn test_cloning_a_pool() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    test_data.contract_client.mock_all_auths().set_pool(
        &test_data.staking_asset_client.address,
        &test_data.lock_period,
        &test_data.min_deposit,
    );

    let new_asset_address: Address = Address::generate(&e);

    assert!(test_data
        .contract_client
        .try_clone_pool(&test_data.staking_asset_client.address, &new_asset_address)
        .is_err());

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.manager,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "clone_pool",
                args: (
                    test_data.staking_asset_client.address.clone(),
                    new_asset_address.clone(),
                )
                    .into_val(&e),
                sub_invokes: &[],
            },
        }])
        .clone_pool(&test_data.staking_asset_client.address, &new_asset_address);

    e.as_contract(&test_data.contract_client.address, || {
        let old_pool: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();
        let new_pool: Pool = e._pools().pool(&new_asset_address).unwrap();

        assert_eq!(old_pool.active, new_pool.active);
        assert_eq!(old_pool.balance, new_pool.balance);
        assert_eq!(old_pool.deposits, new_pool.deposits);
        assert_eq!(old_pool.factor, new_pool.factor);
        assert_eq!(old_pool.lock_period, new_pool.lock_period);
        assert_eq!(old_pool.min_deposit, new_pool.min_deposit);
    });
}

#[test]
fn test_migrating_deposits() {
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

    let mut depositors: Vec<Address> = Vec::new(&e);

    for _ in 0..20 {
        let new_depositor: Address = Address::generate(&e);
        depositors.push_back(new_depositor.clone());

        test_data
            .staking_asset_stellar
            .mock_all_auths()
            .mint(&new_depositor, &(test_data.min_deposit as i128));

        test_data.contract_client.mock_all_auths().deposit(
            &test_data.staking_asset_client.address,
            &new_depositor,
            &test_data.min_deposit,
        );
    }

    let new_asset_address: Address = Address::generate(&e);

    test_data
        .contract_client
        .mock_all_auths()
        .clone_pool(&test_data.staking_asset_client.address, &new_asset_address);

    assert!(test_data
        .contract_client
        .try_migrate_deposits(
            &test_data.staking_asset_client.address,
            &new_asset_address,
            &depositors,
        )
        .is_err());

    e.budget().reset_default();

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.manager,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "migrate_deposits",
                args: (
                    test_data.staking_asset_client.address.clone(),
                    new_asset_address.clone(),
                    depositors.clone(),
                )
                    .into_val(&e),
                sub_invokes: &[],
            },
        }])
        .migrate_deposits(
            &test_data.staking_asset_client.address,
            &new_asset_address,
            &depositors,
        );

    e.budget().reset_default();

    e.as_contract(&test_data.contract_client.address, || {
        for depositor in depositors {
            assert!(e
                ._deposits()
                .get(&test_data.staking_asset_client.address, &depositor)
                .is_none());

            assert!(e._deposits().get(&new_asset_address, &depositor).is_some());
        }
    });
}
