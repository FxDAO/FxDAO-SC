#![cfg(test)]

use crate::errors::ContractErrors;
use crate::storage::pools::{Pool, PoolsDataFunc};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

#[test]
pub fn test_setting_a_pool() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let non_signature_error = test_data.contract_client.try_set_pool(
        &test_data.staking_asset_client.address,
        &test_data.lock_period,
        &test_data.min_deposit,
    );

    assert!(non_signature_error.is_err());

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.manager,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "set_pool",
                args: (
                    test_data.staking_asset_client.address.clone(),
                    test_data.lock_period.clone(),
                    test_data.min_deposit.clone(),
                )
                    .into_val(&e),
                sub_invokes: &[],
            },
        }])
        .set_pool(
            &test_data.staking_asset_client.address,
            &test_data.lock_period,
            &test_data.min_deposit,
        );

    e.as_contract(&test_data.contract_client.address, || {
        let pool: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();

        assert_eq!(&pool.asset, &test_data.staking_asset_client.address);
        assert_eq!(&pool.lock_period, &test_data.lock_period);
        assert_eq!(&pool.min_deposit, &test_data.min_deposit);
        assert_eq!(&pool.active, &false);
        assert_eq!(&pool.balance, &0);
        assert_eq!(&pool.deposits, &0);
        assert_eq!(&pool.factor, &0);
    });

    test_data.contract_client.mock_all_auths().set_pool(
        &test_data.staking_asset_client.address,
        &(test_data.lock_period + 1),
        &(test_data.min_deposit + 1),
    );

    e.as_contract(&test_data.contract_client.address, || {
        let pool: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();

        assert_eq!(&pool.asset, &test_data.staking_asset_client.address);
        assert_eq!(&pool.lock_period, &(test_data.lock_period + 1));
        assert_eq!(&pool.min_deposit, &(test_data.min_deposit + 1));
        assert_eq!(&pool.active, &false);
        assert_eq!(&pool.balance, &0);
        assert_eq!(&pool.deposits, &0);
        assert_eq!(&pool.factor, &0);
    });
}

#[test]
pub fn test_toggling_pool() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let non_existing_pool_error = test_data
        .contract_client
        .mock_all_auths()
        .try_toggle_pool(&test_data.staking_asset_client.address, &false)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        non_existing_pool_error,
        ContractErrors::PoolDoesntExist.into()
    );

    test_data.contract_client.mock_all_auths().set_pool(
        &test_data.staking_asset_client.address,
        &test_data.lock_period,
        &test_data.min_deposit,
    );

    let no_signature_error = test_data
        .contract_client
        .try_toggle_pool(&test_data.staking_asset_client.address, &false);

    assert!(no_signature_error.is_err());

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.admin,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "toggle_pool",
                args: (test_data.staking_asset_client.address.clone(), true).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .toggle_pool(&test_data.staking_asset_client.address, &true);

    e.as_contract(&test_data.contract_client.address, || {
        let pool: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();

        assert_eq!(&pool.active, &true);
    });
}

#[test]
pub fn test_remove_pool() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let non_existing_pool_error = test_data
        .contract_client
        .mock_all_auths()
        .try_remove_pool(&test_data.staking_asset_client.address)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        non_existing_pool_error,
        ContractErrors::PoolDoesntExist.into()
    );

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

    test_data.contract_client.mock_all_auths().deposit(
        &test_data.staking_asset_client.address,
        &depositor,
        &test_data.min_deposit,
    );

    let cant_remove_pool_error = test_data
        .contract_client
        .mock_all_auths()
        .try_remove_pool(&test_data.staking_asset_client.address)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        cant_remove_pool_error,
        ContractErrors::PoolCanNotBeDeleted.into()
    );

    e.ledger().set(LedgerInfo {
        timestamp: test_data.lock_period * 2,
        protocol_version: 22,
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

    let no_signature_error = test_data
        .contract_client
        .try_remove_pool(&test_data.staking_asset_client.address);

    assert!(no_signature_error.is_err());

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.manager,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "remove_pool",
                args: (&test_data.staking_asset_client.address,).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .remove_pool(&test_data.staking_asset_client.address);

    e.as_contract(&test_data.contract_client.address, || {
        assert_eq!(
            e._pools()
                .pool(&test_data.staking_asset_client.address)
                .is_none(),
            true
        );
    });
}
