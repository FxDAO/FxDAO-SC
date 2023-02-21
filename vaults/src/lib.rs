#![no_std]

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

use soroban_sdk::{
    contracterror, contractimpl, contracttype, panic_with_error, Address, BytesN, Env,
};

#[contracttype]
pub struct CoreState {
    admin: Address,
    colla_tokn: BytesN<32>,
    nativ_tokn: BytesN<32>,
    stble_tokn: BytesN<32>,
}

#[contracttype]
pub struct ProtocolState {
    // Min collateral ratio - ex: 1.10
    mn_col_rte: i128,

    // Min vault creation amount - ex: 5000
    mn_v_c_amt: i128,

    // Opening collateral ratio - ex: 1.15
    op_col_rte: i128,
}

#[contracttype]
pub struct ProtocolCollateralPrice {
    // This is the last time the price got updated
    last_updte: u64,

    // This is the current price of the collateral in our protocol
    current: i128,
}

#[contracttype]
pub struct ProtStats {
    tot_vaults: i64,
    tot_debt: i128,
    tot_col: i128,
}

#[contracttype]
pub struct UserVault {
    id: Address,
    total_debt: i128,
    total_col: i128,
}

#[contracttype]
pub enum DataKeys {
    CoreState,
    ProtState,
    ProtRate,
    ProtStats,
    UserVault(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCErrors {
    AlreadyInit = 0,
    Unauthorized = 1,
    UserAlreadyHasVault = 2,
    InvalidInitialDebtAmount = 3,
    InvalidOpeningCollateralRatio = 4,
    UserDoesntHaveAVault = 5,
    DepositAmountIsMoreThanTotalDebt = 6,
}

pub trait VaultsContractTrait {
    fn s_c_state(
        env: Env,
        admin: Address,
        colla_tokn: BytesN<32>,
        nativ_tokn: BytesN<32>,
        stble_tokn: BytesN<32>,
    ) {
    }

    fn g_c_state(env: Env) -> CoreState;

    fn g_p_state(env: Env) -> ProtocolState;
    fn s_p_state(env: Env, caller: Address, mn_col_rte: i128, mn_v_c_amt: i128, op_col_rte: i128);

    fn g_p_c_prce(env: Env) -> ProtocolCollateralPrice;
    fn s_p_c_prce(env: Env, caller: Address, rate: i128);

    fn g_p_stats(env: Env) -> ProtStats;

    fn new_vault(env: Env, caller: Address, initial_debt: i128, collateral_amount: i128);

    fn pay_debt(env: Env, caller: Address, amount: i128);
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

    fn s_p_state(env: Env, caller: Address, mn_col_rte: i128, mn_v_c_amt: i128, op_col_rte: i128) {
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

    fn s_p_c_prce(env: Env, caller: Address, price: i128) {
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
            // TODO: if there last time the rate was changed was more than 15 minutes ago shut down the issuance of new debt
        }
    }

    fn g_p_stats(env: Env) -> ProtStats {
        get_protocol_stats(&env)
    }

    fn new_vault(env: Env, caller: Address, initial_debt: i128, collateral_amount: i128) {
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

        let collateral_value: i128 = protocol_collateral_price.current * collateral_amount;

        let deposit_rate: i128 = collateral_value / initial_debt;

        if deposit_rate < protocol_state.op_col_rte {
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

    fn pay_debt(env: Env, caller: Address, amount: i128) {
        caller.require_auth();

        let key = DataKeys::UserVault(caller.clone());

        if !env.storage().has(&key) {
            panic_with_error!(&env, SCErrors::UserDoesntHaveAVault);
        }

        // TODO: Add fee logic

        let mut user_vault: UserVault = env.storage().get(&key).unwrap().unwrap();

        if amount > user_vault.total_debt {
            panic_with_error!(&env, SCErrors::DepositAmountIsMoreThanTotalDebt);
        }

        let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

        token::Client::new(&env, &core_state.stble_tokn).xfer(
            &caller,
            &env.current_contract_address(),
            &amount,
        );

        let mut protocol_stats: ProtStats = get_protocol_stats(&env);

        if user_vault.total_debt == amount {
            // If the amount is equal to the debt it means it is paid in full so we release the collateral and remove the vault
            protocol_stats.tot_vaults = protocol_stats.tot_vaults - 1;
            protocol_stats.tot_col = protocol_stats.tot_col - user_vault.total_col;

            token::Client::new(&env, &core_state.colla_tokn).xfer(
                &env.current_contract_address(),
                &caller,
                &user_vault.total_col,
            );

            env.storage().remove(&key);
        } else {
            // If amount is not enough to pay all the debt, we just updated the stats of the user's vault
            user_vault.total_debt = user_vault.total_debt - amount;
            env.storage().set(&key, &user_vault);
        }

        protocol_stats.tot_debt = protocol_stats.tot_debt - amount;

        update_protocol_stats(&env, protocol_stats);
    }
}

fn check_admin(env: &Env, caller: Address) {
    let core_state: CoreState = env.storage().get(&DataKeys::CoreState).unwrap().unwrap();

    if core_state.admin != caller {
        panic_with_error!(&env, SCErrors::Unauthorized);
    }
}

fn get_core_state(env: &Env) -> CoreState {
    env.storage().get(&DataKeys::CoreState).unwrap().unwrap()
}

fn get_protocol_state(env: &Env) -> ProtocolState {
    env.storage().get(&DataKeys::ProtState).unwrap().unwrap()
}

fn get_protocol_collateral_price(env: &Env) -> ProtocolCollateralPrice {
    env.storage()
        .get(&DataKeys::ProtRate)
        .unwrap_or(Ok(ProtocolCollateralPrice {
            last_updte: env.ledger().timestamp(),
            current: 0,
        }))
        .unwrap()
}

fn valid_initial_debt(env: &Env, state: &ProtocolState, initial_debt: i128) {
    if state.mn_v_c_amt > initial_debt {
        panic_with_error!(env, SCErrors::InvalidInitialDebtAmount);
    }
}

// TODO: consider remove both deposit_collateral and withdraw_stablecoin
fn deposit_collateral(
    env: &Env,
    collateral_token: BytesN<32>,
    depositor: &Address,
    collateral_amount: i128,
) {
    token::Client::new(&env, &collateral_token).xfer(
        &depositor,
        &env.current_contract_address(),
        &collateral_amount,
    );
}

fn withdraw_stablecoin(
    env: &Env,
    contract: BytesN<32>,
    recipient: &Address,
    stablecoin_amount: i128,
) {
    token::Client::new(&env, &contract).xfer(
        &env.current_contract_address(),
        &recipient,
        &stablecoin_amount,
    );
}

fn get_protocol_stats(env: &Env) -> ProtStats {
    env.storage()
        .get(&DataKeys::ProtStats)
        .unwrap_or(Ok(ProtStats {
            tot_vaults: 0,
            tot_debt: 0,
            tot_col: 0,
        }))
        .unwrap()
}

fn update_protocol_stats(env: &Env, stats: ProtStats) {
    env.storage().set(&DataKeys::ProtStats, &stats);
}

mod test;
