#![cfg(test)]
extern crate std;

use crate::storage::deposits::Deposit;
use crate::tests::utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{Address, Env, Status, Vec};

#[test]
fn test_deposit_funds() {
    let env: Env = Env::default();
    let test_data: TestData = create_test_data(&env);
    init_contract(&test_data);

    let depositor_1: Address = Address::random(&env);
    let depositor_2: Address = Address::random(&env);
    let depositor_3: Address = Address::random(&env);

    for depositor in [&depositor_1, &depositor_2, &depositor_3] {
        test_data
            .deposit_asset
            .mint(&test_data.deposit_asset_admin, &depositor, &100000000000);
    }

    let invalid_amount_error_result = test_data
        .contract_client
        .try_deposit(&depositor_1, &100000000)
        .unwrap_err();

    assert_eq!(
        invalid_amount_error_result,
        Ok(Status::from_contract_error(10001))
    );

    let mut counter: u64 = 0;
    for depositor in [&depositor_1, &depositor_2, &depositor_3] {
        counter += 1;
        env.ledger().set(LedgerInfo {
            timestamp: counter,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
        });

        test_data.contract_client.deposit(&depositor, &5000000000);

        let deposit: Deposit = test_data.contract_client.get_deposit(&depositor);

        assert_eq!(deposit.deposit_time, counter);
        assert_eq!(deposit.amount, 5000000000u128);
        assert_eq!(deposit.id, depositor.clone());
    }

    // Confirm you can deposit twice and the funds will be updated but the timestamp will be the same
    for depositor in [&depositor_1, &depositor_2, &depositor_3] {
        counter += 1;
        env.ledger().set(LedgerInfo {
            timestamp: counter,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
        });

        test_data.contract_client.deposit(&depositor, &5000000000);

        let deposit: Deposit = test_data.contract_client.get_deposit(&depositor);

        assert_eq!(deposit.deposit_time, counter - 3);
        assert_eq!(deposit.amount, 10000000000u128);
        assert_eq!(deposit.id, depositor.clone());
    }

    let depositors: Vec<Address> = test_data.contract_client.get_depositors();
    let target_depositors_value = Vec::from_array(&env, [depositor_1, depositor_2, depositor_3]);
    assert_eq!(depositors, target_depositors_value);
}
