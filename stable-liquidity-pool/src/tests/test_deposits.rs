#![cfg(test)]
use crate::storage::core::CoreState;
use crate::storage::deposits::Deposit;
use crate::tests::test_utils::{create_test_data, init_contract, prepare_test_accounts, TestData};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{map, vec, Address, Env, IntoVal, Status, Symbol, Vec};

#[test]
pub fn test_deposits() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_contract(&env, &test_data);

    let deposit_amount: u128 = 100_0000000;
    let depositor_1: Address = Address::random(&env);
    let depositor_2: Address = Address::random(&env);
    let depositor_3: Address = Address::random(&env);
    let depositors: Vec<Address> = vec![
        &env,
        depositor_1.clone(),
        depositor_2.clone(),
        depositor_3.clone(),
    ] as Vec<Address>;

    prepare_test_accounts(&test_data, &depositors);

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_1,
        &test_data.usdc_token_client.address,
        &deposit_amount,
    );

    let current_auths = env.auths();
    assert_eq!(
        [current_auths.first().unwrap()],
        [&(
            // Address for which auth is performed
            depositor_1.clone(),
            // Identifier of the called contract
            test_data
                .stable_liquidity_pool_contract_client
                .address
                .clone(),
            // Name of the called function
            Symbol::short("deposit"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (
                depositor_1.clone(),
                test_data.usdc_token_client.address.clone(),
                deposit_amount.clone()
            )
                .into_val(&env),
        )]
    );

    let mut core_state: CoreState = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    let deposit_1: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_1);

    assert_eq!(&deposit_1.last_deposit, &0);
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
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_2,
        &test_data.usdt_token_client.address,
        &deposit_amount,
    );

    let deposit_2: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_2);

    assert_eq!(&deposit_2.last_deposit, &1000);
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
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_3,
        &test_data.usdx_token_client.address,
        &deposit_amount,
    );

    let deposit_3: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_3);

    assert_eq!(&deposit_3.last_deposit, &2000);
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

    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_depositors(),
        vec![&env, depositor_1, depositor_2, depositor_3]
    );
}

#[test]
fn test_simple_withdrawals() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_contract(&env, &test_data);

    let deposit_amount: u128 = 100_0000000;
    let depositor_1: Address = Address::random(&env);
    let depositor_2: Address = Address::random(&env);
    let depositor_3: Address = Address::random(&env);
    let depositors: Vec<Address> = vec![
        &env,
        depositor_1.clone(),
        depositor_2.clone(),
        depositor_3.clone(),
    ] as Vec<Address>;

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
        .unwrap_err();

    assert_eq!(nothing_to_withdraw, Ok(Status::from_contract_error(30001)));

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
        .unwrap_err();

    assert_eq!(
        locker_period_uncompleted,
        Ok(Status::from_contract_error(30002))
    );

    env.ledger().set(LedgerInfo {
        timestamp: (3600 * 49),
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
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

    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_depositors(),
        vec![
            &env,
            depositor_1.clone(),
            depositor_2.clone(),
            depositor_3.clone()
        ]
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

    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_depositors(),
        vec![
            &env,
            depositor_1.clone(),
            depositor_2.clone(),
            depositor_3.clone()
        ]
    );

    let mut deposit_3: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_3);

    assert_eq!(&deposit_3.last_deposit, &0);
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

    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_depositors(),
        vec![&env, depositor_1.clone(), depositor_2.clone(),]
    );

    deposit_3 = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_3);

    assert_eq!(&deposit_3.last_deposit, &0);
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
        .unwrap_err();

    assert_eq!(not_enough_error, Ok(Status::from_contract_error(30003)));

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

    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_depositors(),
        vec![&env]
    );
}
