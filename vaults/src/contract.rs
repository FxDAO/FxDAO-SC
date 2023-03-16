use crate::storage_types::*;
use crate::token;
use crate::utils::*;
use num_integer::div_floor;

use soroban_sdk::{contractimpl, panic_with_error, Address, BytesN, Env, Symbol};

// TODO: Explain each function here
pub trait VaultsContractTrait {
    fn init(env: Env, admin: Address, colla_tokn: BytesN<32>, stble_issr: Address);

    fn get_admin(env: Env) -> Address;

    fn g_c_state(env: Env) -> CoreState;

    fn s_p_state(env: Env, mn_col_rte: i128, mn_v_c_amt: i128, op_col_rte: i128);
    fn g_p_state(env: Env) -> ProtocolState;

    fn g_p_stats(env: Env) -> ProtStats;

    /// Currencies methods
    fn new_cy(env: Env, denomination: Symbol, contract: BytesN<32>);
    fn get_cy(env: Env, denomination: Symbol) -> Currency;
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
}

pub struct VaultsContract;

#[contractimpl]
impl VaultsContractTrait for VaultsContract {
    fn init(env: Env, admin: Address, colla_tokn: BytesN<32>, stble_issr: Address) {
        if env.storage().has(&DataKeys::CoreState) {
            panic_with_error!(&env, SCErrors::AlreadyInit);
        }

        let core_state: CoreState = CoreState {
            colla_tokn,
            stble_issr,
        };

        env.storage().set(&DataKeys::CoreState, &core_state);
        env.storage().set(&DataKeys::Admin, &admin);
    }

    fn get_admin(env: Env) -> Address {
        env.storage().get(&DataKeys::Admin).unwrap().unwrap()
    }

    fn g_c_state(env: Env) -> CoreState {
        get_core_state(&env)
    }

    fn s_p_state(env: Env, mn_col_rte: i128, mn_v_c_amt: i128, op_col_rte: i128) {
        check_admin(&env);
        check_positive(&env, &mn_col_rte);
        check_positive(&env, &mn_v_c_amt);
        check_positive(&env, &op_col_rte);

        env.storage().set(
            &DataKeys::ProtState,
            &ProtocolState {
                mn_col_rte,
                mn_v_c_amt,
                op_col_rte,
            },
        );
    }

    fn g_p_state(env: Env) -> ProtocolState {
        get_protocol_state(&env)
    }

    fn g_p_stats(env: Env) -> ProtStats {
        get_protocol_stats(&env)
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

        let protocol_state: ProtocolState = get_protocol_state(&env);

        valid_initial_debt(&env, &protocol_state, initial_debt);

        let currency: Currency = get_currency(&env, denomination);

        let collateral_value: i128 = currency.rate * collateral_amount;

        let deposit_rate: i128 = div_floor(collateral_value, initial_debt);

        if deposit_rate < protocol_state.mn_col_rte {
            panic_with_error!(&env, SCErrors::InvalidOpeningCollateralRatio);
        }

        // TODO: Add fee logic
        let new_vault = UserVault {
            id: caller.clone(),
            total_col: collateral_amount,
            total_debt: initial_debt,
        };

        let core_state: CoreState = get_core_state(&env);

        deposit_collateral(&env, &core_state, &caller, &collateral_amount);

        set_user_vault(&env, &caller, &denomination, &new_vault);

        withdraw_stablecoin(&env, &core_state, &currency, &caller, &initial_debt);

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        protocol_stats.tot_vaults = protocol_stats.tot_vaults + 1;
        protocol_stats.tot_debt = protocol_stats.tot_debt + initial_debt;
        protocol_stats.tot_col = protocol_stats.tot_col + collateral_amount;

        update_protocol_stats(&env, protocol_stats);
    }

    fn get_vault(env: Env, caller: Address, denomination: Symbol) -> UserVault {
        validate_user_vault(&env, caller.clone(), denomination);
        get_user_vault(&env, caller.clone(), denomination)
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

        let mut user_vault: UserVault = get_user_vault(&env, caller.clone(), denomination);

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        user_vault.total_col = user_vault.total_col + collateral_amount;
        protocol_stats.tot_col = protocol_stats.tot_col + collateral_amount;

        set_user_vault(&env, &caller, &denomination, &user_vault);
        update_protocol_stats(&env, protocol_stats);
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

        let mut user_vault: UserVault = get_user_vault(&env, caller.clone(), denomination);

        let protocol_state: ProtocolState = get_protocol_state(&env);

        let new_debt_amount: i128 = user_vault.total_debt + debt_amount;

        let collateral_value: i128 = currency.rate * user_vault.total_col;

        let deposit_rate: i128 = div_floor(collateral_value, new_debt_amount);

        if deposit_rate < protocol_state.op_col_rte {
            panic_with_error!(&env, SCErrors::CollateralRateUnderMinimum);
        }

        withdraw_stablecoin(&env, &core_state, &currency, &caller, &debt_amount);

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        user_vault.total_debt = new_debt_amount;
        protocol_stats.tot_debt = protocol_stats.tot_debt + debt_amount;

        set_user_vault(&env, &caller, &denomination, &user_vault);
        update_protocol_stats(&env, protocol_stats);
    }

    fn pay_debt(env: Env, caller: Address, deposit_amount: i128, denomination: Symbol) {
        caller.require_auth();

        validate_currency(&env, denomination);
        is_currency_active(&env, denomination);
        check_positive(&env, &deposit_amount);
        validate_user_vault(&env, caller.clone(), denomination);

        // TODO: Add fee logic

        let currency: Currency = get_currency(&env, denomination);

        let mut user_vault: UserVault = get_user_vault(&env, caller.clone(), denomination);

        if deposit_amount > user_vault.total_debt {
            panic_with_error!(&env, SCErrors::DepositAmountIsMoreThanTotalDebt);
        }

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        deposit_stablecoin(&env, &currency, &caller, &deposit_amount);

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        if user_vault.total_debt == deposit_amount {
            // If the amount is equal to the debt it means it is paid in full so we release the collateral and remove the vault
            protocol_stats.tot_vaults = protocol_stats.tot_vaults - 1;
            protocol_stats.tot_col = protocol_stats.tot_col - user_vault.total_col;

            token::Client::new(&env, &core_state.colla_tokn).xfer(
                &env.current_contract_address(),
                &caller,
                &user_vault.total_col,
            );

            remove_user_vault(&env, &caller, &denomination);
        } else {
            // If amount is not enough to pay all the debt, we just updated the stats of the user's vault
            user_vault.total_debt = user_vault.total_debt - deposit_amount;
            set_user_vault(&env, &caller, &denomination, &user_vault);
        }

        protocol_stats.tot_debt = protocol_stats.tot_debt - deposit_amount;

        update_protocol_stats(&env, protocol_stats);
    }
}
