#![cfg(test)]

mod vaults_properties {
    extern crate std;

    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{token, Address, Env};

    use crate::storage::vaults::*;

    use crate::utils::payments::calc_fee;

    use crate::tests::test_utils::{
        create_base_data, create_base_variables, set_initial_state, update_oracle_price,
    };
    use crate::tests::test_utils::{InitialVariables, TestData};

    use proptest::prelude::*;

    // Constants for the calling
    const FEE: u128 = 50000;
    const OPENING_COL_RATE: u128 = 1_1500000;

    const DEPOSITOR_COLLATERAL: u128 = 3000_0000000;
    const DEPOSITOR_DEBT: u128 = 100_0000000;
    const DEPOSITOR_INDEX: u128 = 2985_0000000;

    const DEPOSITOR_COLLATERAL_MINUS_FEE: u128 =
        DEPOSITOR_COLLATERAL - u128::div_ceil(DEPOSITOR_COLLATERAL * FEE, 1_0000000);
    const MAX_RATE: u128 = u128::MAX / DEPOSITOR_COLLATERAL_MINUS_FEE;
    const MIN_RATE: u128 = 1 + (OPENING_COL_RATE * DEPOSITOR_DEBT / DEPOSITOR_COLLATERAL_MINUS_FEE);

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        /// In order to deposit, `rate` must be in range \
        /// **let** *fee* = &lceil; *collateral* &times; *fee* / 10,000,000 &rceil; **in** \
        /// **let** *vault_col* = *collateral* - *fee* **in** \
        /// **invariant** 1 + (*opening_collateral_rate* &times; *debt* / *vault_col*) &leq; `rate` &leq; 2^128 - 1 / *vault_col*
        fn rates(rate_low in 0_u128..MIN_RATE, rate_good in MIN_RATE..=MAX_RATE, rate_high in MAX_RATE+1..u128::MAX) {
            // Set conditions
            let env = Env::default();
            env.mock_all_auths();
            let data: TestData = create_base_data(&env);
            let base_variables: InitialVariables = create_base_variables(&env, &data);
            set_initial_state(&env, &data, &base_variables);

            assert_eq!(FEE, data.fee);
            assert_eq!(DEPOSITOR_COLLATERAL_MINUS_FEE, DEPOSITOR_COLLATERAL - calc_fee(&FEE, &DEPOSITOR_COLLATERAL));
            assert_eq!(base_variables.opening_col_rate, OPENING_COL_RATE);

            data.contract_client.set_vault_conditions(
                &base_variables.min_col_rate,
                &1000000000,
                &base_variables.opening_col_rate,
                &data.stable_token_denomination,
            );

            // Depositors funds
            let depositor: Address = Address::generate(&env);

            token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
                .mint(&depositor, &(DEPOSITOR_COLLATERAL as i128));

            // Test low
            update_oracle_price(
                &env,
                &data.oracle_contract_client,
                &data.stable_token_denomination,
                &(rate_low as i128),
            );
            let result = data.contract_client.try_new_vault(
                &OptionalVaultKey::None,
                &depositor,
                &DEPOSITOR_DEBT,
                &DEPOSITOR_COLLATERAL,
                &data.stable_token_denomination,
            );
            assert!(result.is_err()); // Expecting an error

            // Test high
            update_oracle_price(
                &env,
                &data.oracle_contract_client,
                &data.stable_token_denomination,
                &(rate_high as i128),
            );
            let result = data.contract_client.try_new_vault(
                &OptionalVaultKey::None,
                &depositor,
                &DEPOSITOR_DEBT,
                &DEPOSITOR_COLLATERAL,
                &data.stable_token_denomination,
            );
            assert!(result.is_err()); // Expecting an error

            // Test in range
            update_oracle_price(
                &env,
                &data.oracle_contract_client,
                &data.stable_token_denomination,
                &(rate_good as i128),
            );
            data.contract_client.new_vault(
                &OptionalVaultKey::None,
                &depositor,
                &DEPOSITOR_DEBT,
                &DEPOSITOR_COLLATERAL,
                &data.stable_token_denomination,
            );

            let depositor_vault: Vault = data
                .contract_client
                .get_vault(&depositor, &data.stable_token_denomination);

            assert_eq!(depositor_vault.index, DEPOSITOR_INDEX);
        }
    }

    const RATE: u128 = 1_000_000;
    const MAX_COLLATERAL: u128 = (i128::MAX as u128) / 1_000_000_000;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn withdraw_correct(depositor_collateral in 1..=MAX_COLLATERAL) {
            // Set conditions
            let env = Env::default();
            env.mock_all_auths();
            let data: TestData = create_base_data(&env);
            let base_variables: InitialVariables = create_base_variables(&env, &data);
            set_initial_state(&env, &data, &base_variables);

            let depositor_collateral_minus_fees = depositor_collateral - calc_fee(&data.fee, &depositor_collateral);
            let depositor_debt = depositor_collateral / 1_000_000_000; // Little unsure abou t this

            update_oracle_price(
                &env,
                &data.oracle_contract_client,
                &data.stable_token_denomination,
                &(RATE as i128),
            );

            data.contract_client.set_vault_conditions(
                &base_variables.min_col_rate,
                &1000000000,
                &base_variables.opening_col_rate,
                &data.stable_token_denomination,
            );

            // Depositors funds
            let depositor: Address = Address::generate(&env);
            token::StellarAssetClient::new(&env, &data.collateral_token_client.address)
                .mint(&depositor, &(depositor_collateral as i128));
            assert_eq!(data.collateral_token_client.balance(&depositor) as u128, depositor_collateral);
            assert_eq!(data.stable_token_client.balance(&depositor) as u128, 0);

            // Depositing those funds
            data.contract_client.new_vault(
                &OptionalVaultKey::None,
                &depositor,
                &(depositor_debt as u128),
                &(depositor_collateral as u128),
                &data.stable_token_denomination,
            );


            // Validate state and balances
            assert_eq!(data.collateral_token_client.balance(&depositor) as u128, 0);
            assert_eq!(data.stable_token_client.balance(&depositor) as u128, depositor_debt);

            let depositor_vault: Vault = data.contract_client.get_vault(&depositor, &data.stable_token_denomination);
            assert_eq!(depositor_vault.total_collateral, depositor_collateral_minus_fees);

            let vaults_info: VaultsInfo = data.contract_client.get_vaults_info(&data.stable_token_denomination);
            assert_eq!(vaults_info.total_col, depositor_collateral_minus_fees);

            let depositor_index = 1_000_000_000 * depositor_collateral_minus_fees / depositor_debt;
            assert_eq!(depositor_vault.index, depositor_index);

            let vault_key = VaultKey { index: depositor_index, account:depositor.clone(), denomination: data.stable_token_denomination.clone() };

            // Withdrawing the funds
            data.contract_client.pay_debt(
                &OptionalVaultKey::None,
                &vault_key,
                &OptionalVaultKey::None,
                &(depositor_debt as u128),
            );

            // Validate state and balances
            let depositor_collateral_after_withdraw = depositor_collateral_minus_fees - calc_fee(&data.fee, &depositor_collateral_minus_fees);

            assert_eq!(data.collateral_token_client.balance(&depositor) as u128, depositor_collateral_after_withdraw);
            assert_eq!(data.stable_token_client.balance(&depositor) as u128, 0);

            let no_vault = data.contract_client.try_get_vault(&depositor, &data.stable_token_denomination);
            assert!(no_vault.is_err());

            let vaults_info: VaultsInfo = data.contract_client.get_vaults_info(&data.stable_token_denomination);
            assert_eq!(vaults_info.total_col, 0);
        }
    }
}

