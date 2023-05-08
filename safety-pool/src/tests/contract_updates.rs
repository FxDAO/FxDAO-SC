#![cfg(test)]
extern crate std;

use crate::storage::core::CoreState;
use crate::tests::utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol, Vec};

#[test]
fn update_contract_core_state() {
    let env: Env = Env::default();
    let test_data: TestData = create_test_data(&env);
    init_contract(&test_data);

    let mut target_core_state: CoreState = test_data.contract_client.get_core_state();
    assert_eq!(
        target_core_state.clone(),
        CoreState {
            admin: target_core_state.clone().admin,
            treasury_contract: target_core_state.clone().treasury_contract,
            vaults_contract: target_core_state.clone().vaults_contract,
            collateral_asset: target_core_state.clone().collateral_asset,
            deposit_asset: target_core_state.clone().deposit_asset,
            denomination_asset: target_core_state.clone().denomination_asset,
            min_deposit: target_core_state.clone().min_deposit,
            treasury_share: target_core_state.clone().treasury_share,
            liquidator_share: target_core_state.clone().liquidator_share
        }
    );

    // Update admin
    let new_admin: Address = Address::random(&env);
    test_data.contract_client.update_contract_admin(&new_admin);
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            target_core_state.admin.clone(),
            test_data.contract_client.contract_id.clone(),
            Symbol::new(&env, "update_contract_admin"),
            (new_admin.clone(),).into_val(&env),
        )]
    );

    target_core_state.admin = new_admin;
    assert_eq!(
        test_data.contract_client.get_core_state(),
        target_core_state
    );

    // Update vaults contract
    let new_vaults_contract: Address = Address::random(&env);
    test_data
        .contract_client
        .update_vaults_contract(&new_vaults_contract);
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            target_core_state.admin.clone(),
            test_data.contract_client.contract_id.clone(),
            Symbol::new(&env, "update_vaults_contract"),
            (new_vaults_contract.clone(),).into_val(&env),
        )]
    );

    target_core_state.vaults_contract = new_vaults_contract;
    assert_eq!(
        test_data.contract_client.get_core_state(),
        target_core_state
    );

    // Update treasury contract
    let new_treasury_contract: Address = Address::random(&env);
    test_data
        .contract_client
        .update_treasury_contract(&new_treasury_contract);
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            target_core_state.admin.clone(),
            test_data.contract_client.contract_id.clone(),
            Symbol::new(&env, "update_treasury_contract"),
            (new_treasury_contract.clone(),).into_val(&env),
        )]
    );

    target_core_state.treasury_contract = new_treasury_contract;
    assert_eq!(
        test_data.contract_client.get_core_state(),
        target_core_state
    );

    // Update min deposit
    let new_min_deposit: u128 = 50_0000000;
    test_data
        .contract_client
        .update_min_deposit(&new_min_deposit);
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            target_core_state.admin.clone(),
            test_data.contract_client.contract_id.clone(),
            Symbol::new(&env, "update_min_deposit"),
            (new_min_deposit.clone(),).into_val(&env),
        )]
    );

    target_core_state.min_deposit = new_min_deposit;
    assert_eq!(
        test_data.contract_client.get_core_state(),
        target_core_state
    );

    // Update treasury share
    let new_treasury_share: Vec<u32> = vec![&env, 2u32, 3u32] as Vec<u32>;
    test_data
        .contract_client
        .update_treasury_share(&new_treasury_share);
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            target_core_state.admin.clone(),
            test_data.contract_client.contract_id.clone(),
            Symbol::new(&env, "update_treasury_share"),
            (new_treasury_share.clone(),).into_val(&env),
        )]
    );

    target_core_state.treasury_share = new_treasury_share;
    assert_eq!(
        test_data.contract_client.get_core_state(),
        target_core_state
    );

    // Update liquidator share
    let new_liquidator_share: Vec<u32> = vec![&env, 2u32, 3u32] as Vec<u32>;
    test_data
        .contract_client
        .update_liquidator_share(&new_liquidator_share);
    assert_eq!(
        env.recorded_top_authorizations(),
        std::vec![(
            target_core_state.admin.clone(),
            test_data.contract_client.contract_id.clone(),
            Symbol::new(&env, "update_liquidator_share"),
            (new_liquidator_share.clone(),).into_val(&env),
        )]
    );

    target_core_state.liquidator_share = new_liquidator_share;
    assert_eq!(
        test_data.contract_client.get_core_state(),
        target_core_state
    );
}
