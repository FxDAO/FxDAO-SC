use crate::storage_types::*;
use crate::token;
use crate::utils::vaults::*;
use crate::utils::*;
use num_integer::div_floor;

use crate::utils::indexes::get_vaults_data_type_with_index;
use soroban_sdk::{contractimpl, panic_with_error, vec, Address, BytesN, Env, Symbol, Vec};

// TODO: Explain each function here
pub trait VaultsContractTrait {
    /// Set up and management
    fn init(env: Env, admin: Address, col_token: BytesN<32>, stable_issuer: Address);
    fn get_admin(env: Env) -> Address;
    fn get_core_state(env: Env) -> CoreState;

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
    fn create_currency(env: Env, denomination: Symbol, contract: BytesN<32>);
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
    // TODO: Create test which we verify this function works correctly in multi currencies cases
    fn vaults_to_liquidate(env: Env, denomination: Symbol) -> Vec<UserVault>;
}

pub struct VaultsContract;

// TODO: Add events for each function
#[contractimpl]
impl VaultsContractTrait for VaultsContract {
    fn init(env: Env, admin: Address, col_token: BytesN<32>, stable_issuer: Address) {
        if env.storage().has(&DataKeys::CoreState) {
            panic_with_error!(&env, SCErrors::AlreadyInit);
        }

        env.storage().set(
            &DataKeys::CoreState,
            &CoreState {
                col_token,
                stable_issuer,
            },
        );
        env.storage().set(&DataKeys::Admin, &admin);
    }

    fn get_admin(env: Env) -> Address {
        env.storage().get(&DataKeys::Admin).unwrap().unwrap()
    }

    fn get_core_state(env: Env) -> CoreState {
        get_core_state(&env)
    }

    fn set_vault_conditions(
        env: Env,
        min_col_rate: i128,
        min_debt_creation: i128,
        opening_col_rate: i128,
        denomination: Symbol,
    ) {
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
        get_currency_vault_conditions(&env, &denomination)
    }

    fn create_currency(env: Env, denomination: Symbol, contract: BytesN<32>) {
        check_admin(&env);

        if env.storage().has(&DataKeys::Currency(denomination.clone())) {
            panic_with_error!(&env, &SCErrors::CurrencyAlreadyAdded);
        }

        save_currency(
            &env,
            &Currency {
                denomination: denomination,
                active: false,
                contract,
                rate: 0,
                last_updte: env.ledger().timestamp(),
            },
        );
    }

    fn get_currency(env: Env, denomination: Symbol) -> Currency {
        validate_currency(&env, &denomination);
        get_currency(&env, &denomination)
    }

    fn get_currency_stats(env: Env, denomination: Symbol) -> CurrencyStats {
        validate_currency(&env, &denomination);
        get_currency_stats(&env, &denomination)
    }

    fn set_currency_rate(env: Env, denomination: Symbol, rate: i128) {
        // TODO: this method should be updated in the future once there are oracles in the network
        check_admin(&env);
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

        valid_initial_debt(&env, &currency_vault_conditions, initial_debt);

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

        let core_state: CoreState = get_core_state(&env);

        deposit_collateral(&env, &core_state, &caller, &collateral_amount);

        save_new_user_vault(&env, &caller, &denomination, &new_vault);

        withdraw_stablecoin(&env, &core_state, &currency, &caller, &initial_debt);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);

        currency_stats.total_vaults = currency_stats.total_vaults + 1;
        currency_stats.total_debt = currency_stats.total_debt + initial_debt;
        currency_stats.total_col = currency_stats.total_col + collateral_amount;

        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn get_vault(env: Env, user: Address, denomination: Symbol) -> UserVault {
        validate_user_vault(&env, &user, &denomination);
        get_user_vault(&env, &user, &denomination)
    }

