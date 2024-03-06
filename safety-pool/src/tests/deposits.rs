#![cfg(test)]
extern crate std;

use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use crate::errors::SCErrors;
use crate::storage::core::CoreStats;
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

    let depositor_1: Address = Address::generate(&env);
    let depositor_2: Address = Address::generate(&env);
    let depositor_3: Address = Address::generate(&env);

    for depositor in [&depositor_1, &depositor_2, &depositor_3] {
        test_data
            .deposit_asset_client_admin
            .mint(&depositor, &mint_amount);
    }

    let invalid_amount_error_result = test_data
        .contract_client
        .try_deposit(&depositor_1, &100000000)
        .unwrap_err();

    assert_eq!(
        invalid_amount_error_result.unwrap(),
        SCErrors::BelowMinDeposit.into(),
    );

    let mut counter: u64 = 0;
    for depositor in [&depositor_1, &depositor_2, &depositor_3] {
        counter += 1;
        env.ledger().set(LedgerInfo {
            timestamp: counter,
            protocol_version: 1,
            sequence_number: env.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: u32::MAX,
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

        assert_eq!(deposit.last_deposit, counter);
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

    // Confirm you can't deposit twice
    for depositor in [&depositor_1, &depositor_2, &depositor_3] {
        let cant_deposit_twice_error = test_data
            .contract_client
            .try_deposit(&depositor, &5000000000)
            .unwrap_err()
            .unwrap();

        assert_eq!(
            cant_deposit_twice_error,
            SCErrors::DepositAlreadyCreated.into()
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

    let error_lock_period = test_data
        .contract_client
        .try_withdraw(&depositor_1)
        .unwrap_err()
        .unwrap();

    assert_eq!(error_lock_period, SCErrors::LockedPeriodUncompleted.into());

    // We increase the timestamp to comply with
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + (3600 * 50),
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

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
        let no_deposit_error = test_data
            .contract_client
            .try_get_deposit(&address)
            .unwrap_err()
            .unwrap();
        assert_eq!(no_deposit_error, SCErrors::DepositDoesntExist.into());

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

        assert_eq!(
            already_withdrew_error_result.unwrap(),
            SCErrors::DepositDoesntExist.into(),
        );
    }

    // we confirm the contract balance gets drained
    assert_eq!(
        test_data
            .deposit_asset_client
            .balance(&test_data.contract_client.address),
        0
    );

    let final_stats: CoreStats = test_data.contract_client.get_core_stats();
    assert_eq!(
        final_stats.lifetime_deposited,
        (mint_amount / 2) as u128 * 3
    );
    assert_eq!(final_stats.current_deposited, 0);
    assert_eq!(final_stats.lifetime_profit, 0);
    assert_eq!(final_stats.lifetime_liquidated, 0);
}