mod duplicate_currency {
    /* Same Address, different Symbol shouldn't pass but will */

    extern crate std;

    use crate::errors::SCErrors;
    use crate::tests::test_utils::create_base_data;

    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, Address, Env};

    use super::super::test_utils::TestData;

    #[test]
    fn test_duplicate_currency() {
        let env = Env::default();
        env.mock_all_auths();
        let data: TestData = create_base_data(&env);

        data.contract_client.init(
            &data.contract_admin,
            &data.protocol_manager,
            &data.collateral_token_client.address,
            &data.stable_token_issuer,
            &data.treasury,
            &data.fee,
            &data.oracle,
        );

        let denomination = symbol_short!("usd");
        let contract = data.stable_token_client.address;

        // Create the initial currency with "usd"
        data.contract_client
            .create_currency(&denomination, &contract);

        let currency = data.contract_client.get_currency(&denomination);

        assert_eq!(currency.denomination, symbol_short!("usd"));
        assert_eq!(currency.contract, contract);

        // Attempt to create new currency with different address, but "usd", expect fail
        let random_addr = Address::generate(&env);

        let duplicate_denomination = data
            .contract_client
            .try_create_currency(&denomination, &random_addr)
            .unwrap_err()
            .unwrap();

        assert_eq!(
            duplicate_denomination,
            SCErrors::CurrencyAlreadyAdded.into()
        );

        // Attempt to create new currency using same address, but "USD", expect pass even though it is duplicate
        let denomination_capitalised = symbol_short!("USD");
        let fake_currency = match data
            .contract_client
            .try_get_currency(&denomination_capitalised)
        {
            Ok(_) => unreachable!(),
            Err(e) => e.unwrap(),
        };

        assert_eq!(fake_currency, SCErrors::CurrencyDoesntExist.into()); // Shows that "USD" does not exist currently

        data.contract_client.create_currency(
            &denomination_capitalised, // "USD"
            &contract,                 // Same address as "usd"
        );

        let duplicate_currency = data.contract_client.get_currency(&denomination_capitalised); // Tx accepted

        assert_eq!(duplicate_currency.denomination, symbol_short!("USD"));
        assert_eq!(duplicate_currency.contract, contract);
    }
}

