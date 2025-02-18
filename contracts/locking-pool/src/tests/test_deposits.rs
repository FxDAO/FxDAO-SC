#![cfg(test)]

use crate::errors::ContractErrors;
use crate::storage::deposits::{Deposit, DepositsStorageFunc};
use crate::storage::pools::{Pool, PoolsDataFunc};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal, Vec};

#[test]
pub fn test_a_deposit() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    test_data.contract_client.mock_all_auths().set_pool(
        &test_data.staking_asset_client.address,
        &test_data.lock_period,
        &test_data.min_deposit,
    );

    let depositor: Address = Address::generate(&e);
    test_data
        .staking_asset_stellar
        .mock_all_auths()
        .mint(&depositor, &((test_data.min_deposit * 2) as i128));

    assert!(test_data
        .contract_client
        .try_deposit(
            &test_data.staking_asset_client.address,
            &depositor,
            &test_data.min_deposit
        )
        .is_err());

    let inactive_pool_error = test_data
        .contract_client
        .mock_all_auths()
        .try_deposit(&test_data.staking_asset_client.address, &depositor, &1)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        &inactive_pool_error,
        &ContractErrors::PoolDoesntAcceptDeposits.into()
    );

    test_data
        .contract_client
        .mock_all_auths()
        .toggle_pool(&test_data.staking_asset_client.address, &true);

    let min_deposit_error = test_data
        .contract_client
        .mock_all_auths()
        .try_deposit(&test_data.staking_asset_client.address, &depositor, &1)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        &min_deposit_error,
        &ContractErrors::InvalidDepositAmount.into()
    );

    let funds_deposit_error = test_data
        .contract_client
        .mock_all_auths()
        .try_deposit(
            &test_data.staking_asset_client.address,
            &depositor,
            &(test_data.min_deposit * 3),
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        &funds_deposit_error,
        &ContractErrors::FundsDepositFailed.into()
    );

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &depositor,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "deposit",
                args: (
                    test_data.staking_asset_client.address.clone(),
                    depositor.clone(),
                    test_data.min_deposit,
                )
                    .into_val(&e),
                sub_invokes: &[MockAuthInvoke {
                    contract: &test_data.staking_asset_stellar.address,
                    fn_name: "transfer",
                    args: (
                        depositor.clone(),
                        test_data.contract_client.address.clone(),
                        test_data.min_deposit as i128,
                    )
                        .into_val(&e),
                    sub_invokes: &[],
                }],
            },
        }])
        .deposit(
            &test_data.staking_asset_client.address,
            &depositor,
            &test_data.min_deposit,
        );

    e.as_contract(&test_data.contract_client.address, || {
        let deposit: Deposit = e
            ._deposits()
            .get(&test_data.staking_asset_client.address, &depositor)
            .unwrap();
        assert_eq!(&deposit.amount, &test_data.min_deposit);
        assert_eq!(&deposit.unlocks_at, &test_data.lock_period);
        assert_eq!(&deposit.snapshot, &0);

        let core_state: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();
        assert_eq!(&core_state.factor, &0);
        assert_eq!(&core_state.balance, &deposit.amount);
        assert_eq!(&core_state.deposits, &1);
    });

    let already_staked_error = test_data
        .contract_client
        .mock_all_auths()
        .try_deposit(
            &test_data.staking_asset_client.address,
            &depositor,
            &(test_data.min_deposit),
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        &already_staked_error,
        &ContractErrors::DepositAlreadyExists.into()
    );

    assert_eq!(
        test_data.min_deposit as i128,
        test_data.staking_asset_client.balance(&depositor),
    );
    assert_eq!(
        test_data.min_deposit as i128,
        test_data
            .staking_asset_client
            .balance(&test_data.contract_client.address),
    );
}

#[test]
pub fn test_multiple_deposits() {
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

    let depositor_1: Address = Address::generate(&e);
    let depositor_2: Address = Address::generate(&e);
    let depositor_3: Address = Address::generate(&e);
    let depositor_4: Address = Address::generate(&e);
    let depositors: Vec<Address> =
        Vec::from_array(&e, [depositor_1, depositor_2, depositor_3, depositor_4]);

    for depositor in depositors.iter() {
        test_data
            .staking_asset_stellar
            .mock_all_auths()
            .mint(&depositor, &(test_data.min_deposit as i128));

        test_data.contract_client.mock_all_auths().deposit(
            &test_data.staking_asset_client.address,
            &depositor,
            &test_data.min_deposit,
        );
    }

    e.as_contract(&test_data.contract_client.address, || {
        let core_state: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();

        assert_eq!(core_state.deposits, depositors.len() as u64);
        assert_eq!(
            core_state.balance,
            (depositors.len() as u128) * test_data.min_deposit
        );
        assert_eq!(core_state.factor, 0);
    })
}

#[test]
pub fn test_withdraws() {
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

    let depositor_1: Address = Address::generate(&e);
    let depositor_2: Address = Address::generate(&e);
    let depositor_3: Address = Address::generate(&e);
    let depositor_4: Address = Address::generate(&e);
    let depositors: Vec<Address> = Vec::from_array(
        &e,
        [
            depositor_1.clone(),
            depositor_2.clone(),
            depositor_3.clone(),
            depositor_4.clone(),
        ],
    );

    for depositor in depositors.iter() {
        test_data
            .staking_asset_stellar
            .mock_all_auths()
            .mint(&depositor, &(test_data.min_deposit as i128));

        test_data.contract_client.mock_all_auths().deposit(
            &test_data.staking_asset_client.address,
            &depositor,
            &test_data.min_deposit,
        );
    }

    assert!(test_data
        .contract_client
        .try_withdraw(&test_data.staking_asset_client.address, &depositor_4)
        .is_err());

    let stake_is_locked_error = test_data
        .contract_client
        .mock_all_auths()
        .try_withdraw(&test_data.staking_asset_client.address, &depositor_4)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        stake_is_locked_error,
        ContractErrors::DepositIsStillLocked.into()
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
        .mock_auths(&[MockAuth {
            address: &depositor_4,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "withdraw",
                args: (
                    test_data.staking_asset_client.address.clone(),
                    depositor_4.clone(),
                )
                    .into_val(&e),
                sub_invokes: &[],
            },
        }])
        .withdraw(&test_data.staking_asset_client.address, &depositor_4);

    let no_stake_error = test_data
        .contract_client
        .mock_all_auths()
        .try_withdraw(&test_data.staking_asset_client.address, &depositor_4)
        .unwrap_err()
        .unwrap();

    assert_eq!(no_stake_error, ContractErrors::DepositDoesntExist.into());

    test_data
        .contract_client
        .mock_all_auths()
        .withdraw(&test_data.staking_asset_client.address, &depositor_3);

    test_data
        .contract_client
        .mock_all_auths()
        .withdraw(&test_data.staking_asset_client.address, &depositor_2);

    test_data
        .contract_client
        .mock_all_auths()
        .withdraw(&test_data.staking_asset_client.address, &depositor_1);

    e.as_contract(&test_data.contract_client.address, || {
        let core_state: Pool = e
            ._pools()
            .pool(&test_data.staking_asset_client.address)
            .unwrap();

        assert_eq!(core_state.factor, 0);
        assert_eq!(core_state.balance, 0);
        assert_eq!(core_state.deposits, 0);
    });
}
