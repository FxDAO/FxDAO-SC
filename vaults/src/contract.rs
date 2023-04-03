use crate::storage_types::*;
use crate::token;
use crate::utils::vaults::*;
use crate::utils::*;
use num_integer::div_floor;

use soroban_sdk::{contractimpl, panic_with_error, Address, BytesN, Env, Symbol, Vec};

// TODO: Explain each function here
pub trait VaultsContractTrait {
    /// Set up and management
    fn init(env: Env, admin: Address, colla_tokn: BytesN<32>, stble_issr: Address);
    fn get_admin(env: Env) -> Address;
    fn g_c_state(env: Env) -> CoreState;

    /// Currency vaults conditions
    fn s_c_v_c(
        env: Env,
        mn_col_rte: i128,
        mn_v_c_amt: i128,
        op_col_rte: i128,
        denomination: Symbol,
    );
    fn g_c_v_c(env: Env, denomination: Symbol) -> CurrencyVaultsConditions;

    /// Currencies methods
    fn new_cy(env: Env, denomination: Symbol, contract: BytesN<32>);
    fn get_cy(env: Env, denomination: Symbol) -> Currency;
    fn g_cy_stats(env: Env, denomination: Symbol) -> CurrencyStats;
    fn s_cy_rate(env: Env, denomination: Symbol, rate: i128);
    fn toggle_cy(env: Env, denomination: Symbol, active: bool);

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
    fn g_indexes(env: Env, denomination: Symbol) -> Vec<i128>;

    /// Redeeming
    fn redeem(env: Env, caller: Address, amount: i128, denomination: Symbol);
}

pub struct VaultsContract;

#[contractimpl]
impl VaultsContractTrait for VaultsContract {
    fn init(env: Env, admin: Address, colla_tokn: BytesN<32>, stble_issr: Address) {
        if env.storage().has(&DataKeys::CoreState) {
            panic_with_error!(&env, SCErrors::AlreadyInit);
        }

        env.storage().set(
            &DataKeys::CoreState,
            &CoreState {
                colla_tokn,
                stble_issr,
            },
        );
        env.storage().set(&DataKeys::Admin, &admin);
    }

    fn get_admin(env: Env) -> Address {
        env.storage().get(&DataKeys::Admin).unwrap().unwrap()
    }

    fn g_c_state(env: Env) -> CoreState {
        get_core_state(&env)
    }

    fn s_c_v_c(
        env: Env,
        mn_col_rte: i128,
        mn_v_c_amt: i128,
        op_col_rte: i128,
        denomination: Symbol,
    ) {
        check_admin(&env);
        check_positive(&env, &mn_col_rte);
        check_positive(&env, &mn_v_c_amt);
        check_positive(&env, &op_col_rte);
        set_currency_vault_conditions(&env, &mn_col_rte, &mn_v_c_amt, &op_col_rte, &denomination);
    }

    fn g_c_v_c(env: Env, denomination: Symbol) -> CurrencyVaultsConditions {
        get_currency_vault_conditions(&env, &denomination)
    }

    fn new_cy(env: Env, denomination: Symbol, contract: BytesN<32>) {
        check_admin(&env);

        if env.storage().has(&DataKeys::Currency(denomination)) {
            panic_with_error!(&env, &SCErrors::CurrencyAlreadyAdded);
        }

        save_currency(
            &env,
            Currency {
                symbol: denomination,
                active: false,
                contract,
                rate: 0,
                last_updte: env.ledger().timestamp(),
            },
        );
    }

    fn get_cy(env: Env, denomination: Symbol) -> Currency {
        validate_currency(&env, denomination);
        get_currency(&env, denomination)
    }

    fn g_cy_stats(env: Env, denomination: Symbol) -> CurrencyStats {
        validate_currency(&env, denomination);
        get_currency_stats(&env, &denomination)
    }

    fn s_cy_rate(env: Env, denomination: Symbol, rate: i128) {
        // TODO: this method should be updated in the future once there are oracles in the network
        check_admin(&env);
        validate_currency(&env, denomination);
        check_positive(&env, &rate);

        let mut currency = get_currency(&env, denomination);

        // TODO: Check if the price was updated recently
        if currency.rate != rate {
            currency.rate = rate;
            currency.last_updte = env.ledger().timestamp();
            save_currency(&env, currency);
        } else {
            // TODO: if the last time the rate was changed was more than 15 minutes ago shut down the issuance of new debt
        }
    }