mod weak_fairness {
    /* These tests are to demonstrate that multiple orderings are currently possible for vaults when vaults have duplicate
    vault indices. These 3 tests show 3 different positions of a duplicate vault, where the duplicate vault is able to be
    positioned before, inbetween, and after duplicates. */

    extern crate std;

    use crate::storage::vaults::{OptionalVaultKey, Vault, VaultKey};
    use crate::tests::test_utils::{
        create_base_data, create_base_variables, set_initial_state, update_oracle_price,
    };

    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    use super::super::test_utils::TestData;

    #[test]
    fn test_weak_fairness_position_1() {
        let env = Env::default();
        let data = create_base_data(&env);

        let depositor_1: &Address = &Address::generate(&env); // Index: 3233_7500000
        let depositor_2: &Address = &Address::generate(&env); // Index: 3233_7500000
        let depositor_3: &Address = &Address::generate(&env); // Index: 1747_6464285
        let depositor_4: &Address = &Address::generate(&env); // Index: 5970_0000000

        let depositors = [depositor_1, depositor_2, depositor_3, depositor_4];
        create_base_state(&env, &data, &depositors);

        // Fifth depositor (duplicated) // Index: 3233_7500000
        let depositor_5 = Address::generate(&env);

        let expected_depositors = [
            depositor_3,
            &depositor_5,
            depositor_2,
            depositor_1,
            depositor_4,
        ];
        //                                                     ^^^^^^^^^^^^

        insert_and_check_order(&depositor_5, depositor_3, &expected_depositors, &data)
    }

    #[test]
    fn test_weak_fairness_position_2() {
        let env = Env::default();
        let data = create_base_data(&env);

        let depositor_1: &Address = &Address::generate(&env); // Index: 3233_7500000
        let depositor_2: &Address = &Address::generate(&env); // Index: 3233_7500000
        let depositor_3: &Address = &Address::generate(&env); // Index: 1747_6464285
        let depositor_4: &Address = &Address::generate(&env); // Index: 5970_0000000

        let depositors = [depositor_1, depositor_2, depositor_3, depositor_4];
        create_base_state(&env, &data, &depositors);

        // Fifth depositor (duplicated) // Index: 3233_7500000
        let depositor_5 = Address::generate(&env);

        let expected_depositors = [
            depositor_3,
            depositor_2,
            &depositor_5,
            depositor_1,
            depositor_4,
        ];
        //                                                                  ^^^^^^^^^^^^

        insert_and_check_order(&depositor_5, depositor_2, &expected_depositors, &data)
    }

