use crate::storage_types::*;
use crate::token;
use crate::utils::*;

use soroban_sdk::{contractimpl, panic_with_error, Address, BytesN, Env};

// TODO: Explain each function here
pub trait VaultsContractTrait {
    fn s_c_state(
        env: Env,
        admin: Address,
        colla_tokn: BytesN<32>,
        nativ_tokn: BytesN<32>,
        stble_tokn: BytesN<32>,
    );

    fn g_c_state(env: Env) -> CoreState;

    fn g_p_state(env: Env) -> ProtocolState;
    fn s_p_state(env: Env, caller: Address, mn_col_rte: u128, mn_v_c_amt: u128, op_col_rte: u128);

    fn g_p_c_prce(env: Env) -> ProtocolCollateralPrice;
    fn s_p_c_prce(env: Env, caller: Address, rate: u128);

    fn g_p_stats(env: Env) -> ProtStats;

    fn new_vault(env: Env, caller: Address, initial_debt: u128, collateral_amount: u128);

    fn pay_debt(env: Env, caller: Address, amount: u128);

    fn incr_col(env: Env, caller: Address, amount: u128);
    fn incr_debt(env: Env, caller: Address, debt_amount: u128);
}

pub struct VaultsContract;

#[contractimpl]
impl VaultsContractTrait for VaultsContract {
    fn s_c_state(
        env: Env,
        admin: Address,
        colla_tokn: BytesN<32>,
        nativ_tokn: BytesN<32>,
        stble_tokn: BytesN<32>,
    ) {
        if env.storage().has(&DataKeys::CoreState) {
            panic_with_error!(&env, SCErrors::AlreadyInit);
        }

        let core_state = CoreState {
            admin,
            colla_tokn,
            nativ_tokn,
            stble_tokn,
        };

        env.storage().set(&DataKeys::CoreState, &core_state);
    }

    fn g_c_state(env: Env) -> CoreState {
        get_core_state(&env)
    }

    fn g_p_state(env: Env) -> ProtocolState {
        get_protocol_state(&env)
    }

    fn s_p_state(env: Env, caller: Address, mn_col_rte: u128, mn_v_c_amt: u128, op_col_rte: u128) {
        caller.require_auth();
        check_admin(&env, caller);

        env.storage().set(
            &DataKeys::ProtState,
            &ProtocolState {
                mn_col_rte,
                mn_v_c_amt,
                op_col_rte,
            },
        );
    }

    fn g_p_c_prce(env: Env) -> ProtocolCollateralPrice {
        get_protocol_collateral_price(&env)
    }

    fn s_p_c_prce(env: Env, caller: Address, price: u128) {
        // TODO: this method should be updated in the future once there are oracles in the network
        caller.require_auth();
        check_admin(&env, caller);

        let mut protocol_collateral_price: ProtocolCollateralPrice = env
            .storage()
            .get(&DataKeys::ProtRate)
            .unwrap_or(Ok(ProtocolCollateralPrice {
                last_updte: env.ledger().timestamp(),
                current: 1,
            }))
            .unwrap();

        if price != protocol_collateral_price.current {
            protocol_collateral_price.current = price;
            protocol_collateral_price.last_updte = env.ledger().timestamp();
            env.storage()
                .set(&DataKeys::ProtRate, &protocol_collateral_price);
        } else {
            // TODO: if the last time the rate was changed was more than 15 minutes ago shut down the issuance of new debt
        }
    }

    fn g_p_stats(env: Env) -> ProtStats {
        get_protocol_stats(&env)
    }

    fn new_vault(env: Env, caller: Address, initial_debt: u128, collateral_amount: u128) {
        caller.require_auth();

        let key = DataKeys::UserVault(caller.clone());

        if env.storage().has(&key) {
            panic_with_error!(&env, SCErrors::UserAlreadyHasVault);
        }

        // TODO: check if we are in panic mode once is implemented
        // TODO: check if collateral price has been updated lately
        // TODO: Add fee logic

        let protocol_state: ProtocolState = get_protocol_state(&env);

        valid_initial_debt(&env, &protocol_state, initial_debt);

        let protocol_collateral_price: ProtocolCollateralPrice =
            get_protocol_collateral_price(&env);

        let collateral_value: u128 = protocol_collateral_price.current * collateral_amount;

        let deposit_rate: u128 = collateral_value / initial_debt;

        if deposit_rate < protocol_state.mn_col_rte {
            panic_with_error!(&env, SCErrors::InvalidOpeningCollateralRatio);
        }

        let new_vault = UserVault {
            id: caller.clone(),
            total_col: collateral_amount,
            total_debt: initial_debt,
        };

        let core_state: CoreState = get_core_state(&env);

        deposit_collateral(
            &env,
            core_state.colla_tokn.clone(),
            &caller,
            collateral_amount.clone(),
        );

        env.storage().set(&key, &new_vault);

        withdraw_stablecoin(&env, core_state.stble_tokn.clone(), &caller, initial_debt);

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        protocol_stats.tot_vaults = protocol_stats.tot_vaults + 1;
        protocol_stats.tot_debt = protocol_stats.tot_debt + initial_debt;
        protocol_stats.tot_col = protocol_stats.tot_col + collateral_amount;

        update_protocol_stats(&env, protocol_stats);
    }

