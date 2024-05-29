#![cfg(test)]

extern crate std;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Env};

use crate::storage::vaults::OptionalVaultKey;
use crate::tests::test_utils_runtime_verification::{
    create_base_data_vaults, create_test_data_liquidity, setup_liquidity_pools, setup_vaults,
    TestDataLiquidity,
};

#[test]
fn usdx_low_solvent_liquidity() {
    let env: Env = Env::default();
    env.budget().reset_unlimited();

    // Create the test data
    let data_stable: TestDataLiquidity = create_test_data_liquidity(&env);

    // Change stable_token in vault data to USDx from stable liquidity pools data
    let data = create_base_data_vaults(
        &env,
        &data_stable.usdx_token_admin,
        &data_stable.usdx_token_client,
        &data_stable.usdx_token_admin_client,
    );

    // Vaults
    let actor_1: &Address = &Address::generate(&env);
    let actor_2: &Address = &Address::generate(&env);
    let actor_3: &Address = &Address::generate(&env);
    let actor_4: &Address = &Address::generate(&env);

    let vault_depositors = [actor_1, actor_2, actor_3, actor_4];
    setup_vaults(&env, &data, &vault_depositors);

    // Liquidity
    let actor_5: &Address = &Address::generate(&env);
    let actor_6: &Address = &Address::generate(&env);
    let actor_7: &Address = &Address::generate(&env);
    let actor_8: &Address = &Address::generate(&env);

    let stable_depositors = [
        actor_5.clone(),
        actor_6.clone(),
        actor_7.clone(),
        actor_8.clone(),
    ];
    setup_liquidity_pools(&env, &data_stable, &stable_depositors);

    // 1.1 USDx is selling at a lower rate in the market without the protocol being insolvent. There is liquidity in the stable pool.
    // Consider 1000 USDx can be bought with 900 USDc on an external market which actor 5 is going to arbitrage
    // 1. Actor 5 trades 900 USDc for 1000 USDx (simulated)
    let original_usdc = 9000_0000000;
    let original_usdx = 9000_0000000;
    assert_eq!(
        original_usdc,
        data_stable.usdc_token_client.balance(actor_5)
    );
    assert_eq!(
        original_usdx,
        data_stable.usdx_token_client.balance(actor_5)
    );
    data_stable.usdc_token_client.burn(actor_5, &900_0000000);
    data_stable
        .usdx_token_admin_client
        .mint(actor_5, &1000_0000000);
    assert_eq!(8100_0000000, data_stable.usdc_token_client.balance(actor_5));
    assert_eq!(
        10000_0000000,
        data_stable.usdx_token_client.balance(actor_5)
    );

    // 2. Actor 5 swaps 1000 USDx for 1000 - ceil(1000 * 30_000 / 10_000_000) USDc
    assert_eq!(
        4000_0000000,
        data_stable
            .usdc_token_client
            .balance(&data_stable.stable_liquidity_pool_contract_client.address)
    );
    assert_eq!(
        4000_0000000,
        data_stable
            .usdx_token_client
            .balance(&data_stable.stable_liquidity_pool_contract_client.address)
    );
    data_stable.stable_liquidity_pool_contract_client.swap(
        actor_5,
        &data_stable.usdx_token_client.address,
        &data_stable.usdc_token_client.address,
        &1000_0000000,
    );
    // fee = 3_0000000, protocol_share = 1_5000000
    assert_eq!(
        3003_0000000,
        data_stable
            .usdc_token_client
            .balance(&data_stable.stable_liquidity_pool_contract_client.address)
    );
    assert_eq!(
        4998_5000000,
        data_stable
            .usdx_token_client
            .balance(&data_stable.stable_liquidity_pool_contract_client.address)
    );
    assert_eq!(9097_0000000, data_stable.usdc_token_client.balance(actor_5));
    assert_eq!(
        original_usdx,
        data_stable.usdx_token_client.balance(actor_5)
    );

    // 3. Actor 5 profits 1000 - ceil(1000 * 30_000 / 10_000_000) - 900
    assert_eq!(
        97_0000000,
        data_stable.usdc_token_client.balance(actor_5) - original_usdc
    );
    assert_eq!(
        0,
        original_usdx - data_stable.usdx_token_client.balance(actor_5)
    );
}

