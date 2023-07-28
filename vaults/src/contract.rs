use crate::storage_types::*;
use crate::utils::core::*;
use crate::utils::indexes::*;
use crate::utils::legacy_file::*;
use crate::utils::vaults::*;

use num_integer::div_floor;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token, vec, Address, BytesN, Env,
    Symbol, Vec,
};

pub const CONTRACT_DESCRIPTION: Symbol = symbol_short!("Vaults");
pub const CONTRACT_VERSION: Symbol = symbol_short!("0_3_0");

// TODO: Explain each function here
pub trait VaultsContractTrait {
    /// Set up and management
    fn init(
        env: Env,
        admin: Address,
        oracle_admin: Address,
        protocol_manager: Address,
        col_token: Address,
        stable_issuer: Address,
    );

    fn set_admin(env: Env, address: Address);
    fn set_protocol_manager(env: Env, address: Address);
    fn get_admin(env: Env) -> Address;
    fn get_core_state(env: Env) -> CoreState;
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
    fn version(env: Env) -> (Symbol, Symbol);

    /// Currency vaults conditions
    fn set_vault_conditions(
        env: Env,
        min_col_rate: i128,
        min_debt_creation: i128,
        opening_col_rate: i128,
        denomination: Symbol,
    );
    fn get_vault_conditions(env: Env, denomination: Symbol) -> CurrencyVaultsConditions;

    /// Currencies methods
    fn create_currency(env: Env, denomination: Symbol, contract: Address);
    fn get_currency(env: Env, denomination: Symbol) -> Currency;
    fn get_currency_stats(env: Env, denomination: Symbol) -> CurrencyStats;
    fn set_currency_rate(env: Env, denomination: Symbol, rate: i128);
    fn toggle_currency(env: Env, denomination: Symbol, active: bool);

    /// Vaults methods
    fn new_vault(
        env: Env,
        caller: Address,
        initial_debt: i128,
        collateral_amount: i128,
        denomination: Symbol,
    );
    fn get_vault(env: Env, caller: Address, denomination: Symbol) -> UserVault;
    fn incr_col(env: Env, caller: Address, amount: i128, denomination: Symbol);
    fn incr_debt(env: Env, caller: Address, debt_amount: i128, denomination: Symbol);
    fn pay_debt(env: Env, caller: Address, amount: i128, denomination: Symbol);
    fn get_indexes(env: Env, denomination: Symbol) -> Vec<i128>;
    fn get_vaults_with_index(env: Env, denomination: Symbol, index: i128) -> Vec<UserVault>;

    /// Redeeming
    fn redeem(env: Env, caller: Address, amount: i128, denomination: Symbol);

    /// Liquidation
    fn liquidate(env: Env, caller: Address, denomination: Symbol, owners: Vec<Address>);
    fn vaults_to_liquidate(env: Env, denomination: Symbol) -> Vec<UserVault>;
}

#[contract]
pub struct VaultsContract;

// TODO: Add events for each function
#[contractimpl]
impl VaultsContractTrait for VaultsContract {
    fn init(
        env: Env,
        admin: Address,
        oracle_admin: Address,
        protocol_manager: Address,
        col_token: Address,
        stable_issuer: Address,
    ) {
        bump_instance(&env);
        if env.storage().instance().has(&DataKeys::CoreState) {
            panic_with_error!(&env, SCErrors::AlreadyInit);
        }

        env.storage().instance().set(
            &DataKeys::CoreState,
            &CoreState {
                col_token,
                stable_issuer,
            },
        );
        env.storage().instance().set(&DataKeys::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKeys::OracleAdmin, &oracle_admin);
        env.storage()
            .instance()
            .set(&DataKeys::ProtocolManager, &protocol_manager);
    }

    // TODO: Test this
    fn set_admin(env: Env, address: Address) {
        bump_instance(&env);
        check_admin(&env);
        env.storage().instance().set(&DataKeys::Admin, &address);
    }

    // TODO: Test this
    fn set_protocol_manager(env: Env, address: Address) {
        bump_instance(&env);
        check_protocol_manager(&env);
        env.storage()
            .instance()
            .set(&DataKeys::ProtocolManager, &address);
    }

