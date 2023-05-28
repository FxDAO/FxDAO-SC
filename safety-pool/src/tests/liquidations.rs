#![cfg(test)]

use crate::contract::{SafetyPoolContract, SafetyPoolContractClient};
use crate::storage::deposits::Deposit;
use crate::tests::utils::{create_token_contract, set_allowance};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, BytesN, Env, Status, Symbol, Vec};

use crate::vaults;

#[test]
fn test_simple_liquidations_flow() {
    let env: Env = Env::default();

    // Set up the contracts

    // Shared variables
    let treasury_contract: Address = Address::random(&env);

    let currency_price: i128 = 0_0958840;
    let collateral_token_admin: Address = Address::random(&env);
    let collateral_token_client = create_token_contract(&env, &collateral_token_admin);

    let stable_token_admin: Address = Address::random(&env);
    let stable_token_client = create_token_contract(&env, &stable_token_admin);
    let stable_token_denomination = Symbol::short("usd");

    let depositor_1: Address = Address::random(&env);
    let depositor_2: Address = Address::random(&env);
    let depositor_3: Address = Address::random(&env);
    let depositor_4: Address = Address::random(&env);
    let depositor_5: Address = Address::random(&env);
    let depositor_6: Address = Address::random(&env);
    let depositors: [&Address; 6] = [
        &depositor_1,
        &depositor_2,
        &depositor_3,
        &depositor_4,
        &depositor_5,
        &depositor_6,
    ];
    let collateral_initial_balance: i128 = 3000_0000000;

    // Register and start vaults' contract
    let vaults_contract_id: BytesN<32> = env.register_contract_wasm(None, vaults::WASM);
    let vaults_contract_address: Address = Address::from_contract_id(&env, &vaults_contract_id);
    let vaults_contract_client = vaults::Client::new(&env, &vaults_contract_id);
    let vaults_contract_admin: Address = Address::random(&env);
    let min_collateral_rate: i128 = 1_1000000;
    let opening_debt_amount: i128 = 1_0000000;
    let opening_collateral_rate: i128 = 1_1500000;

    vaults_contract_client.init(
        &vaults_contract_admin,
        &collateral_token_client.contract_id,
        &stable_token_admin,
    );

    vaults_contract_client
        .create_currency(&stable_token_denomination, &stable_token_client.contract_id);

    vaults_contract_client.set_currency_rate(&stable_token_denomination, &currency_price);

    vaults_contract_client.toggle_currency(&stable_token_denomination, &true);

    vaults_contract_client.set_vault_conditions(
        &min_collateral_rate,
        &opening_debt_amount,
        &opening_collateral_rate,
        &stable_token_denomination,
    );

    stable_token_client.incr_allow(
        &stable_token_admin,
        &vaults_contract_address,
        &90000000000000000000,
    );

    stable_token_client.mint(
        &stable_token_admin,
        &stable_token_admin,
        &90000000000000000000,
    );

    // Register and start safety pool's contract
    let pool_contract_id: BytesN<32> = env.register_contract(None, SafetyPoolContract);
    // let pool_contract_address: Address = Address::from_contract_id(&env, &pool_contract_id);
    let pool_contract_client = SafetyPoolContractClient::new(&env, &pool_contract_id);
    let pool_contract_admin: Address = Address::random(&env);
    let min_pool_deposit: u128 = 100_0000000;
    let profit_share: Vec<u32> = vec![&env, 1u32, 2u32] as Vec<u32>;
    let liquidator_share: Vec<u32> = vec![&env, 1u32, 2u32] as Vec<u32>;

    pool_contract_client.init(
        &pool_contract_admin,
        &vaults_contract_address,
        &treasury_contract,
        &collateral_token_client.contract_id,
        &stable_token_client.contract_id,
        &stable_token_denomination,
        &min_pool_deposit,
        &profit_share,
        &liquidator_share,
    );

    // We create the initial vaults, a total of 6 vaults will be created where two of them
    // will be liquidated later, a total of 18k collateral (3k each) will be issued. The first 4
    // depositors will deposit all of the stablecoin balance into the pool (400 usd)
    let assets: Vec<BytesN<32>> =
        vec![&env, collateral_token_client.contract_id.clone()] as Vec<BytesN<32>>;
    for (i, depositor) in depositors.iter().enumerate() {
        collateral_token_client.mint(
            &collateral_token_admin,
            &depositor,
            &collateral_initial_balance,
        );

        set_allowance(
            &env,
            &assets,
            &vaults_contract_client.contract_id,
            &depositor,
        );

        let initial_debt: i128;
        if i + 1 < 5 {
            initial_debt = 100_0000000;
        } else {
            initial_debt = 160_0000000;
        }
        vaults_contract_client.new_vault(
            &depositor,
            &initial_debt,
            &collateral_initial_balance,
            &stable_token_denomination,
        );

        // If is depositor between 1 and 4, deposit the stable balance into the pool
        if i + 1 < 5 {
            let stablecoin_balance: i128 = stable_token_client.balance(&depositor);
            pool_contract_client.deposit(&depositor, &(stablecoin_balance as u128));
        }
    }

    env.budget().reset_unlimited(); // We reset the budget

    let liquidator: Address = Address::random(&env);

    // We test that it should fail because there is no vault to liquidate yet
    let no_vaults_error_result = pool_contract_client.try_liquidate(&liquidator).unwrap_err();

    assert_eq!(
        no_vaults_error_result,
        Ok(Status::from_contract_error(30000))
    );

    // We update the price in order to liquidate the two vaults
    let new_currency_price = 0_0586660;
    vaults_contract_client.set_currency_rate(&stable_token_denomination, &new_currency_price);

    env.budget().reset_unlimited(); // We reset the budget
    pool_contract_client.liquidate(&liquidator);

    // Now we confirm the distribution was correct, the calculations go this way:
    // 1.- 2 Vaults were liquidated so the vaults contract should only have a balance of 3_000_0000000 * 4 = 12_000_0000000 of collateral
    // 2.- With a rate of 0_0586660, the value in collateral is 5454_6074387 so a total of 545_3925613 profit will be shared between the depositors and the contract
    // 3.- Depositors will receive 5454_6074387 + (545_3925613 / 2) = 5727_3037194 (1431_8259298 each)
    // 4.- Treasury and the liquidator will receive 272_6962808 / 2 each
    assert_eq!(
        collateral_token_client.balance(&vaults_contract_address),
        12_000_0000000
    );

    for (i, depositor) in depositors.iter().enumerate() {
        if i + 1 < 5 {
            assert_eq!(collateral_token_client.balance(&depositor), 1431_8259298);
        } else {
            assert_eq!(collateral_token_client.balance(&depositor), 0);
        }
    }

    // We now check that each deposit into the pool gets updated and reflect the current pool balance
    let mut total_in_vaults: u128 = 0;
    for depositor in depositors.iter() {
        let deposit: Deposit = pool_contract_client.get_deposit(&depositor);
        total_in_vaults = total_in_vaults + deposit.amount;
    }

    assert_eq!(
        total_in_vaults as i128,
        stable_token_client.balance(&(Address::from_contract_id(&env, &pool_contract_id)))
    );

    assert_eq!(
        collateral_token_client.balance(&treasury_contract),
        272_6962808 / 2
    );

    assert_eq!(
        collateral_token_client.balance(&liquidator),
        272_6962808 / 2
    );
}