#[test]
fn usdx_low_solvent_no_liquidity() {
    let env: Env = Env::default();
    env.budget().reset_unlimited(); // Removes upper bound on resource limit TODO: Better to reset manually?

    // Create the test data
    let data_stable: TestDataLiquidity = create_test_data_liquidity(&env);

    // Change stable_token in vault data to USDx from stable liquidity pools data
    let data = create_base_data_vaults(
        &env,
        &data_stable.usdx_token_admin,
        &data_stable.usdx_token_client,
        &data_stable.usdx_token_admin_client,
    );

    // Vaults
    let actor_1: &Address = &Address::generate(&env);
    let actor_2: &Address = &Address::generate(&env);
    let actor_3: &Address = &Address::generate(&env);
    let actor_4: &Address = &Address::generate(&env);

    let vault_depositors = [actor_1, actor_2, actor_3, actor_4];
    setup_vaults(&env, &data, &vault_depositors);

    // Liquidity
    let actor_5: &Address = &Address::generate(&env);
    let actor_6: &Address = &Address::generate(&env);
    let actor_7: &Address = &Address::generate(&env);
    let actor_8: &Address = &Address::generate(&env);

    let stable_depositors = [
        actor_5.clone(),
        actor_6.clone(),
        actor_7.clone(),
        actor_8.clone(),
    ];
    setup_liquidity_pools(&env, &data_stable, &stable_depositors);

    // 1.2 USDx is selling at a lower rate in the market without the protocol being insolvent. There is no liquidity in the stable pool.
    // Consider 1000 USDx can be bought with 900 USDc on an external market which actor 5 is going to arbitrage
    // 1. Actor 5 trades 900 USDc for 1000 USDx (simulated)
    let original_usdc = 9000_0000000;
    let original_usdx = 9000_0000000;
    assert_eq!(
        original_usdc,
        data_stable.usdc_token_client.balance(actor_5)
    );
    assert_eq!(
        original_usdx,
        data_stable.usdx_token_client.balance(actor_5)
    );
    data_stable.usdc_token_client.burn(actor_5, &900_0000000);
    data_stable
        .usdx_token_admin_client
        .mint(actor_5, &1000_0000000);
    assert_eq!(8100_0000000, data_stable.usdc_token_client.balance(actor_5));
    assert_eq!(
        10000_0000000,
        data_stable.usdx_token_client.balance(actor_5)
    );

    // 2. Actor 5 redeems
    data.contract_client.redeem(actor_5, &symbol_short!("usd"));
    assert_eq!(9860_0000000, data_stable.usdx_token_client.balance(actor_5));
    data.contract_client.redeem(actor_5, &symbol_short!("usd"));
    assert_eq!(9760_0000000, data_stable.usdx_token_client.balance(actor_5));
    data.contract_client.redeem(actor_5, &symbol_short!("usd"));
    assert_eq!(9660_0000000, data_stable.usdx_token_client.balance(actor_5));

    let cannot_redeem = data
        .contract_client
        .try_redeem(actor_5, &symbol_short!("usd"));

    assert!(cannot_redeem.is_err()) // BLOCKED BY LARGE VAULT
}

#[test]
fn usdx_high_solvent_liquidity() {
    let env: Env = Env::default();
    env.budget().reset_unlimited();

    // Create the test data
    let data_stable: TestDataLiquidity = create_test_data_liquidity(&env);

    // Change stable_token in vault data to USDx from stable liquidity pools data
    let data = create_base_data_vaults(
        &env,
        &data_stable.usdx_token_admin,
        &data_stable.usdx_token_client,
        &data_stable.usdx_token_admin_client,
    );

    // Vaults
    let actor_1: &Address = &Address::generate(&env);
    let actor_2: &Address = &Address::generate(&env);
    let actor_3: &Address = &Address::generate(&env);
    let actor_4: &Address = &Address::generate(&env);

    let vault_depositors = [actor_1, actor_2, actor_3, actor_4];
    setup_vaults(&env, &data, &vault_depositors);

    // Liquidity
    let actor_5: &Address = &Address::generate(&env);
    let actor_6: &Address = &Address::generate(&env);
    let actor_7: &Address = &Address::generate(&env);
    let actor_8: &Address = &Address::generate(&env);

    let stable_depositors = [
        actor_5.clone(),
        actor_6.clone(),
        actor_7.clone(),
        actor_8.clone(),
    ];
    setup_liquidity_pools(&env, &data_stable, &stable_depositors);

    // 2.1 USDx is selling at a higher rate in the market without the protocol being insolvent. There is liquidity in the stable pool.
    // Consider 1000 USDx can be sold for 1100 USDc on an external market which actor 5 is going to arbitrage
    // 1. Actor 5 swaps 1000 + fee USDc for 1000 USDx
    let original_usdc = 9000_0000000;
    let original_usdx = 9000_0000000;
    assert_eq!(
        original_usdc,
        data_stable.usdc_token_client.balance(actor_5)
    );
    assert_eq!(
        original_usdx,
        data_stable.usdx_token_client.balance(actor_5)
    );
    assert_eq!(
        4000_0000000,
        data_stable
            .usdc_token_client
            .balance(&data_stable.stable_liquidity_pool_contract_client.address)
    );
    assert_eq!(
        4000_0000000,
        data_stable
            .usdx_token_client
            .balance(&data_stable.stable_liquidity_pool_contract_client.address)
    );
    data_stable.stable_liquidity_pool_contract_client.swap(
        actor_5,
        &data_stable.usdc_token_client.address,
        &data_stable.usdx_token_client.address,
        &1003_0090271,
    );
    // fee = 3_0090271, protocol_share = 1_5045136
    assert_eq!(
        5001_5045135,
        data_stable
            .usdc_token_client
            .balance(&data_stable.stable_liquidity_pool_contract_client.address)
    );
    assert_eq!(
        3000_0000000,
        data_stable
            .usdx_token_client
            .balance(&data_stable.stable_liquidity_pool_contract_client.address)
    );
    assert_eq!(7996_9909729, data_stable.usdc_token_client.balance(actor_5));
    assert_eq!(
        10000_0000000,
        data_stable.usdx_token_client.balance(actor_5)
    );
    assert_eq!(
        1_5045136,
        data_stable.usdc_token_client.balance(&data_stable.treasury)
    );

    // 2. Actor 5 trades 1000 USDx for 1100 USDc (simulated)
    data_stable.usdx_token_client.burn(actor_5, &1000_0000000);
    data_stable
        .usdc_token_admin_client
        .mint(actor_5, &1100_0000000);
    assert_eq!(9096_9909729, data_stable.usdc_token_client.balance(actor_5));
    assert_eq!(
        original_usdx,
        data_stable.usdx_token_client.balance(actor_5)
    );

    // 3. Actor 5 profits 1100 - (1000 + fee)
    assert_eq!(
        96_9909729,
        data_stable.usdc_token_client.balance(actor_5) - original_usdc
    );
    assert_eq!(
        0,
        original_usdx - data_stable.usdx_token_client.balance(actor_5)
    );
}