    fn incr_col(env: Env, caller: Address, collateral_amount: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        check_positive(&env, &collateral_amount);
        validate_user_vault(&env, &caller, &denomination);

        // TODO: Add fee logic

        let core_state: CoreState = get_core_state(&env);

        deposit_collateral(&env, &core_state, &caller, &collateral_amount);

        let current_user_vault: UserVault = get_user_vault(&env, &caller, &denomination);
        let mut new_user_vault: UserVault = current_user_vault.clone();
        new_user_vault.total_col = new_user_vault.total_col + collateral_amount;
        new_user_vault.index =
            calculate_user_vault_index(new_user_vault.total_debt, new_user_vault.total_col);

        update_user_vault(
            &env,
            &caller,
            &denomination,
            &current_user_vault,
            &new_user_vault,
        );

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);
        currency_stats.total_col = currency_stats.total_col + collateral_amount;
        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn incr_debt(env: Env, caller: Address, debt_amount: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        check_positive(&env, &debt_amount);
        validate_user_vault(&env, &caller, &denomination);

        // TODO: Add fee logic
        // TODO: check if we are in panic mode once is implemented
        // TODO: check if collateral price has been updated lately

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        let currency: Currency = get_currency(&env, &denomination);

        let current_user_vault: UserVault = get_user_vault(&env, &caller, &denomination);
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
        update_user_vault(
            &env,
            &caller,
            &denomination,
            &current_user_vault,
            &new_user_vault,
        );

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);
        currency_stats.total_debt = currency_stats.total_debt + debt_amount;
        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn pay_debt(env: Env, caller: Address, deposit_amount: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        check_positive(&env, &deposit_amount);
        validate_user_vault(&env, &caller, &denomination);

        // TODO: Add fee logic

        let currency: Currency = get_currency(&env, &denomination);

        let current_user_vault: UserVault = get_user_vault(&env, &caller, &denomination);
        let mut updated_user_vault: UserVault = current_user_vault.clone();

        if deposit_amount > current_user_vault.total_debt {
            panic_with_error!(&env, SCErrors::DepositAmountIsMoreThanTotalDebt);
        }

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        deposit_stablecoin(&env, &core_state, &currency, &caller, &deposit_amount);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);

        if current_user_vault.total_debt == deposit_amount {
            // If the amount is equal to the debt it means it is paid in full so we release the collateral and remove the vault
            currency_stats.total_vaults = currency_stats.total_vaults - 1;
            currency_stats.total_col = currency_stats.total_col - current_user_vault.total_col;

            token::Client::new(&env, &core_state.col_token).xfer(
                &env.current_contract_address(),
                &caller,
                &current_user_vault.total_col,
            );

            remove_user_vault(&env, &caller, &denomination, &current_user_vault);
        } else {
            // If amount is not enough to pay all the debt, we just updated the stats of the user's vault
            updated_user_vault.total_debt = updated_user_vault.total_debt - deposit_amount;
            updated_user_vault.index = calculate_user_vault_index(
                updated_user_vault.total_debt,
                updated_user_vault.total_col,
            );
            update_user_vault(
                &env,
                &caller,
                &denomination,
                &current_user_vault,
                &updated_user_vault,
            );
        }

        currency_stats.total_debt = currency_stats.total_debt - deposit_amount;
        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn get_indexes(env: Env, denomination: Symbol) -> Vec<i128> {
        get_sorted_indexes_list(&env, &denomination)
    }

    fn get_vaults_with_index(env: Env, denomination: Symbol, index: i128) -> Vec<UserVault> {
        let data_keys: Vec<UserVaultDataType> =
            get_vaults_data_type_with_index(&env, &denomination, &index);
        let mut vaults: Vec<UserVault> = vec![&env] as Vec<UserVault>;

        for result in data_keys.iter() {
            let data_key: UserVaultDataType = result.unwrap();
            let vault: UserVault = get_user_vault(&env, &data_key.user, &data_key.denomination);
            vaults.push_back(vault);
        }

        vaults
    }

    fn redeem(env: Env, caller: Address, amount_to_redeem: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        check_positive(&env, &amount_to_redeem);

        // TODO: Add fee logic

        let core_state: CoreState = get_core_state(&env);
        let currency: Currency = get_currency(&env, &denomination);

        let redeemable_vaults: Vec<UserVault> =
            get_redeemable_vaults(&env, &amount_to_redeem, &currency);

        deposit_stablecoin(&env, &core_state, &currency, &caller, &amount_to_redeem);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);

        // Update the redeemable vaults information
        let mut amount_redeemed: i128 = 0;
        let mut collateral_to_withdraw: i128 = 0;

