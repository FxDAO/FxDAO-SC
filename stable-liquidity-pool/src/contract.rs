use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStorageFunc, LockingState};
use crate::storage::deposits::{Deposit, DepositsStorageFunc};
use crate::utils::deposits::{make_deposit, make_withdrawal, validate_deposit_asset};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, Address, BytesN, Env, Map, Vec,
};

pub trait StableLiquidityPoolContractTrait {
    fn init(
        e: Env,
        admin: Address,
        manager: Address,
        governance_token: Address,
        accepted_assets: Vec<Address>,
        fee_percentage: u128,
        treasury: Address,
    );

    fn get_core_state(e: Env) -> CoreState;

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);

    fn deposit(e: Env, caller: Address, asset: Address, amount: u128);

    fn withdraw(e: Env, caller: Address, shares_to_redeem: u128, assets_orders: Map<Address, u128>);

    fn get_deposit(e: Env, caller: Address) -> Deposit;

    fn swap(e: Env, caller: Address, from_asset: Address, to_asset: Address, amount: u128);

    // Gov rewards fns
    fn lock(e: Env, caller: Address);
    fn unlock(e: Env, caller: Address);
    fn distribute(e: Env, caller: Address, amt: u128);
}

#[contract]
pub struct StableLiquidityPoolContract;

#[contractimpl]
impl StableLiquidityPoolContractTrait for StableLiquidityPoolContract {
    fn init(
        e: Env,
        admin: Address,
        manager: Address,
        governance_token: Address,
        accepted_assets: Vec<Address>,
        fee_percentage: u128,
        treasury: Address,
    ) {
        if e._core_state().is_some() {
            panic_with_error!(&e, SCErrors::ContractAlreadyInitiated);
        }
        e._set_core(&CoreState {
            admin,
            manager,
            governance_token,
            accepted_assets,
            fee_percentage,
            total_deposited: 0,
            share_price: 1_0000000,
            total_shares: 0,
            treasury,
        });
        e._set_locking_state(&LockingState {
            total: 0,
            factor: 0,
        });
        e._bump_instance();
    }

