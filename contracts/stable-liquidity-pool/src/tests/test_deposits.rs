#![cfg(test)]

use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::storage::deposits::Deposit;
use crate::tests::test_utils::{create_test_data, init_contract, prepare_test_accounts, TestData};
use soroban_sdk::testutils::arbitrary::std;
use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger, LedgerInfo,
};
use soroban_sdk::{map, symbol_short, Address, Env, IntoVal, Vec};

#[test]
pub fn test_deposits() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_contract(&env, &test_data);

    let deposit_amount: u128 = 100_0000000;
    let depositor_1: Address = Address::generate(&env);
    let depositor_2: Address = Address::generate(&env);
    let depositor_3: Address = Address::generate(&env);
    let depositors: Vec<Address> = Vec::from_array(
        &env,
        [
            depositor_1.clone(),
            depositor_2.clone(),
            depositor_3.clone(),
        ],
    );

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

    let mut core_state: CoreState = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    let deposit_1: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_1);

    assert_eq!(&deposit_1.unlocks_at, &(0 + (3600 * 48)));
    assert_eq!(&deposit_1.depositor, &depositor_1);
    assert_eq!(&deposit_1.shares, &deposit_amount);
    assert_eq!(
        &test_data.usdc_token_client.balance(&depositor_1),
        &((test_data.minted_asset_amount - deposit_amount) as i128),
    );
    assert_eq!(&core_state.total_deposited, &deposit_amount);

    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_2,
        &test_data.usdt_token_client.address,
        &deposit_amount,
    );

    let deposit_2: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_2);

    assert_eq!(&deposit_2.unlocks_at, &(1000 + (3600 * 48)));
    assert_eq!(&deposit_2.depositor, &depositor_2);
    assert_eq!(&deposit_2.shares, &deposit_amount);
    assert_eq!(
        &test_data.usdt_token_client.balance(&depositor_2),
        &((test_data.minted_asset_amount - deposit_amount) as i128),
    );

    core_state = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    assert_eq!(&core_state.total_deposited, &(deposit_amount * 2));

    env.ledger().set(LedgerInfo {
        timestamp: 2000,
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_3,
        &test_data.usdx_token_client.address,
        &deposit_amount,
    );

    let deposit_3: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_3);

    assert_eq!(&deposit_3.unlocks_at, &(2000 + (3600 * 48)));
    assert_eq!(&deposit_3.depositor, &depositor_3);
    assert_eq!(&deposit_3.shares, &deposit_amount);
    assert_eq!(
        &test_data.usdx_token_client.balance(&depositor_3),
        &((test_data.minted_asset_amount - deposit_amount) as i128),
    );

    core_state = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    assert_eq!(&core_state.total_deposited, &(deposit_amount * 3));
}