    #[test]
    fn test_weak_fairness_position_3() {
        let env = Env::default();
        let data = create_base_data(&env);

        let depositor_1: &Address = &Address::generate(&env); // Index: 3233_7500000
        let depositor_2: &Address = &Address::generate(&env); // Index: 3233_7500000
        let depositor_3: &Address = &Address::generate(&env); // Index: 1747_6464285
        let depositor_4: &Address = &Address::generate(&env); // Index: 5970_0000000

        let depositors = [depositor_1, depositor_2, depositor_3, depositor_4];
        create_base_state(&env, &data, &depositors);

        // Fifth depositor (duplicated) // Index: 3233_7500000
        let depositor_5 = Address::generate(&env);

        let expected_depositors = [
            depositor_3,
            depositor_2,
            depositor_1,
            &depositor_5,
            depositor_4,
        ];
        //                                                                               ^^^^^^^^^^^^

        insert_and_check_order(&depositor_5, depositor_1, &expected_depositors, &data)
    }

    fn create_base_state(env: &Env, data: &TestData, depositors: &[&Address]) {
        env.mock_all_auths();
        let base_variables = create_base_variables(&env, &data);
        set_initial_state(&env, &data, &base_variables);

        let currency_price: u128 = 920330;
        let min_col_rate: u128 = 11000000;
        let min_debt_creation: u128 = 1000000000;
        let opening_col_rate: u128 = 11500000;

        data.contract_client.set_vault_conditions(
            &min_col_rate,
            &min_debt_creation,
            &opening_col_rate,
            &data.stable_token_denomination,
        );

        update_oracle_price(
            &env,
            &data.oracle_contract_client,
            &data.stable_token_denomination,
            &(currency_price as i128),
        );

        // 1st Set of tests
        // This section includes and checks that every time we create a new vault the values are updated

        // First deposit
        // This deposit should have an index of: 3233_7500000
        let depositor_1 = depositors[0];
        let depositor_1_debt: u128 = 100_0000000;
        let depositor_1_collateral_amount: u128 = 3250_0000000;

        data.collateral_token_admin_client
            .mint(&depositor_1, &(depositor_1_collateral_amount as i128 * 2));

        data.contract_client.new_vault(
            &OptionalVaultKey::None,
            &depositor_1,
            &depositor_1_debt,
            &depositor_1_collateral_amount,
            &data.stable_token_denomination,
        );

        let depositor_1_vault: Vault = data
            .contract_client
            .get_vault(&depositor_1, &data.stable_token_denomination);

        assert_eq!(depositor_1_vault.index, 3233_7500000);

        // Second depositor
        // This deposit should have an index of: 3233_7500000
        let depositor_2 = depositors[1];
        let depositor_2_debt: u128 = 100_0000000;
        let depositor_2_collateral_amount: u128 = 3250_0000000;

        data.collateral_token_admin_client
            .mint(&depositor_2, &(depositor_2_collateral_amount as i128 * 2));

        data.contract_client.new_vault(
            &OptionalVaultKey::Some(VaultKey {
                index: depositor_1_vault.index.clone(),
                account: depositor_1_vault.account.clone(),
                denomination: data.stable_token_denomination.clone(),
            }),
            &depositor_2,
            &depositor_2_debt,
            &depositor_2_collateral_amount,
            &data.stable_token_denomination,
        );

        let depositor_2_vault: Vault = data
            .contract_client
            .get_vault(&depositor_2, &data.stable_token_denomination);

        assert_eq!(depositor_2_vault.index, 3233_7500000);

        // Third depositor
        // This deposit should have an index of: 1747_6464285
        let depositor_3 = depositors[2];
        let depositor_3_debt: u128 = 140_0000000;
        let depositor_3_collateral_amount: u128 = 2459_0000000;

        data.collateral_token_admin_client
            .mint(&depositor_3, &(depositor_3_collateral_amount as i128 * 2));

        data.contract_client.new_vault(
            &OptionalVaultKey::None,
            &depositor_3,
            &depositor_3_debt,
            &depositor_3_collateral_amount,
            &data.stable_token_denomination,
        );

        let depositor_3_vault: Vault = data
            .contract_client
            .get_vault(&depositor_3, &data.stable_token_denomination);

        assert_eq!(depositor_3_vault.index, 1747_6464285);

        // Fourth depositor
        // This deposit should have an index of: 5970_0000000
        let depositor_4 = depositors[3];
        let depositor_4_debt: u128 = 150_0000000;
        let depositor_4_collateral_amount: u128 = 9000_0000000;

        data.collateral_token_admin_client
            .mint(&depositor_4, &(depositor_4_collateral_amount as i128 * 2));

        data.contract_client.new_vault(
            &OptionalVaultKey::Some(VaultKey {
                index: depositor_1_vault.index.clone(),
                account: depositor_1_vault.account.clone(),
                denomination: data.stable_token_denomination.clone(),
            }),
            &depositor_4,
            &depositor_4_debt,
            &depositor_4_collateral_amount,
            &data.stable_token_denomination,
        );

        let depositor_4_vault: Vault = data
            .contract_client
            .get_vault(&depositor_4, &data.stable_token_denomination);

        assert_eq!(depositor_4_vault.index, 5970_0000000);

        // Assert the order is correct
        let ordered_depositors = [depositor_3, depositor_2, depositor_1, depositor_4];

        let vault_info = data
            .contract_client
            .get_vaults_info(&data.stable_token_denomination);

        let mut key = match vault_info.lowest_key {
            OptionalVaultKey::None => unreachable!(),
            OptionalVaultKey::Some(key) => key,
        };

        let mut vault = data
            .contract_client
            .get_vault(&key.account, &key.denomination);

        for i in 0..ordered_depositors.len() {
            assert_eq!(vault.account, *ordered_depositors[i]);

            if i != ordered_depositors.len() - 1 {
                key = match vault.next_key {
                    OptionalVaultKey::None => unreachable!(),
                    OptionalVaultKey::Some(key) => key,
                };

                vault = data.contract_client.get_vault_from_key(&key)
            }
        }
    }

