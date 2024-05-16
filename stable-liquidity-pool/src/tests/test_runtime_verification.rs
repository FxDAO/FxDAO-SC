#![cfg(test)]

use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::storage::deposits::Deposit;
// use crate::tests::test_utils::{create_test_data_stable, init_contract_stable, prepare_test_accounts_stable, TestDataStable};
use crate::tests::test_utils::{create_test_data, init_contract, prepare_test_accounts, TestData};
use soroban_sdk::testutils::arbitrary::std;
use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger, LedgerInfo,
};
use soroban_sdk::{map, symbol_short, Address, Env, IntoVal, Vec};

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn test_deposit_lock(early_time in 0..172800u64, late_time in 172800_u64..u64::MAX) { // 3600 * 48 = 172800
        let env: Env = Env::default();
        env.mock_all_auths();

        let test_data: TestData = create_test_data(&env);
        init_contract(&env, &test_data);

        let deposit_amount: u128 = 100_0000000;
        let depositor_1: Address = Address::generate(&env);
        let depositors: Vec<Address> = Vec::from_array(&env, [depositor_1.clone()]);

        prepare_test_accounts(&test_data, &depositors);

        test_data.stable_liquidity_pool_contract_client.deposit(
            &depositor_1,
            &test_data.usdc_token_client.address,
            &deposit_amount,
        );

        let current_auths = env.auths();
        assert_eq!(
            current_auths.first().unwrap(),
            &(
                depositor_1.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        test_data
                            .stable_liquidity_pool_contract_client
                            .address
                            .clone(),
                        symbol_short!("deposit"),
                        (
                            depositor_1.clone(),
                            test_data.usdc_token_client.address.clone(),
                            deposit_amount.clone()
                        )
                            .into_val(&env),
                    )),
                    sub_invocations: std::vec![AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            test_data.usdc_token_client.address.clone(),
                            symbol_short!("transfer"),
                            (
                                depositor_1.clone(),
                                test_data
                                    .stable_liquidity_pool_contract_client
                                    .address
                                    .clone(),
                                (deposit_amount as i128).clone()
                            )
                                .into_val(&env),
                        )),
                        sub_invocations: std::vec![],
                    }],
                }
            )
        );

        let core_state: CoreState = test_data.stable_liquidity_pool_contract_client.get_core_state();

        let deposit_1: Deposit = test_data.stable_liquidity_pool_contract_client.get_deposit(&depositor_1);

        assert_eq!(&deposit_1.unlocks_at, &(0 + (3600 * 48)));
        assert_eq!(&deposit_1.depositor, &depositor_1);
        assert_eq!(&deposit_1.shares, &deposit_amount);
        assert_eq!(&test_data.usdc_token_client.balance(&depositor_1), &((test_data.minted_asset_amount - deposit_amount) as i128));
        assert_eq!(&core_state.total_deposited, &deposit_amount);

        // Set time to early time
        env.ledger().set(LedgerInfo {
            timestamp: early_time, // 0..(3600 * 48)
            protocol_version: 1,
            sequence_number: env.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: u32::MAX,
        });

        // Attempt to withdraw entire deposit
        let locker_period_uncompleted = test_data
            .stable_liquidity_pool_contract_client
            .try_withdraw(
                &depositor_1,
                &deposit_amount,
                &map![
                    &env,
                    (test_data.usdc_token_client.address.clone(), deposit_amount),
                ],
            )
            .unwrap_err()
            .unwrap();

        // Locking period should still be in effect
        assert_eq!(locker_period_uncompleted, SCErrors::LockedPeriodUncompleted.into());

        // Set time to late time
        env.ledger().set(LedgerInfo {
            timestamp: late_time, // (3600 * 48)..2^64
            protocol_version: 1,
            sequence_number: env.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: u32::MAX,
        });

        // Withdraw whole deposit will pass now
        test_data
            .stable_liquidity_pool_contract_client
            .withdraw(
                &depositor_1,
                &(deposit_amount),
                &map![
                    &env,
                    (test_data.usdc_token_client.address.clone(), deposit_amount),
                ],
            );
    }
}