#[test]
fn test_simple_withdrawals() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_contract(&env, &test_data);

    let deposit_amount: u128 = 100_0000000;
    let depositor_1: Address = Address::generate(&env);
    let depositor_2: Address = Address::generate(&env);
    let depositor_3: Address = Address::generate(&env);
    let depositors: Vec<Address> = Vec::from_array(
        &env,
        [
            depositor_1.clone(),
            depositor_2.clone(),
            depositor_3.clone(),
        ],
    );

    prepare_test_accounts(&test_data, &depositors);

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_1,
        &test_data.usdc_token_client.address,
        &deposit_amount,
    );

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_2,
        &test_data.usdt_token_client.address,
        &deposit_amount,
    );

    let nothing_to_withdraw = test_data
        .stable_liquidity_pool_contract_client
        .try_withdraw(
            &depositor_3,
            &51_0000000,
            &map![
                &env,
                (test_data.usdx_token_client.address.clone(), 50_0000000),
                (test_data.usdc_token_client.address.clone(), 0),
                (test_data.usdt_token_client.address.clone(), 1),
            ],
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(nothing_to_withdraw, SCErrors::NothingToWithdraw.into());

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_3,
        &test_data.usdx_token_client.address,
        &deposit_amount,
    );

    let locker_period_uncompleted = test_data
        .stable_liquidity_pool_contract_client
        .try_withdraw(
            &depositor_3,
            &51_0000000,
            &map![
                &env,
                (test_data.usdx_token_client.address.clone(), 50_0000000),
                (test_data.usdc_token_client.address.clone(), 1),
                (test_data.usdt_token_client.address.clone(), 0),
            ],
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(
        locker_period_uncompleted,
        SCErrors::LockedPeriodUncompleted.into()
    );

    env.ledger().set(LedgerInfo {
        timestamp: (3600 * 49),
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    let core_state: CoreState = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    assert_eq!(&core_state.total_deposited, &300_0000000);
    assert_eq!(
        test_data
            .usdc_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        100_0000000
    );
    assert_eq!(
        test_data
            .usdt_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        100_0000000
    );
    assert_eq!(
        test_data
            .usdx_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        100_0000000
    );

    test_data.stable_liquidity_pool_contract_client.withdraw(
        &depositor_3,
        &50_0000000,
        &map![
            &env,
            (test_data.usdc_token_client.address.clone(), 50_0000000),
            (test_data.usdx_token_client.address.clone(), 0),
            (test_data.usdt_token_client.address.clone(), 0),
        ],
    );

    let mut deposit_3: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_3);

    assert_eq!(&deposit_3.unlocks_at, &(0 + (3600 * 48)));
    assert_eq!(&deposit_3.shares, &50_0000000);
    assert_eq!(&deposit_3.depositor, &depositor_3);
    assert_eq!(
        &(test_data.usdc_token_client.balance(&depositor_3) as u128),
        &(test_data.minted_asset_amount + 50_0000000)
    );
    assert_eq!(
        &(test_data.usdx_token_client.balance(&depositor_3) as u128),
        &(test_data.minted_asset_amount - 100_0000000)
    );
    assert_eq!(
        test_data
            .usdc_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        50_0000000
    );

    test_data.stable_liquidity_pool_contract_client.withdraw(
        &depositor_3,
        &50_0000000,
        &map![
            &env,
            (test_data.usdc_token_client.address.clone(), 0),
            (test_data.usdx_token_client.address.clone(), 0),
            (test_data.usdt_token_client.address.clone(), 50_0000000),
        ],
    );

    deposit_3 = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_3);

    assert_eq!(&deposit_3.unlocks_at, &0);
    assert_eq!(&deposit_3.shares, &0);
    assert_eq!(
        &(test_data.usdt_token_client.balance(&depositor_3) as u128),
        &(test_data.minted_asset_amount + 50_0000000)
    );
    assert_eq!(
        test_data
            .usdt_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        50_0000000
    );

    assert_eq!(
        &test_data
            .stable_liquidity_pool_contract_client
            .get_core_state()
            .total_deposited,
        &200_0000000
    );

    assert_eq!(
        &test_data
            .stable_liquidity_pool_contract_client
            .get_core_state()
            .total_shares,
        &200_0000000
    );

    test_data.stable_liquidity_pool_contract_client.withdraw(
        &depositor_2,
        &100_0000000,
        &map![
            &env,
            (test_data.usdc_token_client.address.clone(), 50_0000000),
            (test_data.usdx_token_client.address.clone(), 0),
            (test_data.usdt_token_client.address.clone(), 50_0000000),
        ],
    );

    let not_enough_error = test_data
        .stable_liquidity_pool_contract_client
        .try_withdraw(
            &depositor_1,
            &101_0000000,
            &map![
                &env,
                (test_data.usdc_token_client.address.clone(), 0),
                (test_data.usdx_token_client.address.clone(), 101_0000000),
                (test_data.usdt_token_client.address.clone(), 0),
            ],
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(not_enough_error, SCErrors::NotEnoughSharesToWithdraw.into());

    test_data.stable_liquidity_pool_contract_client.withdraw(
        &depositor_1,
        &100_0000000,
        &map![
            &env,
            (test_data.usdc_token_client.address.clone(), 0),
            (test_data.usdx_token_client.address.clone(), 100_0000000),
            (test_data.usdt_token_client.address.clone(), 0),
        ],
    );

    let last_core_state: CoreState = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    assert_eq!(&last_core_state.share_price, &1_0000000);
    assert_eq!(&last_core_state.total_deposited, &0);
    assert_eq!(&last_core_state.total_shares, &0);
}

#[test]
fn test_share_price_withdrawals() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_contract(&env, &test_data);

    let total_liquidity_amount: u128 = 1234_5678987;

    let depositor: Address = Address::generate(&env);
    test_data
        .usdx_token_admin_client
        .mint(&depositor, &(total_liquidity_amount as i128));

    let swapper: Address = Address::generate(&env);
    test_data
        .usdc_token_admin_client
        .mint(&swapper, &(total_liquidity_amount as i128 * 11));

    let attacker: Address = Address::generate(&env);
    let attacker_deposit_amount: u128 = 100_0000000u128;
    test_data
        .usdt_token_admin_client
        .mint(&attacker, &(attacker_deposit_amount as i128));

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor,
        &test_data.usdx_token_client.address,
        &total_liquidity_amount,
    );

    for i in 0..10 {
        let from_asset: Address = if i % 2 == 0 {
            test_data.usdc_token_client.address.clone()
        } else {
            test_data.usdx_token_client.address.clone()
        };

        let to_asset: Address = if i % 2 == 0 {
            test_data.usdx_token_client.address.clone()
        } else {
            test_data.usdc_token_client.address.clone()
        };

        // The (37037037 * i) is just a simple way to reduce the amount to swap because we know the protocol keeps the fees in the input asset
        let swap_amount: u128 = total_liquidity_amount - (37037037 * i);

        test_data.stable_liquidity_pool_contract_client.swap(
            &swapper,
            &from_asset,
            &to_asset,
            &swap_amount,
        );
    }

    test_data.stable_liquidity_pool_contract_client.deposit(
        &attacker,
        &test_data.usdt_token_client.address,
        &attacker_deposit_amount,
    );

    env.ledger().set(LedgerInfo {
        timestamp: 3600 * 24 * 3,
        protocol_version: 1,
        sequence_number: env.ledger().sequence() + 1,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    let attacker_deposit: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&attacker);

    let pool_core_state: CoreState = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    // The attacker needs to only be able to withdraw its deposit initial value or a lower amount
    assert!(
        attacker_deposit_amount
            >= (attacker_deposit.shares * pool_core_state.total_deposited)
                / pool_core_state.total_shares
    );
}
