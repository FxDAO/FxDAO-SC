#![cfg(test)]

use crate::tests::utils::{create_test_data, create_token_contract, init_contract, TestData};
use crate::vaults;
use crate::vaults::{OptionalVaultKey, VaultKey};
use soroban_sdk::arbitrary::std::println;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{symbol_short, vec, Address, Env, Vec};

use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use crate::storage::core::CoreStats;
use crate::storage::deposits::Deposit;

#[test]
fn fully_test_complex_liquidations_rewards_flow() {
    let env: Env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited(); // We reset the budget

    let test_data: TestData = create_test_data(&env);
    init_contract(&test_data);

    let treasury_contract: Address = Address::random(&env);

    let governance_token_admin: Address = Address::random(&env);
    let (governance_token_client, governance_token_client_admin) =
        create_token_contract(&env, &governance_token_admin);

    // Register and start vaults' contract
    let currency_price: u128 = 1_0958840;
    let xlm_token_admin: Address = Address::random(&env);
    let (xlm_token_client, xlm_token_client_admin) = create_token_contract(&env, &xlm_token_admin);

    let usd_token_admin: Address = Address::random(&env);
    let (usd_token_client, usd_token_client_admin) = create_token_contract(&env, &usd_token_admin);
    let usd_token_denomination = symbol_short!("usd");

    let vaults_contract_address: Address = env.register_contract_wasm(None, vaults::WASM);
    let vaults_contract_client = vaults::Client::new(&env, &vaults_contract_address);
    let vaults_contract_admin: Address = Address::random(&env);
    let min_collateral_rate: u128 = 1_1000000;
    let opening_debt_amount: u128 = 1_0000000;
    let opening_collateral_rate: u128 = 1_1500000;

    vaults_contract_client.init(
        &vaults_contract_admin,
        &vaults_contract_admin,
        &vaults_contract_admin,
        &xlm_token_client.address,
        &usd_token_admin,
    );

    vaults_contract_client.create_currency(&usd_token_denomination, &usd_token_client.address);

    vaults_contract_client.set_currency_rate(&usd_token_denomination, &currency_price);

    vaults_contract_client.toggle_currency(&usd_token_denomination, &true);

    vaults_contract_client.set_vault_conditions(
        &min_collateral_rate,
        &opening_debt_amount,
        &opening_collateral_rate,
        &usd_token_denomination,
    );

    usd_token_client.approve(
        &usd_token_admin,
        &vaults_contract_address,
        &90000000000000000000,
        &200_000,
    );

    usd_token_client_admin.mint(&usd_token_admin, &90000000000000000000);

    // Register the pool contract
    let pool_contract_id: Address = env.register_contract(None, SafetyPoolContract);
    let pool_contract_client = SafetyPoolContractClient::new(&env, &pool_contract_id);
    let pool_contract_admin: Address = Address::random(&env);
    let min_pool_deposit: u128 = 100_0000000;
    let profit_share: Vec<u32> = vec![&env, 1u32, 2u32] as Vec<u32>;
    let liquidator_share: Vec<u32> = vec![&env, 1u32, 2u32] as Vec<u32>;

    pool_contract_client.init(
        &pool_contract_admin,
        &vaults_contract_address,
        &treasury_contract,
        &xlm_token_client.address,
        &usd_token_client.address,
        &usd_token_denomination,
        &min_pool_deposit,
        &profit_share,
        &liquidator_share,
        &governance_token_client.address,
    );

    // We create the first vault depositor
    let vault_depositor_1: Address = Address::random(&env);
    xlm_token_client_admin.mint(&vault_depositor_1, &45833_3333333);
    usd_token_client_admin.mint(&vault_depositor_1, &5500_0000000);

    vaults_contract_client.new_vault(
        &OptionalVaultKey::None,
        &vault_depositor_1,
        &5500_0000000,
        &45833_3333333,
        &usd_token_denomination,
    );

    // We drop the collateral price so it can be liquidated
    vaults_contract_client.set_currency_rate(&usd_token_denomination, &0_0900000);

    // Phase 1: 3 depositors with different values each
    let depositor_1: Address = Address::random(&env);
    usd_token_client_admin.mint(&depositor_1, &5000_0000000);
    pool_contract_client.deposit(&depositor_1, &5000_0000000);

    let depositor_2: Address = Address::random(&env);
    usd_token_client_admin.mint(&depositor_2, &7500_0000000);
    pool_contract_client.deposit(&depositor_2, &7500_0000000);

    let depositor_3: Address = Address::random(&env);
    usd_token_client_admin.mint(&depositor_3, &3725_0000000);
    pool_contract_client.deposit(&depositor_3, &3725_0000000);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            lifetime_deposited: 16225_0000000,
            current_deposited: 16225_0000000,
            lifetime_profit: 0,
            lifetime_liquidated: 0,
            current_liquidated: 0,
            collateral_factor: 0,
            deposit_factor: 1_0000000,
        }
    );

    // Phase 2: We liquidate the vault
    let liquidator_1: Address = Address::random(&env);
    pool_contract_client.liquidate(&liquidator_1);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            lifetime_deposited: 16225_0000000,
            current_deposited: 10725_0000000,
            lifetime_profit: 0,
            lifetime_liquidated: 45833_3333333,
            current_liquidated: 45833_3333333,
            collateral_factor: 2_8248587,
            deposit_factor: 0_6610170,
        }
    );

    // Phase 3: 2 new depositors in the pool
    let depositor_4: Address = Address::random(&env);
    usd_token_client_admin.mint(&depositor_4, &8000_0000000);
    pool_contract_client.deposit(&depositor_4, &8000_0000000);

    assert_eq!(
        pool_contract_client
            .get_deposit(&depositor_4)
            .current_collateral_factor,
        2_8248587
    );

    let depositor_5: Address = Address::random(&env);
    usd_token_client_admin.mint(&depositor_5, &6000_0000000);
    pool_contract_client.deposit(&depositor_5, &6000_0000000);

    assert_eq!(
        pool_contract_client
            .get_deposit(&depositor_5)
            .current_collateral_factor,
        2_8248587
    );

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            lifetime_deposited: 30225_0000000,
            current_deposited: 24725_0000000,
            lifetime_profit: 0,
            lifetime_liquidated: 45833_3333333,
            current_liquidated: 45833_3333333,
            collateral_factor: 2_8248587,
            deposit_factor: 0_6610170,
        }
    );

    // Phase 4: We make another liquidation

    // First let's set the new currency price (this time we want to have a profit from the liquidation)
    vaults_contract_client.set_currency_rate(&usd_token_denomination, &0_1815000);

    let vault_depositor_2: Address = Address::random(&env);
    let vault_depositor_2_debt: u128 = 8000_0000000;
    let vault_depositor_2_collateral: u128 = 72727_2727300;
    xlm_token_client_admin.mint(&vault_depositor_2, &(vault_depositor_2_collateral as i128));
    usd_token_client_admin.mint(&vault_depositor_2, &(vault_depositor_2_debt as i128));

    vaults_contract_client.new_vault(
        &OptionalVaultKey::None,
        &vault_depositor_2,
        &vault_depositor_2_debt,
        &vault_depositor_2_collateral,
        &usd_token_denomination,
    );

    // We set the new currency price so it can be liquidated
    vaults_contract_client.set_currency_rate(&usd_token_denomination, &0_1200000);

    // We liquidate the second vault
    let liquidator_2: Address = Address::random(&env);
    pool_contract_client.liquidate(&liquidator_2);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            lifetime_deposited: 30225_0000000,
            current_deposited: 16725_0000000,
            lifetime_profit: 6060_6060634,
            lifetime_liquidated: 115530_3030316,
            current_liquidated: 115530_3030316,
            collateral_factor: 4_6881906,
            deposit_factor: 0_4471389,
        }
    );

    // Phase 5: Another new depositor and one depositor leaving

    assert_eq!(usd_token_client.balance(&depositor_1), 0);
    assert_eq!(xlm_token_client.balance(&depositor_1), 0);

    env.ledger().set(LedgerInfo {
        timestamp: 172801,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_expiration: 0,
        min_persistent_entry_expiration: 0,
        max_entry_expiration: 0,
    });

    pool_contract_client.withdraw(&depositor_1);

    let depositor_6: Address = Address::random(&env);
    usd_token_client_admin.mint(&depositor_6, &7500_0000000);
    pool_contract_client.deposit(&depositor_6, &7500_0000000);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            lifetime_deposited: 37725_0000000,
            current_deposited: 21989_3055000,
            lifetime_profit: 6060_6060634,
            lifetime_liquidated: 115530_3030316,
            current_liquidated: 92089_3500316,
            collateral_factor: 4_6881906,
            deposit_factor: 0_4471389,
        }
    );

    // Phase 6: Everybody withdraws the funds

    pool_contract_client.withdraw(&depositor_2);
    pool_contract_client.withdraw(&depositor_3);
    pool_contract_client.withdraw(&depositor_4);
    pool_contract_client.withdraw(&depositor_5);

    env.ledger().set(LedgerInfo {
        timestamp: 172801 * 100,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_expiration: 0,
        min_persistent_entry_expiration: 0,
        max_entry_expiration: 0,
    });

    pool_contract_client.withdraw(&depositor_6);

    println!("{:?}", xlm_token_client.balance(&depositor_2));
    println!("{:?}", xlm_token_client.balance(&depositor_3));
    println!("{:?}", xlm_token_client.balance(&depositor_4));
    println!("{:?}", xlm_token_client.balance(&depositor_5));
    println!("{:?}", xlm_token_client.balance(&depositor_6));

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            lifetime_deposited: 37725_0000000,
            current_deposited: 0,
            lifetime_profit: 6060_6060634,
            lifetime_liquidated: 115530_3030316,
            current_liquidated: 115469_9573372,
            collateral_factor: 4_6881906,
            deposit_factor: 1_0000000,
        }
    );

    assert_eq!(usd_token_client.balance(&pool_contract_client.address), 0);
    assert_eq!(
        xlm_token_client.balance(&pool_contract_client.address),
        0_0036416
    );

    // assert_eq!(xlm_token_client.balance(&depositor_2), 38480_9730000);
    // assert_eq!(xlm_token_client.balance(&depositor_3), 19112_2165900);
    // assert_eq!(xlm_token_client.balance(&depositor_4), 18447_5016000);
    // assert_eq!(xlm_token_client.balance(&depositor_5), 13835_6262000);
    // assert_eq!(usd_token_client.balance(&depositor_5), 4058_6450793);
    // assert_eq!(xlm_token_client.balance(&depositor_6), 0);
}