    fn get_admin(env: Env) -> Address {
        bump_instance(&env);
        env.storage().instance().get(&DataKeys::Admin).unwrap()
    }

    fn get_core_state(env: Env) -> CoreState {
        bump_instance(&env);
        get_core_state(&env)
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        bump_instance(&env);
        check_admin(&env);
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn version(env: Env) -> (Symbol, Symbol) {
        bump_instance(&env);
        (CONTRACT_DESCRIPTION, CONTRACT_VERSION)
    }

    fn set_vault_conditions(
        env: Env,
        min_col_rate: i128,
        min_debt_creation: i128,
        opening_col_rate: i128,
        denomination: Symbol,
    ) {
        bump_instance(&env);
        check_admin(&env);
        check_positive(&env, &min_col_rate);
        check_positive(&env, &min_debt_creation);
        check_positive(&env, &opening_col_rate);
        set_currency_vault_conditions(
            &env,
            &min_col_rate,
            &min_debt_creation,
            &opening_col_rate,
            &denomination,
        );
    }

    fn get_vault_conditions(env: Env, denomination: Symbol) -> CurrencyVaultsConditions {
        bump_instance(&env);
        get_currency_vault_conditions(&env, &denomination)
    }

    fn create_currency(env: Env, denomination: Symbol, contract: Address) {
        bump_instance(&env);
        check_protocol_manager(&env);

        if env
            .storage()
            .instance()
            .has(&DataKeys::Currency(denomination.clone()))
        {
            panic_with_error!(&env, &SCErrors::CurrencyAlreadyAdded);
        }

        save_currency(
            &env,
            &Currency {
                denomination,
                active: false,
                contract,
                rate: 0,
                last_updte: env.ledger().timestamp(),
            },
        );
    }

    fn get_currency(env: Env, denomination: Symbol) -> Currency {
        bump_instance(&env);
        validate_currency(&env, &denomination);
        get_currency(&env, &denomination)
    }

    fn get_currency_stats(env: Env, denomination: Symbol) -> CurrencyStats {
        bump_instance(&env);
        validate_currency(&env, &denomination);
        get_currency_stats(&env, &denomination)
    }

    fn set_currency_rate(env: Env, denomination: Symbol, rate: i128) {
        bump_instance(&env);
        // TODO: this method should be updated in the future once there are oracles in the network
        check_oracle_admin(&env);
        validate_currency(&env, &denomination);
        check_positive(&env, &rate);

        let mut currency = get_currency(&env, &denomination);

        // TODO: Check if the price was updated recently
        if currency.rate != rate {
            currency.rate = rate;
            currency.last_updte = env.ledger().timestamp();
            save_currency(&env, &currency);
        } else {
            // TODO: if the last time the rate was changed was more than 15 minutes ago shut down the issuance of new debt
        }
    }

    fn toggle_currency(env: Env, denomination: Symbol, active: bool) {
        bump_instance(&env);
        check_admin(&env);
        validate_currency(&env, &denomination);
        let mut currency = get_currency(&env, &denomination);
        currency.active = active;
        save_currency(&env, &currency);
    }

    fn new_vault(
        env: Env,
        caller: Address,
        initial_debt: i128,
        collateral_amount: i128,
        denomination: Symbol,
    ) {
        bump_instance(&env);
        // TODO: check if we are in panic mode once is implemented

        caller.require_auth();
        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        vault_spot_available(&env, caller.clone(), &denomination);
        check_positive(&env, &initial_debt);
        check_positive(&env, &collateral_amount);

        // TODO: check if collateral price has been updated lately

        let currency_vault_conditions: CurrencyVaultsConditions =
            get_currency_vault_conditions(&env, &denomination);

        validate_initial_debt(&env, &currency_vault_conditions, initial_debt);

        let currency: Currency = get_currency(&env, &denomination);

        let collateral_value: i128 = currency.rate * collateral_amount;

        let deposit_collateral_rate: i128 = div_floor(collateral_value, initial_debt);

        if deposit_collateral_rate < currency_vault_conditions.min_col_rate {
            panic_with_error!(&env, SCErrors::InvalidOpeningCollateralRatio);
        }

        // TODO: Add fee logic
        let new_vault = UserVault {
            id: caller.clone(),
            total_debt: initial_debt,
            total_col: collateral_amount,
            index: calculate_user_vault_index(initial_debt, collateral_amount),
            denomination: denomination.clone(),
        };
        let user_vault_data_type: UserVaultDataType = UserVaultDataType {
            user: caller.clone(),
            denomination: denomination.clone(),
        };
        let vaults_data_types_with_index_key: VaultsDataKeys =
            VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                index: new_vault.index,
                denomination: denomination.clone(),
            });
        let vaults_indexes_list_key: VaultsDataKeys =
            VaultsDataKeys::VaultsIndexes(denomination.clone());