#[test]
fn usdx_high_solvent_no_liquidity() {
    let env: Env = Env::default();

    // Create the test data
    let data_stable: TestDataLiquidity = create_test_data_liquidity(&env);

    // Change stable_token in vault data to USDx from stable liquidity pools data
    let data = create_base_data_vaults(
        &env,
        &data_stable.usdx_token_admin,
        &data_stable.usdx_token_client,
        &data_stable.usdx_token_admin_client,
    );

    // Vaults
    let actor_1: &Address = &Address::generate(&env);
    let actor_2: &Address = &Address::generate(&env);
    let actor_3: &Address = &Address::generate(&env);
    let actor_4: &Address = &Address::generate(&env);

    let vault_depositors = [actor_1, actor_2, actor_3, actor_4];
    setup_vaults(&env, &data, &vault_depositors);

    let actor_5: &Address = &Address::generate(&env);

    // 2.2
    // 2.1 USDx is selling at a higher rate in the market without the protocol being insolvent. There is no liquidity in the stable pool.
    // Consider 1000 USDx can be sold for 1100 USDc on an external market which actor 5 is going to arbitrage
    // 1. Actor 5 is going to create a debt position in vaults at opening_collateral_ratio, to get 1000 XLM
    let req_xlm: &u128 = &12_558_3094594; // ceil( (200 * opening_col_rate * debt) / (199 * rate) )
    data.collateral_token_admin_client
        .mint(actor_5, &(*req_xlm as i128));
    assert_eq!(0, data_stable.usdx_token_client.balance(actor_5));
    assert_eq!(0, data_stable.usdc_token_client.balance(actor_5));
    assert_eq!(
        12_558_3094594,
        data.collateral_token_client.balance(actor_5)
    );

    data.contract_client.new_vault(
        &OptionalVaultKey::None,
        actor_5,
        &1000_0000000,
        req_xlm,
        &symbol_short!("usd"),
    );

    assert_eq!(1000_0000000, data_stable.usdx_token_client.balance(actor_5));
    assert_eq!(0, data_stable.usdc_token_client.balance(actor_5));
    assert_eq!(0, data.collateral_token_client.balance(actor_5));

    // 2. Actor 5 trades 1000 USDx for 1100 USDc (simulated)
    data_stable.usdx_token_client.burn(actor_5, &1000_0000000);
    data_stable
        .usdc_token_admin_client
        .mint(actor_5, &1100_0000000);

    assert_eq!(0, data_stable.usdx_token_client.balance(actor_5));
    assert_eq!(1100_0000000, data_stable.usdc_token_client.balance(actor_5));
    assert_eq!(0, data.collateral_token_client.balance(actor_5));

    // 5. If USDc is traded for XLM at the same price the oracle has USDx at to see profit

    // TODO: Now actor 5 cannot get USDx through liquidity pool to recover debt position, is it possible this scenario repeated could cause depeg?
}
