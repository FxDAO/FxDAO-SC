#![cfg(test)]
use crate::storage::core::CoreState;
use crate::storage::deposits::Deposit;
use crate::tests::test_utils::{create_test_data, init_contract, prepare_test_accounts, TestData};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{map, vec, Address, Env, IntoVal, Status, Symbol, Vec};

#[test]
fn test_swaps_and_profit_retiring() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_contract(&env, &test_data);

    let deposit_amount: u128 = 100_0000000;
    let depositor_1: Address = Address::random(&env);
    let depositor_2: Address = Address::random(&env);
    let depositor_3: Address = Address::random(&env);
    let depositor_4: Address = Address::random(&env);
    let depositors: Vec<Address> = vec![
        &env,
        depositor_1.clone(),
        depositor_2.clone(),
        depositor_3.clone(),
        depositor_4.clone(),
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

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_3,
        &test_data.usdx_token_client.address,
        &deposit_amount,
    );

    let customer_1: Address = Address::random(&env);

    test_data
        .usdc_token_client
        .mint(&customer_1, &(deposit_amount as i128));

    assert_eq!(
        &(test_data.usdc_token_client.balance(&customer_1) as u128),
        &deposit_amount
    );

    test_data.stable_liquidity_pool_contract_client.swap(
        &customer_1,
        &test_data.usdc_token_client.address,
        &test_data.usdx_token_client.address,
        &deposit_amount,
    );

    assert_eq!(
        [env.auths().first().unwrap()],
        [&(
            // Address for which auth is performed
            customer_1.clone(),
            // Identifier of the called contract
            test_data
                .stable_liquidity_pool_contract_client
                .address
                .clone(),
            // Name of the called function
            Symbol::short("swap"),
            // Arguments used (converted to the env-managed vector via `into_val`)
            (
                customer_1.clone(),
                test_data.usdc_token_client.address.clone(),
                test_data.usdx_token_client.address.clone(),
                deposit_amount.clone(),
            )
                .into_val(&env),
        )]
    );

    assert_eq!(&test_data.usdc_token_client.balance(&customer_1), &0);
    assert_eq!(
        &test_data.usdx_token_client.balance(&customer_1),
        &99_7000000
    );

    let core_state: CoreState = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    assert_eq!(&core_state.share_price, &1_0005000);
    assert_eq!(&core_state.total_shares, &300_0000000);
    assert_eq!(&core_state.total_deposited, &300_1500000);
    assert_eq!(
        &test_data
            .usdc_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        &1998500000
    );
    assert_eq!(
        &test_data
            .usdx_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        &(100_0000000 - 99_7000000)
    );

    let customer_2: Address = Address::random(&env);

    test_data
        .usdt_token_client
        .mint(&customer_2, &(deposit_amount as i128));

    test_data.stable_liquidity_pool_contract_client.swap(
        &customer_2,
        &test_data.usdt_token_client.address,
        &test_data.usdc_token_client.address,
        &deposit_amount,
    );

    assert_eq!(
        &test_data
            .stable_liquidity_pool_contract_client
            .get_core_state()
            .share_price,
        &1_0010000
    );
    assert_eq!(
        &test_data
            .stable_liquidity_pool_contract_client
            .get_core_state()
            .total_shares,
        &300_0000000
    );
    assert_eq!(
        &test_data
            .stable_liquidity_pool_contract_client
            .get_core_state()
            .total_deposited,
        &300_3000000
    );
    assert_eq!(
        &test_data
            .usdt_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        &199_8500000
    );
    assert_eq!(
        &test_data
            .usdc_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        &(199_8500000 - 99_7000000)
    );

    test_data.stable_liquidity_pool_contract_client.deposit(
        &depositor_4,
        &test_data.usdc_token_client.address,
        &deposit_amount,
    );

    assert_eq!(
        &test_data
            .usdc_token_client
            .balance(&test_data.stable_liquidity_pool_contract_client.address),
        &200_1500000
    );

    let updated_core_state: CoreState = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    assert_eq!(&updated_core_state.share_price, &1_0010000);
    assert_eq!(&updated_core_state.total_shares, &399_9000999);
    assert_eq!(&updated_core_state.total_deposited, &400_3000000);

    // At this point depositor 1, 2 and 3 have 100_0000000 shares each
    // While depositor 4 has 99_9000999
    // The contract currently has the next balances:
    // USDC: 200_1500000
    // USDT: 199_8500000
    // USDx:   0_3000000
    // We now start withdrawing the funds and we check the swaps profits

    env.ledger().set(LedgerInfo {
        timestamp: 3600 * 50,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    test_data.stable_liquidity_pool_contract_client.withdraw(
        &depositor_1,
        &100_0000000,
        &map![
            &env,
            (test_data.usdc_token_client.address.clone(), 50_0500000),
            (test_data.usdt_token_client.address.clone(), 50_0500000),
            (test_data.usdx_token_client.address.clone(), 0),
        ],
    );

    let deposit_1: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_1);

    assert_eq!(deposit_1.shares, 0);

    test_data.stable_liquidity_pool_contract_client.withdraw(
        &depositor_2,
        &100_0000000,
        &map![
            &env,
            (test_data.usdc_token_client.address.clone(), 50_0500000),
            (test_data.usdt_token_client.address.clone(), 49_7500000),
            (test_data.usdx_token_client.address.clone(), 00_3000000),
        ],
    );

    let deposit_2: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_2);

    assert_eq!(deposit_2.shares, 0);

    test_data.stable_liquidity_pool_contract_client.withdraw(
        &depositor_3,
        &100_0000000,
        &map![
            &env,
            (test_data.usdc_token_client.address.clone(), 50_0500000),
            (test_data.usdt_token_client.address.clone(), 50_0500000),
            (test_data.usdx_token_client.address.clone(), 00_0000000),
        ],
    );

    let deposit_3: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_3);

    assert_eq!(deposit_3.shares, 0);

    test_data.stable_liquidity_pool_contract_client.withdraw(
        &depositor_4,
        &99_9000999,
        &map![
            &env,
            (test_data.usdc_token_client.address.clone(), 50_0000000),
            (test_data.usdt_token_client.address.clone(), 50_0000000),
            (test_data.usdx_token_client.address.clone(), 00_0000000),
        ],
    );

    let deposit_4: Deposit = test_data
        .stable_liquidity_pool_contract_client
        .get_deposit(&depositor_4);

    assert_eq!(deposit_4.shares, 0);

    let last_usdc_balance = test_data
        .usdc_token_client
        .balance(&test_data.stable_liquidity_pool_contract_client.address);

    assert_eq!(last_usdc_balance, 0);

    let last_usdt_balance = test_data
        .usdt_token_client
        .balance(&test_data.stable_liquidity_pool_contract_client.address);

    assert_eq!(last_usdt_balance, 0);

    let last_usdx_balance = test_data
        .usdx_token_client
        .balance(&test_data.stable_liquidity_pool_contract_client.address);

    assert_eq!(last_usdx_balance, 0);

    let last_core_state: CoreState = test_data
        .stable_liquidity_pool_contract_client
        .get_core_state();

    assert_eq!(last_core_state.total_deposited, 0);
    assert_eq!(last_core_state.share_price, 1_0000000);
    assert_eq!(last_core_state.total_shares, 0);
}
