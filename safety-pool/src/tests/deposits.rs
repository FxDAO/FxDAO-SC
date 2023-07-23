#![cfg(test)]
extern crate std;

use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use crate::errors::SCErrors;
use crate::storage::deposits::Deposit;
use crate::tests::utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger, LedgerInfo,
};
use soroban_sdk::{symbol_short, token, vec, Address, Env, IntoVal, Vec};

// TODO: TEST authentication
#[test]
fn test_deposit_funds() {
    let env: Env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    init_contract(&test_data);

    let mint_amount: i128 = 10000000000;

    let depositor_1: Address = Address::random(&env);
    let depositor_2: Address = Address::random(&env);
    let depositor_3: Address = Address::random(&env);

    for depositor in [&depositor_1, &depositor_2, &depositor_3] {
        test_data
            .deposit_asset_client_admin
            .mint(&depositor, &mint_amount);
    }

    let invalid_amount_error_result = test_data
        .contract_client
        .try_deposit(&depositor_1, &100000000)
        .unwrap_err();

    // TODO: FIX THIS ONE SOROBAN FIX IT
    // assert_eq!(
    //     invalid_amount_error_result.unwrap(),
    //     SCErrors::BelowMinDeposit.into(),
    // );

    let mut counter: u64 = 0;
    for depositor in [&depositor_1, &depositor_2, &depositor_3] {
        counter += 1;
        env.ledger().set(LedgerInfo {
            timestamp: counter,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_expiration: 0,
            min_persistent_entry_expiration: 0,
            max_entry_expiration: 0,
        });

        test_data
            .contract_client
            .deposit(&depositor, &(mint_amount as u128 / 2));

        let current_auths = env.auths();
        // Check the function is requiring the sender approved this operation
        assert_eq!(
            current_auths.first().unwrap(),
            &(
                depositor.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        test_data.contract_client.address.clone(),
                        symbol_short!("deposit"),
                        (depositor.clone(), (mint_amount as u128 / 2)).into_val(&env),
                    )),
                    sub_invocations: std::vec![AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            test_data.deposit_asset_client.address.clone(),
                            symbol_short!("transfer"),
                            (
                                depositor.clone(),
                                test_data.contract_client.address.clone(),
                                (mint_amount / 2),
                            )
                                .into_val(&env),
                        )),
                        sub_invocations: std::vec![]
                    }],
                }
            ),
        );

        let deposit: Deposit = test_data.contract_client.get_deposit(&depositor);

        assert_eq!(deposit.deposit_time, counter);
        assert_eq!(deposit.amount, mint_amount as u128 / 2);
        assert_eq!(deposit.depositor, depositor.clone());

        // Check the balance in the contract and depositor gets updated
        assert_eq!(
            test_data.deposit_asset_client.balance(&depositor),
            mint_amount / 2
        );
        assert_eq!(
            test_data
                .deposit_asset_client
                .balance(&test_data.contract_client.address),
            (mint_amount / 2) * counter as i128
        );
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
            min_temp_entry_expiration: 0,
            min_persistent_entry_expiration: 0,
            max_entry_expiration: 0,
        });

        test_data.contract_client.deposit(&depositor, &5000000000);

        let deposit: Deposit = test_data.contract_client.get_deposit(&depositor);

        assert_eq!(deposit.deposit_time, counter - 3);
        assert_eq!(deposit.amount, mint_amount as u128);
        assert_eq!(deposit.depositor, depositor.clone());

        // Check the balance in the contract and depositor gets updated
        assert_eq!(test_data.deposit_asset_client.balance(&depositor), 0);
        assert_eq!(
            test_data
                .deposit_asset_client
                .balance(&test_data.contract_client.address),
            (mint_amount / 2) * counter as i128
        );
    }

    let mut depositors: Vec<Address> = test_data.contract_client.get_depositors();
    let target_depositors_value = Vec::from_array(
        &env,
        [
            depositor_1.clone(),
            depositor_2.clone(),
            depositor_3.clone(),
        ],
    );
    assert_eq!(depositors, target_depositors_value);

    // Check that withdrawing deposits works ok
    for address in depositors.clone().iter() {
        test_data.contract_client.withdraw(&address);

        // Check the function is requiring the sender approved this operation
        assert_eq!(
            env.auths().first().unwrap(),
            &(
                address.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        test_data.contract_client.address.clone(),
                        symbol_short!("withdraw"),
                        (address.clone(),).into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                },
            ),
        );

        // Check that the "depositors" Vec gets updated
        depositors.pop_front();
        let updated_depositors = test_data.contract_client.get_depositors();
        assert_eq!(depositors, updated_depositors);

        // Check that the deposit gets updated (value is zero)
        let updated_deposit: Deposit = test_data.contract_client.get_deposit(&address);
        assert_eq!(updated_deposit.amount, 0);

        // We check the depositor got all its funds
        assert_eq!(
            test_data.deposit_asset_client.balance(&address),
            mint_amount
        );

        // Test that if the user already withdrew its fund it should fail if try again
        let already_withdrew_error_result = test_data
            .contract_client
            .try_withdraw(&depositor_1)
            .unwrap_err();

        // TODO: UPDATE THIS ONCE SOROBAN FIX IT
        // assert_eq!(
        //     already_withdrew_error_result.unwrap(),
        //     SCErrors::NothingToWithdraw.into(),
        // );
    }

    // we confirm the contract balance gets drained
    assert_eq!(
        test_data
            .deposit_asset_client
            .balance(&test_data.contract_client.address),
        0
    );
}
