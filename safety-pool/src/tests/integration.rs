#![cfg(test)]

use crate::tests::utils::{create_test_data, create_token_contract, init_contract, TestData};
use crate::vaults;
use crate::vaults::{OptionalVaultKey, VaultKey};
use num_integer::div_floor;
use soroban_sdk::iter::UnwrappedEnumerable;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Vec};

use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use crate::errors::SCErrors;
use crate::storage::core::CoreStats;
use crate::storage::deposits::Deposit;
use crate::storage::liquidations::Liquidation;

#[test]
fn fully_test_complex_liquidations_rewards_flow() {
    let env: Env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited(); // We reset the budget

    let test_data: TestData = create_test_data(&env);
    init_contract(&test_data);

    let treasury_contract: Address = Address::generate(&env);

    let governance_token_admin: Address = Address::generate(&env);
    let (governance_token_client, governance_token_client_admin) =
        create_token_contract(&env, &governance_token_admin);

    // Register and start vaults' contract
    let currency_price: u128 = 1_0958840;
    let xlm_token_admin: Address = Address::generate(&env);
    let (xlm_token_client, xlm_token_client_admin) = create_token_contract(&env, &xlm_token_admin);

    let usd_token_admin: Address = Address::generate(&env);
    let (usd_token_client, usd_token_client_admin) = create_token_contract(&env, &usd_token_admin);
    let usd_token_denomination = symbol_short!("usd");

    let vaults_contract_address: Address = env.register_contract_wasm(None, vaults::WASM);
    let vaults_contract_client = vaults::Client::new(&env, &vaults_contract_address);
    let vaults_contract_admin: Address = Address::generate(&env);
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

    vaults_contract_client.set_currency_rate(&usd_token_denomination, &0_1000000);

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
    let pool_contract_admin: Address = Address::generate(&env);
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
    let vault_depositor_1: Address = Address::generate(&env);
    xlm_token_client_admin.mint(&vault_depositor_1, &63250_0000000);
    usd_token_client_admin.mint(&vault_depositor_1, &5500_0000000);

    vaults_contract_client.new_vault(
        &OptionalVaultKey::None,
        &vault_depositor_1,
        &5500_0000000,
        &63250_0000000,
        &usd_token_denomination,
    );

    // Phase 1: 3 depositors with different values each
    let depositor_1: Address = Address::generate(&env);
    let depositor_1_mint: u128 = 5000_0000000;
    usd_token_client_admin.mint(&depositor_1, &(depositor_1_mint as i128));
    pool_contract_client.deposit(&depositor_1, &depositor_1_mint);

    let depositor_2: Address = Address::generate(&env);
    let depositor_2_mint: u128 = 7500_0000000;
    usd_token_client_admin.mint(&depositor_2, &(depositor_2_mint as i128));
    pool_contract_client.deposit(&depositor_2, &depositor_2_mint);

    let depositor_3: Address = Address::generate(&env);
    let depositor_3_mint: u128 = 3725_0000000;
    usd_token_client_admin.mint(&depositor_3, &(depositor_3_mint as i128));
    pool_contract_client.deposit(&depositor_3, &depositor_3_mint);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            total_deposits: 3,
            lifetime_deposited: 16225_0000000,
            current_deposited: 16225_0000000,
            lifetime_profit: 0,
            lifetime_liquidated: 0,
            liquidation_index: 0,
            total_shares: 16225_0000000,
            share_price: 1_0000000,
            rewards_factor: 0,
        }
    );

    // Phase 2: We liquidate the vault

    // We drop the collateral price so it can be liquidated
    vaults_contract_client.set_currency_rate(&usd_token_denomination, &0_0956521);

    let liquidator_1: Address = Address::generate(&env);
    pool_contract_client.liquidate(&liquidator_1);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            total_deposits: 3,
            lifetime_deposited: 16225_0000000,
            current_deposited: 10725_0000000,
            lifetime_profit: 5749_9555682,
            lifetime_liquidated: 60375_0222159,
            liquidation_index: 1,
            rewards_factor: 0,
            total_shares: 16225_0000000,
            share_price: 0_6610169,
        }
    );

    let liquidation_1: Liquidation = pool_contract_client
        .get_liquidations(&(vec![&env, 0u64] as Vec<u64>))
        .get(0)
        .unwrap();

    assert_eq!(
        liquidation_1,
        Liquidation {
            index: 0,
            total_deposits: 3,
            total_debt_paid: 5500_0000000,
            total_col_liquidated: 60375_0222159,
            col_to_withdraw: 60375_0222159,
            share_price: 1_0000000,
            total_shares: 16225_0000000,
            shares_redeemed: 0,
        }
    );

    // Phase 3: 2 new depositors in the pool
    let depositor_4: Address = Address::generate(&env);
    usd_token_client_admin.mint(&depositor_4, &8000_0000000);
    pool_contract_client.deposit(&depositor_4, &8000_0000000);

    let depositor_5: Address = Address::generate(&env);
    usd_token_client_admin.mint(&depositor_5, &6000_0000000);
    pool_contract_client.deposit(&depositor_5, &6000_0000000);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            total_deposits: 5,
            lifetime_deposited: 30225_0000000,
            current_deposited: 24725_0000000,
            lifetime_profit: 5749_9555682,
            lifetime_liquidated: 60375_0222159,
            liquidation_index: 1,
            rewards_factor: 0,
            total_shares: 37404_4887542,
            share_price: 0_6610169,
        }
    );

    // Phase 4: We make another liquidation

    // First let's set the new currency price (this time we want to have a profit from the liquidation)
    vaults_contract_client.set_currency_rate(&usd_token_denomination, &0_1000000);

    let vault_depositor_2: Address = Address::generate(&env);
    let vault_depositor_2_debt: u128 = 8000_0000000;
    let vault_depositor_2_collateral: u128 = 92000_0000000;
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
    vaults_contract_client.set_currency_rate(&usd_token_denomination, &0_0956521);

    // We liquidate the second vault
    let liquidator_2: Address = Address::generate(&env);
    pool_contract_client.liquidate(&liquidator_2);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            total_deposits: 5,
            lifetime_deposited: 30225_0000000,
            current_deposited: 16725_0000000,
            lifetime_profit: 14113_5273037,
            lifetime_liquidated: 148193_2363482,
            liquidation_index: 2,
            total_shares: 37404_4887542,
            share_price: 0_4471388,
            rewards_factor: 0,
        }
    );

    let liquidation_2: Liquidation = pool_contract_client
        .get_liquidations(&(vec![&env, 1u64] as Vec<u64>))
        .get(0)
        .unwrap();

    assert_eq!(
        liquidation_2,
        Liquidation {
            index: 1,
            total_deposits: 5,
            total_debt_paid: 8000_0000000,
            total_col_liquidated: 87818_2141323,
            col_to_withdraw: 87818_2141323,
            share_price: 0_6610169,
            total_shares: 37404_4887542,
            shares_redeemed: 0,
        }
    );

    vaults_contract_client.set_currency_rate(&usd_token_denomination, &0_1000000);

    // Phase 5: Another new depositor and one depositor leaving

    assert_eq!(usd_token_client.balance(&depositor_1), 0);
    assert_eq!(xlm_token_client.balance(&depositor_1), 0);

    env.ledger().set(LedgerInfo {
        timestamp: 172801,
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    // We try to make the first withdraw.
    let available_claim_error = pool_contract_client
        .try_withdraw(&depositor_1)
        .unwrap_err()
        .unwrap();

    assert_eq!(
        &available_claim_error,
        &SCErrors::CollateralAvailable.into()
    );

    assert_eq!(xlm_token_client.balance(&depositor_1), 0);
    pool_contract_client.withdraw_col(&depositor_1);
    assert_eq!(xlm_token_client.balance(&depositor_1), 30344_5388565);

    pool_contract_client.withdraw(&depositor_1);

    let depositor_6: Address = Address::generate(&env);
    usd_token_client_admin.mint(&depositor_6, &7500_0000000);
    pool_contract_client.deposit(&depositor_6, &7500_0000000);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            total_deposits: 5,
            lifetime_deposited: 37725_0000000,
            current_deposited: 21989_3057616,
            lifetime_profit: 14113_5273037,
            lifetime_liquidated: 148193_2363482,
            liquidation_index: 2,
            total_shares: 49177_8038858,
            share_price: 0_4471388,
            rewards_factor: 0,
        }
    );

    // Phase 6: Everybody withdraws the funds

    assert_eq!(xlm_token_client.balance(&depositor_5), 0);
    pool_contract_client.withdraw_col(&depositor_5);
    assert_eq!(xlm_token_client.balance(&depositor_5), 21310_7885468);
    pool_contract_client.withdraw(&depositor_5);

    assert_eq!(xlm_token_client.balance(&depositor_3), 0);
    pool_contract_client.withdraw_col(&depositor_3);
    assert_eq!(xlm_token_client.balance(&depositor_3), 22606_6860860);
    pool_contract_client.withdraw(&depositor_3);

    assert_eq!(xlm_token_client.balance(&depositor_2), 0);
    pool_contract_client.withdraw_col(&depositor_2);
    assert_eq!(xlm_token_client.balance(&depositor_2), 45516_8243402);
    pool_contract_client.withdraw(&depositor_2);

    assert_eq!(xlm_token_client.balance(&depositor_4), 0);
    pool_contract_client.withdraw_col(&depositor_4);
    assert_eq!(xlm_token_client.balance(&depositor_4), 28414_3985187);
    pool_contract_client.withdraw(&depositor_4);

    env.ledger().set(LedgerInfo {
        timestamp: 172801 * 100,
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    pool_contract_client.withdraw(&depositor_6);

    assert_eq!(
        pool_contract_client.get_core_stats(),
        CoreStats {
            total_deposits: 0,
            lifetime_deposited: 37725_0000000,
            current_deposited: 0,
            lifetime_profit: 14113_5273037,
            lifetime_liquidated: 148193_2363482,
            liquidation_index: 2,
            rewards_factor: 0,
            total_shares: 0,
            share_price: 0_4471388,
        }
    );

    assert_eq!(usd_token_client.balance(&pool_contract_client.address), 0);
    assert_eq!(xlm_token_client.balance(&pool_contract_client.address), 0);
}
