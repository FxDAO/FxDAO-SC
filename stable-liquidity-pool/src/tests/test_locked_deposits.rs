#![cfg(test)]

use crate::storage::core::{CoreStorageFunc, LockingState};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::arbitrary::std::println;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

struct GovTestData {
    user1: Address,
    user1_deposit: u128,
    user2: Address,
    user2_deposit: u128,
    user3: Address,
    user3_deposit: u128,
    user4: Address,
    user4_deposit: u128,
}

fn create_locked_test_data(e: &Env) -> GovTestData {
    GovTestData {
        user1: Address::generate(&e),
        user1_deposit: 500_0000000,
        user2: Address::generate(&e),
        user2_deposit: 125_0000000,
        user3: Address::generate(&e),
        user3_deposit: 300_0000000,
        user4: Address::generate(&e),
        user4_deposit: 225_0000000,
    }
}

#[test]
pub fn lock_deposits() {
    let e: Env = Env::default();

    let test_data: TestData = create_test_data(&e);
    init_contract(&e, &test_data);

    let locked_test_data: GovTestData = create_locked_test_data(&e);

    for user in [
        &locked_test_data.user1,
        &locked_test_data.user2,
        &locked_test_data.user3,
        &locked_test_data.user4,
    ] {
        test_data
            .usdx_token_admin_client
            .mock_all_auths()
            .mint(&user, &(test_data.minted_asset_amount as i128));
        test_data
            .usdc_token_admin_client
            .mock_all_auths()
            .mint(&user, &(test_data.minted_asset_amount as i128));
        test_data
            .usdt_token_admin_client
            .mock_all_auths()
            .mint(&user, &(test_data.minted_asset_amount as i128));
    }

    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .deposit(
            &locked_test_data.user1,
            &test_data.usdx_token_client.address,
            &locked_test_data.user1_deposit,
        );

    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .deposit(
            &locked_test_data.user2,
            &test_data.usdc_token_client.address,
            &locked_test_data.user2_deposit,
        );

    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_deposit(&locked_test_data.user1)
            .shares,
        locked_test_data.user1_deposit
    );

    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_deposit(&locked_test_data.user2)
            .shares,
        locked_test_data.user2_deposit
    );

    // At this point the share price is 1_0000000 and both user1 and user 2 have the same amount of the deposit in shares
    // Now, a user will swap 200x7 USDT into 100x7 USDC and 100x7 USDx

    // user 1 will lock its deposit now and user 2 will do it after the swap

    test_data
        .stable_liquidity_pool_contract_client
        .mock_auths(&[MockAuth {
            address: &locked_test_data.user1,
            invoke: &MockAuthInvoke {
                contract: &test_data.stable_liquidity_pool_contract_client.address,
                fn_name: "lock",
                args: (locked_test_data.user1.clone(),).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .lock(&locked_test_data.user1);

    let first_client: Address = Address::generate(&e);
    test_data
        .usdt_token_admin_client
        .mock_all_auths()
        .mint(&first_client, &200_0000000);

    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .swap(
            &first_client,
            &test_data.usdt_token_client.address,
            &test_data.usdx_token_client.address,
            &100_0000000,
        );
    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .swap(
            &first_client,
            &test_data.usdt_token_client.address,
            &test_data.usdc_token_client.address,
            &100_0000000,
        );

    // User first_client should now have usdc and usdx minus the swap fee of 0.3% so we check that's correct
    assert_eq!(
        test_data.usdx_token_client.balance(&first_client),
        99_7000000
    );
    assert_eq!(
        test_data.usdx_token_client.balance(&first_client),
        99_7000000
    );

    test_data
        .stable_liquidity_pool_contract_client
        .mock_auths(&[MockAuth {
            address: &locked_test_data.user2,
            invoke: &MockAuthInvoke {
                contract: &test_data.stable_liquidity_pool_contract_client.address,
                fn_name: "lock",
                args: (locked_test_data.user2.clone(),).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .lock(&locked_test_data.user2);

    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_deposit(&locked_test_data.user1)
            .locked,
        true
    );
    assert_eq!(
        test_data
            .stable_liquidity_pool_contract_client
            .get_deposit(&locked_test_data.user1)
            .unlocks_at,
        e.ledger().timestamp() + (3600 * 24 * 7)
    );

    e.ledger().set(LedgerInfo {
        timestamp: e.ledger().timestamp() + (3600 * 24),
        protocol_version: 1,
        sequence_number: e.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    let distributor: Address = Address::generate(&e);
    test_data
        .governance_token_admin_client
        .mock_all_auths()
        .mint(&distributor, &(7000_0000000 * 3));

    test_data
        .stable_liquidity_pool_contract_client
        .mock_auths(&[MockAuth {
            address: &distributor,
            invoke: &MockAuthInvoke {
                contract: &test_data.stable_liquidity_pool_contract_client.address,
                fn_name: "distribute",
                args: (distributor.clone(), 7000_0000000u128).into_val(&e),
                sub_invokes: &[MockAuthInvoke {
                    contract: &test_data.governance_token_client.address,
                    fn_name: "transfer",
                    args: (
                        distributor.clone(),
                        test_data
                            .stable_liquidity_pool_contract_client
                            .address
                            .clone(),
                        7000_0000000i128,
                    )
                        .into_val(&e),
                    sub_invokes: &[],
                }],
            },
        }])
        .distribute(&distributor, &7000_0000000);

    e.as_contract(
        &test_data.stable_liquidity_pool_contract_client.address,
        || {
            let state: LockingState = e._locking_state().unwrap();
            assert_eq!(
                state.total,
                locked_test_data.user1_deposit + locked_test_data.user2_deposit
            );
            assert_eq!(state.factor, 11_2000000);
        },
    );

    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .deposit(
            &locked_test_data.user3,
            &test_data.usdx_token_client.address,
            &locked_test_data.user3_deposit,
        );

    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .deposit(
            &locked_test_data.user4,
            &test_data.usdc_token_client.address,
            &locked_test_data.user4_deposit,
        );

    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .lock(&locked_test_data.user3);

    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .lock(&locked_test_data.user4);

    // At this point there is a total of 625_0000000 + 524_7481208 shares

    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .distribute(&distributor, &7000_0000000);

    e.as_contract(
        &test_data.stable_liquidity_pool_contract_client.address,
        || {
            let state: LockingState = e._locking_state().unwrap();
            assert_eq!(state.total, 625_0000000 + 524_7481208);
            assert_eq!(state.factor, 17_2882900);
        },
    );

    e.ledger().set(LedgerInfo {
        timestamp: e.ledger().timestamp() + (3600 * 24 * 7),
        protocol_version: 1,
        sequence_number: e.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    test_data
        .stable_liquidity_pool_contract_client
        .mock_auths(&[MockAuth {
            address: &locked_test_data.user1,
            invoke: &MockAuthInvoke {
                contract: &test_data.stable_liquidity_pool_contract_client.address,
                fn_name: "unlock",
                args: (locked_test_data.user1.clone(),).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .unlock(&locked_test_data.user1);
    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .unlock(&locked_test_data.user2);
    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .unlock(&locked_test_data.user3);
    test_data
        .stable_liquidity_pool_contract_client
        .mock_all_auths()
        .unlock(&locked_test_data.user4);

    assert_eq!(
        test_data
            .governance_token_client
            .balance(&locked_test_data.user1),
        8644_1450000
    );
    assert_eq!(
        test_data
            .governance_token_client
            .balance(&locked_test_data.user2),
        2161_0362500
    );
    assert_eq!(
        test_data
            .governance_token_client
            .balance(&locked_test_data.user3),
        1825_6107063
    );
    assert_eq!(
        test_data
            .governance_token_client
            .balance(&locked_test_data.user4),
        1369_2080300
    );
}