    fn insert_and_check_order(
        new_depositor: &Address,
        prev_depositor: &Address,
        expected_depositors: &[&Address],
        data: &TestData,
    ) {
        let new_depositor_debt: u128 = 100_0000000;
        let new_depositor_collateral_amount: u128 = 3250_0000000;

        data.collateral_token_admin_client.mint(
            &new_depositor,
            &(new_depositor_collateral_amount as i128 * 2),
        );

        let depositor_1_vault: Vault = data
            .contract_client
            .get_vault(&prev_depositor, &data.stable_token_denomination);

        data.contract_client.new_vault(
            &OptionalVaultKey::Some(VaultKey {
                index: depositor_1_vault.index.clone(),
                account: depositor_1_vault.account.clone(),
                denomination: data.stable_token_denomination.clone(),
            }),
            &new_depositor,
            &new_depositor_debt,
            &new_depositor_collateral_amount,
            &data.stable_token_denomination,
        );

        let depositor_5_vault: Vault = data
            .contract_client
            .get_vault(&new_depositor, &data.stable_token_denomination);

        assert_eq!(depositor_5_vault.index, 3233_7500000);

        let vault_info = data
            .contract_client
            .get_vaults_info(&data.stable_token_denomination);

        let mut key = match vault_info.lowest_key {
            OptionalVaultKey::None => unreachable!(),
            OptionalVaultKey::Some(key) => key,
        };

        let mut vault = data
            .contract_client
            .get_vault(&key.account, &key.denomination);
        for i in 0..expected_depositors.len() {
            assert_eq!(vault.account, *expected_depositors[i]);

            if i != expected_depositors.len() - 1 {
                key = match vault.next_key {
                    OptionalVaultKey::None => unreachable!(),
                    OptionalVaultKey::Some(key) => key,
                };

                vault = data.contract_client.get_vault_from_key(&key)
            }
        }
    }
}