        let core_state: CoreState = get_core_state(&env);

        deposit_collateral(&env, &core_state, &caller, &collateral_amount);

        save_new_user_vault(
            &env,
            &new_vault,
            &user_vault_data_type,
            &vaults_data_types_with_index_key,
            &vaults_indexes_list_key,
        );

        withdraw_stablecoin(&env, &core_state, &currency, &caller, &initial_debt);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);

        currency_stats.total_vaults = currency_stats.total_vaults + 1;
        currency_stats.total_debt = currency_stats.total_debt + initial_debt;
        currency_stats.total_col = currency_stats.total_col + collateral_amount;

        set_currency_stats(&env, &denomination, &currency_stats);

        bump_user_vault(&env, user_vault_data_type);
        bump_vaults_data_types_with_index(&env, &vaults_data_types_with_index_key);
        bump_vaults_indexes_list(&env, &vaults_indexes_list_key);
    }

    fn get_vault(env: Env, user: Address, denomination: Symbol) -> UserVault {
        bump_instance(&env);

        let vaults_indexes_list_key: VaultsDataKeys =
            VaultsDataKeys::VaultsIndexes(denomination.clone());
        let user_vault_data_type: UserVaultDataType = UserVaultDataType { user, denomination };

        validate_user_vault(&env, &user_vault_data_type);
        let user_vault: UserVault = get_user_vault(&env, &user_vault_data_type);

        let vaults_data_types_with_index_key: VaultsDataKeys =
            VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                index: user_vault.index.clone(),
                denomination: user_vault.denomination.clone(),
            });

        bump_user_vault(&env, user_vault_data_type);
        bump_vaults_data_types_with_index(&env, &vaults_data_types_with_index_key);
        bump_vaults_indexes_list(&env, &vaults_indexes_list_key);

        user_vault
    }

    fn incr_col(env: Env, caller: Address, collateral_amount: i128, denomination: Symbol) {
        bump_instance(&env);
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        check_positive(&env, &collateral_amount);

        let user_vault_data_type: UserVaultDataType = UserVaultDataType {
            user: caller.clone(),
            denomination: denomination.clone(),
        };
        let vaults_indexes_list_key: VaultsDataKeys =
            VaultsDataKeys::VaultsIndexes(denomination.clone());

        validate_user_vault(&env, &user_vault_data_type);

        // TODO: Add fee logic

        let core_state: CoreState = get_core_state(&env);

        deposit_collateral(&env, &core_state, &caller, &collateral_amount);

        let current_user_vault: UserVault = get_user_vault(&env, &user_vault_data_type);
        let mut new_user_vault: UserVault = current_user_vault.clone();
        new_user_vault.total_col = new_user_vault.total_col + collateral_amount;
        new_user_vault.index =
            calculate_user_vault_index(new_user_vault.total_debt, new_user_vault.total_col);

        let current_vaults_data_types_with_index_key: VaultsDataKeys =
            VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                index: current_user_vault.index.clone(),
                denomination: denomination.clone(),
            });

        let new_vaults_data_types_with_index_key: VaultsDataKeys =
            VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                index: new_user_vault.index.clone(),
                denomination: denomination.clone(),
            });

        update_user_vault(
            &env,
            &current_user_vault,
            &new_user_vault,
            &user_vault_data_type,
            &vaults_indexes_list_key,
            &current_vaults_data_types_with_index_key,
            &new_vaults_data_types_with_index_key,
        );

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);
        currency_stats.total_col = currency_stats.total_col + collateral_amount;
        set_currency_stats(&env, &denomination, &currency_stats);

        bump_user_vault(&env, user_vault_data_type);
        bump_vaults_data_types_with_index(&env, &new_vaults_data_types_with_index_key);
        bump_vaults_indexes_list(&env, &vaults_indexes_list_key);
    }

    fn incr_debt(env: Env, caller: Address, debt_amount: i128, denomination: Symbol) {
        bump_instance(&env);
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        check_positive(&env, &debt_amount);

        let user_vault_data_type: UserVaultDataType = UserVaultDataType {
            user: caller.clone(),
            denomination: denomination.clone(),
        };

        validate_user_vault(&env, &user_vault_data_type);

        // TODO: Add fee logic
        // TODO: check if we are in panic mode once is implemented
        // TODO: check if collateral price has been updated lately

        let core_state: CoreState = get_core_state(&env);

        let currency: Currency = get_currency(&env, &denomination);

        let current_user_vault: UserVault = get_user_vault(&env, &user_vault_data_type);
        let mut new_user_vault: UserVault = current_user_vault.clone();

        let currency_vault_conditions: CurrencyVaultsConditions =
            get_currency_vault_conditions(&env, &denomination);

        let new_debt_amount: i128 = current_user_vault.total_debt + debt_amount;

        let collateral_value: i128 = currency.rate * current_user_vault.total_col;

        let deposit_rate: i128 = div_floor(collateral_value, new_debt_amount);

        if deposit_rate < currency_vault_conditions.opening_col_rate {
            panic_with_error!(&env, SCErrors::CollateralRateUnderMinimum);
        }

        withdraw_stablecoin(&env, &core_state, &currency, &caller, &debt_amount);

        new_user_vault.total_debt = new_debt_amount;
        new_user_vault.index =
            calculate_user_vault_index(new_user_vault.total_debt, new_user_vault.total_col);

        let current_vaults_data_types_with_index_key: VaultsDataKeys =
            VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                index: current_user_vault.index.clone(),
                denomination: denomination.clone(),
            });

        let new_vaults_data_types_with_index_key: VaultsDataKeys =
            VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                index: new_user_vault.index.clone(),
                denomination: denomination.clone(),
            });

        let vaults_indexes_list_key: VaultsDataKeys =
            VaultsDataKeys::VaultsIndexes(denomination.clone());

        update_user_vault(
            &env,
            &current_user_vault,
            &new_user_vault,
            &user_vault_data_type,
            &vaults_indexes_list_key,
            &current_vaults_data_types_with_index_key,
            &new_vaults_data_types_with_index_key,
        );

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);
        currency_stats.total_debt = currency_stats.total_debt + debt_amount;
        set_currency_stats(&env, &denomination, &currency_stats);

        bump_user_vault(&env, user_vault_data_type);
        bump_vaults_data_types_with_index(&env, &new_vaults_data_types_with_index_key);
        bump_vaults_indexes_list(&env, &vaults_indexes_list_key);
    }

    fn pay_debt(env: Env, caller: Address, deposit_amount: i128, denomination: Symbol) {
        bump_instance(&env);
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        check_positive(&env, &deposit_amount);

        let user_vault_data_type: UserVaultDataType = UserVaultDataType {
            user: caller.clone(),
            denomination: denomination.clone(),
        };

        validate_user_vault(&env, &user_vault_data_type);

        // TODO: Add fee logic

        let currency: Currency = get_currency(&env, &denomination);

        let current_user_vault: UserVault = get_user_vault(&env, &user_vault_data_type);
        let mut new_user_vault: UserVault = current_user_vault.clone();

        if deposit_amount > current_user_vault.total_debt {
            panic_with_error!(&env, SCErrors::DepositAmountIsMoreThanTotalDebt);
        }

        let core_state: CoreState = get_core_state(&env);

        deposit_stablecoin(&env, &core_state, &currency, &caller, &deposit_amount);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);

        let current_vaults_data_types_with_index_key: VaultsDataKeys =
            VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                index: current_user_vault.index.clone(),
                denomination: denomination.clone(),
            });

        let vaults_indexes_list_key: VaultsDataKeys =
            VaultsDataKeys::VaultsIndexes(denomination.clone());

        if current_user_vault.total_debt == deposit_amount {
            // If the amount is equal to the debt it means it is paid in full so we release the collateral and remove the vault
            currency_stats.total_vaults = currency_stats.total_vaults - 1;
            currency_stats.total_col = currency_stats.total_col - current_user_vault.total_col;

            token::Client::new(&env, &core_state.col_token).transfer(
                &env.current_contract_address(),
                &caller,
                &current_user_vault.total_col,
            );

            let vaults_data_types_with_index_key: VaultsDataKeys =
                VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                    index: current_user_vault.index,
                    denomination: denomination.clone(),
                });

            let vaults_indexes_list_key: VaultsDataKeys =
                VaultsDataKeys::VaultsIndexes(denomination.clone());

            remove_user_vault(
                &env,
                &current_user_vault,
                &user_vault_data_type,
                &vaults_data_types_with_index_key,
                &vaults_indexes_list_key,
            );

            bump_vaults_data_types_with_index(&env, &vaults_data_types_with_index_key);
        } else {
            // If amount is not enough to pay all the debt, we just updated the stats of the user's vault
            new_user_vault.total_debt = new_user_vault.total_debt - deposit_amount;
            new_user_vault.index =
                calculate_user_vault_index(new_user_vault.total_debt, new_user_vault.total_col);

            let new_vaults_data_types_with_index_key: VaultsDataKeys =
                VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                    index: new_user_vault.index.clone(),
                    denomination: denomination.clone(),
                });

            update_user_vault(
                &env,
                &current_user_vault,
                &new_user_vault,
                &user_vault_data_type,
                &vaults_indexes_list_key,
                &current_vaults_data_types_with_index_key,
                &new_vaults_data_types_with_index_key,
            );

            bump_vaults_data_types_with_index(&env, &new_vaults_data_types_with_index_key);
        }

        currency_stats.total_debt = currency_stats.total_debt - deposit_amount;
        set_currency_stats(&env, &denomination, &currency_stats);

        bump_user_vault(&env, user_vault_data_type);
        bump_vaults_indexes_list(&env, &vaults_indexes_list_key);
    }

    fn get_indexes(env: Env, denomination: Symbol) -> Vec<i128> {
        bump_instance(&env);

        let vaults_indexes_list_key: VaultsDataKeys = VaultsDataKeys::VaultsIndexes(denomination);

        bump_vaults_indexes_list(&env, &vaults_indexes_list_key);
        get_vaults_indexes_list(&env, &vaults_indexes_list_key)
    }

    fn get_vaults_with_index(env: Env, denomination: Symbol, index: i128) -> Vec<UserVault> {
        bump_instance(&env);

        let vaults_data_types_with_index_key: VaultsDataKeys =
            VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                index,
                denomination: denomination.clone(),
            });

        let data_keys: Vec<UserVaultDataType> =
            get_vaults_data_type_with_index(&env, &vaults_data_types_with_index_key);
        let mut vaults: Vec<UserVault> = vec![&env] as Vec<UserVault>;

        for user_vault_data_type in data_keys.iter() {
            let vault: UserVault = get_user_vault(&env, &user_vault_data_type);

            bump_user_vault(&env, user_vault_data_type);
            vaults.push_back(vault);
        }

        let vaults_indexes_list_key: VaultsDataKeys =
            VaultsDataKeys::VaultsIndexes(denomination.clone());

        bump_vaults_data_types_with_index(&env, &vaults_data_types_with_index_key);
        bump_vaults_indexes_list(&env, &vaults_indexes_list_key);

        vaults
    }

    fn redeem(env: Env, caller: Address, amount_to_redeem: i128, denomination: Symbol) {
        bump_instance(&env);
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        check_positive(&env, &amount_to_redeem);

        // TODO: Add fee logic

        let core_state: CoreState = get_core_state(&env);
        let currency: Currency = get_currency(&env, &denomination);

        let vaults_indexes_list_key: VaultsDataKeys =
            VaultsDataKeys::VaultsIndexes(denomination.clone());

        let redeemable_vaults: Vec<UserVault> =
            get_redeemable_vaults(&env, &amount_to_redeem, &currency, &vaults_indexes_list_key);

        deposit_stablecoin(&env, &core_state, &currency, &caller, &amount_to_redeem);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);

        // Update the redeemable vaults information
        let mut amount_redeemed: i128 = 0;
        let mut collateral_to_withdraw: i128 = 0;

        for current_user_vault in redeemable_vaults.iter() {
            let user_vault_data_type: UserVaultDataType = UserVaultDataType {
                user: current_user_vault.id.clone(),
                denomination: current_user_vault.denomination.clone(),
            };

            let vaults_data_types_with_index_key: VaultsDataKeys =
                VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                    index: current_user_vault.index,
                    denomination: denomination.clone(),
                });

            let vaults_indexes_list_key: VaultsDataKeys =
                VaultsDataKeys::VaultsIndexes(denomination.clone());

            if (amount_redeemed + current_user_vault.total_debt) > amount_to_redeem {
                let mut new_user_vault: UserVault = current_user_vault.clone();
                let missing_amount: i128 = amount_to_redeem - amount_redeemed;
                let missing_collateral: i128 = div_floor(missing_amount * 10000000, currency.rate);

                new_user_vault.total_col = new_user_vault.total_col - missing_collateral;
                new_user_vault.total_debt = new_user_vault.total_debt - missing_amount;
                new_user_vault.index =
                    calculate_user_vault_index(new_user_vault.total_debt, new_user_vault.total_col);

                currency_stats.total_col = currency_stats.total_col - missing_collateral;
                currency_stats.total_debt = currency_stats.total_debt - missing_amount;

                collateral_to_withdraw = collateral_to_withdraw + missing_collateral;
                amount_redeemed = amount_redeemed + missing_amount;

                let current_vaults_data_types_with_index_key: VaultsDataKeys =
                    VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                        index: current_user_vault.index.clone(),
                        denomination: denomination.clone(),
                    });

                let new_vaults_data_types_with_index_key: VaultsDataKeys =
                    VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                        index: new_user_vault.index.clone(),
                        denomination: denomination.clone(),
                    });

                update_user_vault(
                    &env,
                    &current_user_vault,
                    &new_user_vault,
                    &user_vault_data_type,
                    &vaults_indexes_list_key,
                    &current_vaults_data_types_with_index_key,
                    &new_vaults_data_types_with_index_key,
                );

                bump_user_vault(&env, user_vault_data_type);
                bump_vaults_data_types_with_index(&env, &vaults_data_types_with_index_key);
            } else {
                let collateral_amount =
                    div_floor(current_user_vault.total_debt * 10000000, currency.rate);

                collateral_to_withdraw = collateral_to_withdraw + collateral_amount;
                amount_redeemed = amount_redeemed + current_user_vault.total_debt;

                currency_stats.total_vaults = currency_stats.total_vaults - 1;
                currency_stats.total_col = currency_stats.total_col - current_user_vault.total_col;
                currency_stats.total_debt =
                    currency_stats.total_debt - current_user_vault.total_debt;

                withdraw_collateral(
                    &env,
                    &core_state,
                    &current_user_vault.id,
                    &(current_user_vault.total_col - collateral_amount),
                );

                remove_user_vault(
                    &env,
                    &current_user_vault,
                    &user_vault_data_type,
                    &vaults_data_types_with_index_key,
                    &vaults_indexes_list_key,
                );

                bump_vaults_data_types_with_index(&env, &vaults_data_types_with_index_key);
            }
        }

        withdraw_collateral(&env, &core_state, &caller, &collateral_to_withdraw);
        set_currency_stats(&env, &denomination, &currency_stats);

        bump_vaults_indexes_list(&env, &vaults_indexes_list_key);
    }

    fn liquidate(env: Env, liquidator: Address, denomination: Symbol, owners: Vec<Address>) {
        bump_instance(&env);
        liquidator.require_auth();

        // TODO: Add fee logic

        let core_state: CoreState = get_core_state(&env);
        let currency: Currency = get_currency(&env, &denomination);
        let currency_vault_conditions: CurrencyVaultsConditions =
            get_currency_vault_conditions(&env, &denomination);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);
        let mut collateral_to_withdraw: i128 = 0;
        let mut amount_to_deposit: i128 = 0;

        for owner in owners.iter() {
            let user_vault_data_type: UserVaultDataType = UserVaultDataType {
                user: owner,
                denomination: denomination.clone(),
            };
            let user_vault: UserVault = get_user_vault(&env, &user_vault_data_type);

            if !can_be_liquidated(&user_vault, &currency, &currency_vault_conditions) {
                panic_with_error!(&env, SCErrors::UserVaultCantBeLiquidated);
            }

            collateral_to_withdraw = collateral_to_withdraw + user_vault.total_col;
            amount_to_deposit = amount_to_deposit + user_vault.total_debt;

            currency_stats.total_vaults = currency_stats.total_vaults - 1;
            currency_stats.total_col = currency_stats.total_col - user_vault.total_col;
            currency_stats.total_debt = currency_stats.total_debt - user_vault.total_debt;

            let vaults_data_types_with_index_key: VaultsDataKeys =
                VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                    index: user_vault.index,
                    denomination: denomination.clone(),
                });

            let vaults_indexes_list_key: VaultsDataKeys =
                VaultsDataKeys::VaultsIndexes(denomination.clone());

            remove_user_vault(
                &env,
                &user_vault,
                &user_vault_data_type,
                &vaults_data_types_with_index_key,
                &vaults_indexes_list_key,
            );

            bump_vaults_data_types_with_index(&env, &vaults_data_types_with_index_key);
            bump_vaults_indexes_list(&env, &vaults_indexes_list_key);
        }

        withdraw_collateral(&env, &core_state, &liquidator, &collateral_to_withdraw);
        deposit_stablecoin(
            &env,
            &core_state,
            &currency,
            &liquidator,
            &amount_to_deposit,
        );
        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn vaults_to_liquidate(env: Env, denomination: Symbol) -> Vec<UserVault> {
        bump_instance(&env);

        let vaults_indexes_list_key: VaultsDataKeys =
            VaultsDataKeys::VaultsIndexes(denomination.clone());

        let indexes: Vec<i128> = get_vaults_indexes_list(&env, &vaults_indexes_list_key);
        let mut vaults: Vec<UserVault> = vec![&env] as Vec<UserVault>;
        let mut completed: bool = false;

        let currency: Currency = get_currency(&env, &denomination);
        let currency_vaults_conditions: CurrencyVaultsConditions =
            get_currency_vault_conditions(&env, &denomination);

        for index in indexes.iter() {
            let vaults_data_types_with_index_key: VaultsDataKeys =
                VaultsDataKeys::VaultsDataTypesWithIndex(VaultsWithIndexDataType {
                    index,
                    denomination: denomination.clone(),
                });

            let vaults_data_types: Vec<UserVaultDataType> =
                get_vaults_data_type_with_index(&env, &vaults_data_types_with_index_key);

            for user_vault_data_type in vaults_data_types.iter() {
                let user_vault: UserVault = get_user_vault(&env, &user_vault_data_type);

                if can_be_liquidated(&user_vault, &currency, &currency_vaults_conditions) {
                    // This condition is because the indexes include all denominations
                    if user_vault_data_type.denomination == currency.denomination {
                        vaults.push_back(user_vault);
                    }
                } else {
                    completed = true;
                    break;
                }
            }

            if completed {
                break;
            }
        }

        vaults
    }
}