    fn pay_debt(env: Env, caller: Address, deposit_amount: u128) {
        caller.require_auth();

        let key = DataKeys::UserVault(caller.clone());

        if !env.storage().has(&key) {
            panic_with_error!(&env, SCErrors::UserDoesntHaveAVault);
        }

        // TODO: Add fee logic

        let mut user_vault: UserVault = env.storage().get(&key).unwrap().unwrap();

        if deposit_amount > user_vault.total_debt {
            panic_with_error!(&env, SCErrors::DepositAmountIsMoreThanTotalDebt);
        }

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        token::Client::new(&env, &core_state.stble_tokn).xfer(
            &caller,
            &env.current_contract_address(),
            &(deposit_amount as i128),
        );

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        if user_vault.total_debt == deposit_amount {
            // If the amount is equal to the debt it means it is paid in full so we release the collateral and remove the vault
            protocol_stats.tot_vaults = protocol_stats.tot_vaults - 1;
            protocol_stats.tot_col = protocol_stats.tot_col - user_vault.total_col;

            token::Client::new(&env, &core_state.colla_tokn).xfer(
                &env.current_contract_address(),
                &caller,
                &(user_vault.total_col as i128),
            );

            env.storage().remove(&key);
        } else {
            // If amount is not enough to pay all the debt, we just updated the stats of the user's vault
            user_vault.total_debt = user_vault.total_debt - deposit_amount;
            env.storage().set(&key, &user_vault);
        }

        protocol_stats.tot_debt = protocol_stats.tot_debt - deposit_amount;

        update_protocol_stats(&env, protocol_stats);
    }

    fn incr_col(env: Env, caller: Address, collateral_amount: u128) {
        caller.require_auth();

        let key = DataKeys::UserVault(caller.clone());

        if !env.storage().has(&key) {
            panic_with_error!(&env, SCErrors::UserDoesntHaveAVault);
        }

        // TODO: Add fee logic

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        token::Client::new(&env, &core_state.colla_tokn).xfer(
            &caller,
            &env.current_contract_address(),
            &(collateral_amount as i128),
        );

        let mut user_vault: UserVault = env.storage().get(&key).unwrap().unwrap();

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        user_vault.total_col = user_vault.total_col + collateral_amount;
        protocol_stats.tot_col = protocol_stats.tot_col + collateral_amount;

        env.storage().set(&key, &user_vault);
        update_protocol_stats(&env, protocol_stats);
    }

    fn incr_debt(env: Env, caller: Address, debt_amount: u128) {
        caller.require_auth();

        let key = DataKeys::UserVault(caller.clone());

        if !env.storage().has(&key) {
            panic_with_error!(&env, SCErrors::UserDoesntHaveAVault);
        }

        // TODO: Add fee logic
        // TODO: check if we are in panic mode once is implemented
        // TODO: check if collateral price has been updated lately

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        let protocol_collateral_price: ProtocolCollateralPrice =
            get_protocol_collateral_price(&env);

        let mut user_vault: UserVault = env.storage().get(&key).unwrap().unwrap();

        let protocol_state: ProtocolState = get_protocol_state(&env);

        let new_debt_amount: u128 = user_vault.total_debt + debt_amount;

        let collateral_value: u128 = protocol_collateral_price.current * user_vault.total_col;

        let deposit_rate: u128 = collateral_value / new_debt_amount;

        if deposit_rate < protocol_state.op_col_rte {
            panic_with_error!(&env, SCErrors::CollateralRateUnderMinimun);
        }

        token::Client::new(&env, &core_state.stble_tokn).xfer(
            &env.current_contract_address(),
            &caller,
            &(debt_amount as i128),
        );

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        user_vault.total_debt = new_debt_amount;
        protocol_stats.tot_debt = protocol_stats.tot_debt + debt_amount;

        env.storage().set(&key, &user_vault);
        update_protocol_stats(&env, protocol_stats);
    }
}