    fn toggle_cy(env: Env, denomination: Symbol, active: bool) {
        check_admin(&env);
        validate_currency(&env, denomination);
        let mut currency = get_currency(&env, denomination);
        currency.active = active;
        save_currency(&env, currency);
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
        validate_currency(&env, denomination);
        is_currency_active(&env, denomination);
        vault_spot_available(&env, caller.clone(), denomination);
        check_positive(&env, &initial_debt);
        check_positive(&env, &collateral_amount);

        // TODO: check if collateral price has been updated lately

        let currency_vault_conditions: CurrencyVaultsConditions =
            get_currency_vault_conditions(&env, &denomination);

        valid_initial_debt(&env, &currency_vault_conditions, initial_debt);

        let currency: Currency = get_currency(&env, denomination);

        let collateral_value: i128 = currency.rate * collateral_amount;

        let deposit_collateral_rate: i128 = div_floor(collateral_value, initial_debt);

        if deposit_collateral_rate < currency_vault_conditions.mn_col_rte {
            panic_with_error!(&env, SCErrors::InvalidOpeningCollateralRatio);
        }

        // TODO: Add fee logic
        let new_vault = UserVault {
            id: caller.clone(),
            total_debt: initial_debt,
            total_col: collateral_amount,
            index: calculate_user_vault_index(initial_debt, collateral_amount),
        };

        let core_state: CoreState = get_core_state(&env);

        deposit_collateral(&env, &core_state, &caller, &collateral_amount);

        save_new_user_vault(&env, &caller, &denomination, &new_vault);

        withdraw_stablecoin(&env, &core_state, &currency, &caller, &initial_debt);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);

        currency_stats.tot_vaults = currency_stats.tot_vaults + 1;
        currency_stats.tot_debt = currency_stats.tot_debt + initial_debt;
        currency_stats.tot_col = currency_stats.tot_col + collateral_amount;

        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn get_vault(env: Env, user: Address, denomination: Symbol) -> UserVault {
        validate_user_vault(&env, user.clone(), denomination);
        get_user_vault(&env, user.clone(), denomination)
    }

    fn incr_col(env: Env, caller: Address, collateral_amount: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, denomination);
        is_currency_active(&env, denomination);
        check_positive(&env, &collateral_amount);
        validate_user_vault(&env, caller.clone(), denomination);

        // TODO: Add fee logic

        let core_state: CoreState = get_core_state(&env);

        deposit_collateral(&env, &core_state, &caller, &collateral_amount);

        let current_user_vault: UserVault = get_user_vault(&env, caller.clone(), denomination);
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
        currency_stats.tot_col = currency_stats.tot_col + collateral_amount;
        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn incr_debt(env: Env, caller: Address, debt_amount: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, denomination);
        is_currency_active(&env, denomination);
        check_positive(&env, &debt_amount);
        validate_user_vault(&env, caller.clone(), denomination);

        // TODO: Add fee logic
        // TODO: check if we are in panic mode once is implemented
        // TODO: check if collateral price has been updated lately

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        let currency: Currency = get_currency(&env, denomination);

        let current_user_vault: UserVault = get_user_vault(&env, caller.clone(), denomination);
        let mut new_user_vault: UserVault = current_user_vault.clone();

        let currency_vault_conditions: CurrencyVaultsConditions =
            get_currency_vault_conditions(&env, &denomination);

        let new_debt_amount: i128 = current_user_vault.total_debt + debt_amount;

        let collateral_value: i128 = currency.rate * current_user_vault.total_col;

        let deposit_rate: i128 = div_floor(collateral_value, new_debt_amount);

        if deposit_rate < currency_vault_conditions.op_col_rte {
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
        currency_stats.tot_debt = currency_stats.tot_debt + debt_amount;
        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn pay_debt(env: Env, caller: Address, deposit_amount: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, denomination);
        is_currency_active(&env, denomination);
        check_positive(&env, &deposit_amount);
        validate_user_vault(&env, caller.clone(), denomination);

        // TODO: Add fee logic

        let currency: Currency = get_currency(&env, denomination);

        let current_user_vault: UserVault = get_user_vault(&env, caller.clone(), denomination);
        let mut updated_user_vault: UserVault = current_user_vault.clone();

        if deposit_amount > current_user_vault.total_debt {
            panic_with_error!(&env, SCErrors::DepositAmountIsMoreThanTotalDebt);
        }

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        deposit_stablecoin(&env, &core_state, &currency, &caller, &deposit_amount);

        let mut currency_stats: CurrencyStats = get_currency_stats(&env, &denomination);

        if current_user_vault.total_debt == deposit_amount {
            // If the amount is equal to the debt it means it is paid in full so we release the collateral and remove the vault
            currency_stats.tot_vaults = currency_stats.tot_vaults - 1;
            currency_stats.tot_col = currency_stats.tot_col - current_user_vault.total_col;

            token::Client::new(&env, &core_state.colla_tokn).xfer(
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

        currency_stats.tot_debt = currency_stats.tot_debt - deposit_amount;
        set_currency_stats(&env, &denomination, &currency_stats);
    }

    fn g_indexes(env: Env, denomination: Symbol) -> Vec<i128> {
        get_sorted_indexes_list(&env, &denomination)
    }

    fn redeem(env: Env, caller: Address, amount_to_redeem: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, denomination);
        is_currency_active(&env, denomination);
        check_positive(&env, &amount_to_redeem);

        // TODO: Add fee logic

        let core_state: CoreState = get_core_state(&env);
        let currency: Currency = get_currency(&env, denomination);

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

                currency_stats.tot_col = currency_stats.tot_col - missing_collateral;
                currency_stats.tot_debt = currency_stats.tot_debt - missing_amount;

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

                currency_stats.tot_vaults = currency_stats.tot_vaults - 1;
                currency_stats.tot_col = currency_stats.tot_col - user_vault.total_col;
                currency_stats.tot_debt = currency_stats.tot_debt - user_vault.total_debt;

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
}
