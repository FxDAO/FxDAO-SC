#![cfg(test)]

use crate::storage::core::{CoreDataKeys, CoreStorageFunc};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal};

#[test]
pub fn test_init_contract() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    e.as_contract(&test_data.contract_client.address, || {
        let admin: Address = e._core().address(&CoreDataKeys::Admin).unwrap();
        let manager: Address = e._core().address(&CoreDataKeys::Manager).unwrap();
        let rewards_asset: Address = e._core().address(&CoreDataKeys::RewardsAsset).unwrap();
        assert_eq!(&rewards_asset, &test_data.rewards_asset_client.address);
        assert_eq!(&admin, &test_data.admin);
        assert_eq!(&manager, &test_data.manager);
    });
}

#[test]
pub fn test_core_updates_validations() {
    let e: Env = Env::default();
    let test_data: TestData = create_test_data(&e);
    init_contract(&test_data);

    let new_admin: Address = Address::generate(&e);
    let new_manager: Address = Address::generate(&e);

    let no_admin_signature_error = test_data.contract_client.try_set_admin(&new_admin);
    assert!(no_admin_signature_error.is_err());

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.admin,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "set_admin",
                args: (new_admin.clone(),).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .set_admin(&new_admin);

    let no_manager_signature_error = test_data.contract_client.try_set_manager(&new_manager);
    assert!(no_manager_signature_error.is_err());

    test_data
        .contract_client
        .mock_auths(&[MockAuth {
            address: &test_data.manager,
            invoke: &MockAuthInvoke {
                contract: &test_data.contract_client.address,
                fn_name: "set_manager",
                args: (new_manager.clone(),).into_val(&e),
                sub_invokes: &[],
            },
        }])
        .set_manager(&new_manager);
}