    fn get_core_state(e: Env) -> CoreState {
        e._bump_instance();
        e._core_state().unwrap()
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        e._bump_instance();
        e._core_state().unwrap().admin.require_auth();
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn deposit(e: Env, caller: Address, asset: Address, amount_deposit: u128) {
        e._bump_instance();
        caller.require_auth();
        let mut core_state: CoreState = e._core_state().unwrap();

        if amount_deposit < 1_0000000 {
            panic_with_error!(&e, &SCErrors::InvalidDepositAmount);
        }

        if !validate_deposit_asset(&core_state.accepted_assets, &asset) {
            panic_with_error!(&e, &SCErrors::InvalidAsset);
        }

        make_deposit(&e, &caller, &asset, &amount_deposit);

        let shares_to_issue: u128 = (amount_deposit * 1_0000000) / core_state.share_price;
        let mut deposit: Deposit = e._deposit(&caller).unwrap_or(Deposit {
            depositor: caller.clone(),
            locked: false,
            unlocks_at: 0,
            snapshot: 0,
            shares: 0,
        });

        if deposit.locked {
            panic_with_error!(&e, &SCErrors::LockedDeposit)
        }

        deposit.unlocks_at = e.ledger().timestamp() + (3600 * 48);
        deposit.shares = deposit.shares + shares_to_issue;
        e._set_deposit(&deposit);

        core_state.total_deposited = core_state.total_deposited + amount_deposit;
        core_state.total_shares = core_state.total_shares + shares_to_issue;
        e._set_core(&core_state);

        e._bump_deposit(&caller);
    }

    fn withdraw(
        e: Env,
        caller: Address,
        shares_to_redeem: u128,
        assets_orders: Map<Address, u128>,
    ) {
        e._bump_instance();
        caller.require_auth();
        let mut core_state: CoreState = e._core_state().unwrap();
        let calculated_amount_to_withdraw: u128 =
            (shares_to_redeem * core_state.total_deposited) / core_state.total_shares;

        let mut deposit: Deposit = e._deposit(&caller).unwrap_or(Deposit {
            depositor: caller.clone(),
            locked: false,
            unlocks_at: 0,
            snapshot: 0,
            shares: 0,
        });
        if deposit.shares == 0 {
            panic_with_error!(&e, &SCErrors::NothingToWithdraw);
        }

        if &deposit.shares < &shares_to_redeem {
            panic_with_error!(&e, &SCErrors::NotEnoughSharesToWithdraw);
        }

        if e.ledger().timestamp() < deposit.unlocks_at {
            panic_with_error!(&e, &SCErrors::LockedPeriodUncompleted);
        }

        if deposit.locked {
            panic_with_error!(&e, &SCErrors::LockedDeposit)
        }

        let mut withdraw_amount: u128 = 0;

        for token in core_state.accepted_assets.iter() {
            if assets_orders.contains_key(token.clone()) {
                withdraw_amount = withdraw_amount + assets_orders.get(token.clone()).unwrap();
            }
        }

        if calculated_amount_to_withdraw != withdraw_amount {
            panic_with_error!(&e, &SCErrors::InvalidWithdraw);
        }

        for (asset, amount) in assets_orders.iter() {
            if !validate_deposit_asset(&core_state.accepted_assets, &asset) {
                panic_with_error!(&e, &SCErrors::InvalidAsset);
            }

            if amount != 0 {
                make_withdrawal(&e, &deposit.depositor, &asset, &amount);
            }
        }

        if shares_to_redeem < deposit.shares {
            deposit.shares = deposit.shares - shares_to_redeem;
            e._set_deposit(&deposit);
            e._bump_deposit(&caller);
        } else {
            e._remove_deposit(&caller);
        }

        core_state.total_deposited = core_state.total_deposited - withdraw_amount;

        if core_state.total_deposited > 0 && core_state.total_deposited < 1_0000000 {
            panic_with_error!(&e, &SCErrors::InvalidWithdraw);
        }

        core_state.total_shares = core_state.total_shares - shares_to_redeem;
        if core_state.total_deposited == 0 && core_state.total_shares == 0 {
            core_state.share_price = 1_0000000;
        }
        e._set_core(&core_state);
    }

    fn get_deposit(e: Env, caller: Address) -> Deposit {
        e._bump_instance();
        if let Some(deposit) = e._deposit(&caller) {
            e._bump_deposit(&caller.clone());
            deposit
        } else {
            Deposit {
                depositor: caller.clone(),
                locked: false,
                unlocks_at: 0,
                snapshot: 0,
                shares: 0,
            }
        }
    }

    fn swap(e: Env, caller: Address, from_asset: Address, to_asset: Address, amount: u128) {
        e._bump_instance();
        caller.require_auth();

        let mut core_state: CoreState = e._core_state().unwrap();

        if !validate_deposit_asset(&core_state.accepted_assets, &from_asset) {
            panic_with_error!(&e, &SCErrors::InvalidAsset);
        }

        if !validate_deposit_asset(&core_state.accepted_assets, &to_asset) {
            panic_with_error!(&e, &SCErrors::InvalidAsset);
        }

        let fee: u128 = (amount * core_state.fee_percentage).div_ceil(1_0000000);
        let protocol_share: u128 = fee.div_ceil(2);
        let amount_to_exchange: u128 = amount - fee;

        make_deposit(&e, &caller, &from_asset, &amount);
        make_withdrawal(&e, &caller, &to_asset, &amount_to_exchange);

        token::Client::new(&e, &from_asset).transfer(
            &e.current_contract_address(),
            &core_state.treasury,
            &(protocol_share.clone() as i128),
        );

        let pool_profit: u128 = fee - protocol_share;
        let new_total_deposited: u128 = core_state.total_deposited + pool_profit;
        let new_share_price: u128 =
            (new_total_deposited * core_state.share_price) / core_state.total_deposited;

        core_state.share_price = new_share_price;
        core_state.total_deposited = new_total_deposited;

        e._set_core(&core_state);
    }

    fn lock(e: Env, caller: Address) {
        e._bump_instance();
        caller.require_auth();

        let mut deposit: Deposit = e._deposit(&caller).unwrap_or_else(|| {
            panic_with_error!(&e, &SCErrors::MissingDeposit);
        });
        if deposit.locked {
            panic_with_error!(&e, &SCErrors::AlreadyLocked);
        }

        deposit.locked = true;
        deposit.unlocks_at = e.ledger().timestamp() + (3600 * 24 * 7);

        let mut locking_state: LockingState = e._locking_state().unwrap();
        locking_state.total += deposit.shares;
        e._set_locking_state(&locking_state);

        deposit.snapshot = locking_state.factor;
        e._set_deposit(&deposit);
        e._bump_deposit(&caller.clone());
    }

    fn unlock(e: Env, caller: Address) {
        e._bump_instance();
        caller.require_auth();

        let mut deposit: Deposit = e._deposit(&caller).unwrap_or_else(|| {
            panic_with_error!(&e, &SCErrors::NotLockedDeposit);
        });

        if !deposit.locked {
            panic_with_error!(&e, &SCErrors::NotLockedDeposit)
        }

        if e.ledger().timestamp() < deposit.unlocks_at {
            panic_with_error!(&e, &SCErrors::LockedPeriodUncompleted);
        }

        let mut locking_state: LockingState = e._locking_state().unwrap();
        locking_state.total -= deposit.shares;
        e._set_locking_state(&locking_state);

        let reward: u128 = (deposit.shares * (locking_state.factor - deposit.snapshot)) / 1_0000000;

        deposit.locked = false;
        deposit.snapshot = 0;
        e._set_deposit(&deposit);

        make_withdrawal(
            &e,
            &caller,
            &e._core_state().unwrap().governance_token,
            &reward,
        );
        e._bump_deposit(&caller);
    }

    fn distribute(e: Env, caller: Address, amt: u128) {
        e._bump_instance();
        caller.require_auth();

        let core_state: CoreState = e._core_state().unwrap();
        let mut locking_state: LockingState = e._locking_state().unwrap();

        if locking_state.total == 0 {
            panic_with_error!(&e, &SCErrors::CantDistribute);
        }

        make_deposit(&e, &caller, &core_state.governance_token, &amt);

        locking_state.factor += (amt * 1_0000000) / locking_state.total;
        e._set_locking_state(&locking_state);
    }
}
