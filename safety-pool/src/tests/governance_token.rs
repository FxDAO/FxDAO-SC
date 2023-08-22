#![cfg(test)]

use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use crate::tests::utils::{create_test_data, create_token_contract, init_contract, TestData};
use crate::vaults;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{vec, Address, Env, Symbol, Vec};

#[test]
fn distribute_governance_token() {
    let env: Env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    init_contract(&test_data);
    test_data
        .governance_asset_client_admin
        .mint(&test_data.contract_client.address, &1_000_000_0000000);

    let mint_amount: i128 = 1000_0000000;

    let depositor_1: Address = Address::random(&env);
    let depositor_2: Address = Address::random(&env);
    let depositor_3: Address = Address::random(&env);

    env.ledger().set(LedgerInfo {
        timestamp: 3601 * 48,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_expiration: 0,
        min_persistent_entry_expiration: 0,
        max_entry_expiration: 0,
    });

    for depositor in [&depositor_1, &depositor_2] {
        test_data
            .deposit_asset_client_admin
            .mint(&depositor, &mint_amount);

        test_data
            .contract_client
            .deposit(&depositor, &(mint_amount as u128));
    }

    assert_eq!(test_data.contract_client.last_gov_distribution_time(), 0);

    test_data
        .contract_client
        .distribute_governance_token(&test_data.contract_admin);

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() * 3,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_expiration: 0,
        min_persistent_entry_expiration: 0,
        max_entry_expiration: 0,
    });

    assert_eq!(
        test_data.contract_client.last_gov_distribution_time(),
        3601 * 48
    );

    test_data
        .deposit_asset_client_admin
        .mint(&depositor_3, &mint_amount);

    test_data
        .contract_client
        .deposit(&depositor_3, &((mint_amount / 2) as u128));

    test_data
        .contract_client
        .distribute_governance_token(&test_data.contract_admin);

    // We check that the first two depositors received their rewards while third one didn't
    // The first two should receive 4109_5000000 each because they are the only ones and both deposited the same amount
    for depositor in [&depositor_1, &depositor_2] {
        assert_eq!(
            test_data.governance_asset_client.balance(&depositor),
            4109_5000000
        );
    }

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() * 3,
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
        .distribute_governance_token(&test_data.contract_admin);

    assert_eq!(
        test_data
            .governance_asset_client
            .balance(&test_data.contract_client.address),
        1_000_000_0000000 - (8219_0000000 * 2)
    );

    // The new distribution should be:
    // Depositor 1 and 2 should receive 2739_3927000
    // Depositor 3 should receive 1643_8000000
    for depositor in [&depositor_1, &depositor_2] {
        assert_eq!(
            test_data.governance_asset_client.balance(&depositor),
            4109_5000000 + 3287_6000000
        );
    }
    assert_eq!(
        test_data.governance_asset_client.balance(&depositor_3),
        1643_8000000,
    );
}