        for redeemable_vault in redeemable_vaults.iter() {
            let user_vault: UserVault = redeemable_vault.unwrap();

            if (amount_redeemed + user_vault.total_debt) > amount_to_redeem {
                let mut updated_vault: UserVault = user_vault.clone();
                let missing_amount: i128 = amount_to_redeem - amount_redeemed;
                let missing_collateral: i128 = div_floor(missing_amount * 10000000, currency.rate);

                updated_vault.total_col = updated_vault.total_col - missing_collateral;
                updated_vault.total_debt = updated_vault.total_debt - missing_amount;
                updated_vault.index =
                    calculate_user_vault_index(updated_vault.total_debt, updated_vault.total_col);

                currency_stats.total_col = currency_stats.total_col - missing_collateral;
                currency_stats.total_debt = currency_stats.total_debt - missing_amount;

                collateral_to_withdraw = collateral_to_withdraw + missing_collateral;
                amount_redeemed = amount_redeemed + missing_amount;

                update_user_vault(
                    &env,
                    &user_vault.id,
                    &denomination,
                    &user_vault,
                    &updated_vault,
                );
            } else {
                let collateral_amount = div_floor(user_vault.total_debt * 10000000, currency.rate);

                collateral_to_withdraw = collateral_to_withdraw + collateral_amount;
                amount_redeemed = amount_redeemed + user_vault.total_debt;

                currency_stats.total_vaults = currency_stats.total_vaults - 1;
                currency_stats.total_col = currency_stats.total_col - user_vault.total_col;
                currency_stats.total_debt = currency_stats.total_debt - user_vault.total_debt;

                withdraw_collateral(
                    &env,
                    &core_state,
                    &user_vault.id,
                    &(user_vault.total_col - collateral_amount),
                );

                remove_user_vault(&env, &user_vault.id, &denomination, &user_vault);
            }
        }

        withdraw_collateral(&env, &core_state, &caller, &collateral_to_withdraw);
        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn liquidate(env: Env, liquidator: Address, denomination: Symbol, owners: Vec<Address>) {
        liquidator.require_auth();

        // TODO: Add fee logic

        let core_state: CoreState = get_core_state(&env);
        let currency: Currency = get_currency(&env, &denomination);
        let currency_vault_conditions: CurrencyVaultsConditions =
            get_currency_vault_conditions(&env, &denomination);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);
        let mut collateral_to_withdraw: i128 = 0;
        let mut amount_to_deposit: i128 = 0;

        for item in owners.iter() {
            let owner: Address = item.unwrap();
            let user_vault: UserVault = get_user_vault(&env, &owner, &denomination);

            if !can_be_liquidated(&user_vault, &currency, &currency_vault_conditions) {
                panic_with_error!(&env, &SCErrors::UserVaultCantBeLiquidated);
            }

            collateral_to_withdraw = collateral_to_withdraw + user_vault.total_col;
            amount_to_deposit = amount_to_deposit + user_vault.total_debt;

            currency_stats.total_vaults = currency_stats.total_vaults - 1;
            currency_stats.total_col = currency_stats.total_col - user_vault.total_col;
            currency_stats.total_debt = currency_stats.total_debt - user_vault.total_debt;

            remove_user_vault(&env, &owner, &denomination, &user_vault);
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
        let indexes: Vec<i128> = get_sorted_indexes_list(&env, &denomination);
        let mut vaults: Vec<UserVault> = vec![&env] as Vec<UserVault>;
        let mut completed: bool = false;

        let currency: Currency = get_currency(&env, &denomination);
        let currency_vaults_conditions: CurrencyVaultsConditions =
            get_currency_vault_conditions(&env, &denomination);

        for result in indexes.iter() {
            let index: i128 = result.unwrap();
            let vaults_data_types: Vec<UserVaultDataType> =
                get_vaults_data_type_with_index(&env, &denomination, &index);

            for result2 in vaults_data_types.iter() {
                let vault_data_type: UserVaultDataType = result2.unwrap();

                let user_vault: UserVault =
                    get_user_vault(&env, &vault_data_type.user, &vault_data_type.denomination);

                if can_be_liquidated(&user_vault, &currency, &currency_vaults_conditions) {
                    // This condition is because the indexes include all denominations
                    if vault_data_type.denomination == currency.denomination {
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
