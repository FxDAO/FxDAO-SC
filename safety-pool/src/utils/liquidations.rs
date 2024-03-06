use crate::storage::core::CoreStats;
use crate::storage::deposits::Deposit;
use crate::storage::liquidations::{Liquidation, LiquidationsDataKeys};
use crate::utils::deposits::make_withdrawal;
use num_integer::div_floor;
use soroban_sdk::{Address, Env};

pub const DAY_IN_LEDGERS: u32 = 17280;
pub const PERSISTENT_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 30;
pub const PERSISTENT_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 20;

pub fn bump_liquidation(env: &Env, index: u64) {
    env.storage().persistent().extend_ttl(
        &LiquidationsDataKeys::Liquidation(index),
        PERSISTENT_BUMP_CONSTANT_THRESHOLD,
        PERSISTENT_BUMP_CONSTANT,
    );
}

pub fn get_liquidation(env: &Env, index: u64) -> Liquidation {
    env.storage()
        .persistent()
        .get(&LiquidationsDataKeys::Liquidation(index))
        .unwrap()
}

pub fn check_liquidation_exist(env: &Env, index: u64) -> bool {
    env.storage()
        .persistent()
        .has(&LiquidationsDataKeys::Liquidation(index))
}

pub fn set_liquidation(env: &Env, liquidation: &Liquidation) {
    env.storage().persistent().set(
        &LiquidationsDataKeys::Liquidation(liquidation.index.clone()),
        liquidation,
    );
}

pub fn remove_liquidation(env: &Env, index: u64) {
    env.storage()
        .persistent()
        .remove(&LiquidationsDataKeys::Liquidation(index));
}

pub fn withdraw_collateral(
    env: &Env,
    core_stats: &CoreStats,
    collateral_asset: &Address,
    deposit: &Deposit,
) {
    let mut col_to_withdraw: u128 = 0;
    let mut total_debt_covered: u128 = 0;
    let mut index: u64 = deposit.liquidation_index.clone();
    let mut finished: bool = false;

    while !finished {
        if !check_liquidation_exist(&env, index.clone()) {
            finished = true;
            break;
        }

        let mut liquidation: Liquidation = get_liquidation(&env, index.clone());
        let shares_left: u128 = liquidation.total_shares - liquidation.shares_redeemed;

        let percentage_owned: u128 = div_floor(deposit.shares * 1_0000000, shares_left);

        let share_of_col_liquidated: u128 =
            div_floor(liquidation.col_to_withdraw * percentage_owned, 1_0000000);

        let share_of_debt_paid: u128 =
            div_floor(liquidation.total_debt_paid * percentage_owned, 1_0000000);

        liquidation.shares_redeemed += deposit.shares;
        set_liquidation(&env, &liquidation);

        col_to_withdraw += share_of_col_liquidated;
        total_debt_covered += share_of_debt_paid;
        index += 1;

        if total_debt_covered >= deposit.amount || (deposit.amount - total_debt_covered) < 1_0000000
        {
            finished = true;
        }
    }

    make_withdrawal(
        &env,
        &collateral_asset,
        &deposit.depositor,
        col_to_withdraw as i128,
    );
}
