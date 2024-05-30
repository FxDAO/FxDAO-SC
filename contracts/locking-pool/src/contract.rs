use crate::errors::ContractErrors;
use crate::storage::core::{CoreDataKeys, CoreStorageFunc};
use crate::storage::deposits::{Deposit, DepositsStorageFunc};
use crate::storage::pools::{Pool, PoolsDataFunc};
use crate::utils::core::validate;
use log::error;
use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, BytesN, Env};

pub trait LockingPoolContractTrait {
    fn upgrade(e: Env, hash: BytesN<32>);
    fn set_admin(e: Env, address: Address);
    fn set_manager(e: Env, address: Address);
    fn add_pool(e: Env, deposit_asset: Address, lock_period: u64, min_deposit: u128);
    fn toggle_pool(e: Env, deposit_asset: Address, status: bool);
    fn deposit(e: Env, deposit_asset: Address, caller: Address, amount: u128);
    fn withdraw(e: Env, deposit_asset: Address, caller: Address);
    fn distribute(e: Env, deposit_asset: Address, amount: u128);
}

#[contract]
pub struct LockingPoolContract;

#[contractimpl]
impl LockingPoolContractTrait for LockingPoolContract {
    fn upgrade(e: Env, hash: BytesN<32>) {
        validate(&e, CoreDataKeys::Manager);
        e.deployer().update_current_contract_wasm(hash);
        e._core().bump();
    }

    fn set_admin(e: Env, address: Address) {
        if let Some(v) = e._core().address(&CoreDataKeys::Admin) {
            v.require_auth();
        }

        e._core().set_address(&CoreDataKeys::Admin, &address);
        e._core().bump();
    }

    fn set_manager(e: Env, address: Address) {
        if let Some(v) = e._core().address(&CoreDataKeys::Manager) {
            v.require_auth();
        }

        e._core().set_address(&CoreDataKeys::Manager, &address);
        e._core().bump();
    }

    fn add_pool(e: Env, deposit_asset: Address, lock_period: u64, min_deposit: u128) {
        validate(&e, CoreDataKeys::Manager);

        let new_pool: Pool = Pool {
            active: false,
            asset: deposit_asset,
            balance: 0,
            deposits: 0,
            factor: 0,
            lock_period,
            min_deposit,
        };

        e._pools().set_pool(&new_pool);
        e._pools().bump_pool(&new_pool.asset);
        e._core().bump();
    }

    fn toggle_pool(e: Env, deposit_asset: Address, status: bool) {
        validate(&e, CoreDataKeys::Admin);

        let mut pool: Pool = e._pools().pool(&deposit_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        pool.active = status;

        e._pools().set_pool(&pool);
        e._pools().bump_pool(&pool.asset);
        e._core().bump();
    }

    fn deposit(e: Env, deposit_asset: Address, caller: Address, amount: u128) {
        caller.require_auth();

        let mut pool: Pool = e._pools().pool(&deposit_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        if !pool.active {
            panic_with_error!(&e, &ContractErrors::PoolDoesntAcceptDeposits);
        }

        if amount < pool.min_deposit {
            panic_with_error!(&e, &ContractErrors::InvalidDepositAmount);
        }

        if e._deposits().get(&deposit_asset, &caller).is_some() {
            panic_with_error!(&e, &ContractErrors::DepositAlreadyExists);
        }

        let result = token::Client::new(&e, &pool.asset).try_transfer(
            &caller,
            &e.current_contract_address(),
            &(amount.clone() as i128),
        );

        if result.is_err() {
            panic_with_error!(&e, &ContractErrors::FundsDepositFailed);
        }

        pool.deposits += 1;
        pool.balance += amount;

        let deposit: Deposit = Deposit {
            amount,
            snapshot: pool.factor,
            unlocks_at: e.ledger().timestamp() + pool.lock_period,
        };

        e._deposits().set(&deposit_asset, &caller, &deposit);
        e._deposits().bump(&deposit_asset, &caller);

        e._pools().set_pool(&pool);
        e._pools().bump_pool(&pool.asset);
        e._core().bump();
    }

    fn withdraw(e: Env, deposit_asset: Address, caller: Address) {
        caller.require_auth();

        let mut pool: Pool = e._pools().pool(&deposit_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        let deposit: Deposit = e
            ._deposits()
            .get(&deposit_asset, &caller)
            .unwrap_or_else(|| {
                panic_with_error!(&e, &ContractErrors::DepositDoesntExist);
            });

        if e.ledger().timestamp() < deposit.unlocks_at {
            panic_with_error!(&e, &ContractErrors::DepositIsStillLocked);
        }

        e._deposits().remove(&deposit_asset, &caller);

        let reward: u128 = (deposit.amount * (pool.factor - deposit.snapshot)) / 1_0000000;

        pool.deposits -= 1;
        pool.balance -= deposit.amount;

        if pool.deposits == 0 && pool.balance == 0 {
            pool.factor = 0;
        }

        if reward > 0 {
            let result =
                token::Client::new(&e, &e._core().address(&CoreDataKeys::RewardsAsset).unwrap())
                    .try_transfer(&e.current_contract_address(), &caller, &(reward as i128));

            if result.is_err() {
                panic_with_error!(&e, &ContractErrors::RewardsWithdrawFailed);
            }
        }

        let result = token::Client::new(&e, &pool.asset).try_transfer(
            &e.current_contract_address(),
            &caller,
            &(deposit.amount as i128),
        );

        if result.is_err() {
            panic_with_error!(&e, &ContractErrors::FundsWithdrawFailed);
        }

        e._pools().set_pool(&pool);
        e._pools().bump_pool(&pool.asset);
        e._core().bump();
    }

    fn distribute(e: Env, deposit_asset: Address, amount: u128) {
        validate(&e, CoreDataKeys::Manager);

        let mut pool: Pool = e._pools().pool(&deposit_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        if pool.balance == 0 {
            panic_with_error!(&e, &ContractErrors::CantDistributeReward);
        }

        let result =
            token::Client::new(&e, &e._core().address(&CoreDataKeys::RewardsAsset).unwrap())
                .try_transfer(
                    &e._core().address(&CoreDataKeys::Manager).unwrap(),
                    &e.current_contract_address(),
                    &(amount as i128),
                );

        if result.is_err() {
            panic_with_error!(&e, &ContractErrors::RewardsDepositFailed);
        }

        pool.factor += (amount * 1_0000000) / pool.balance;
        e._pools().set_pool(&pool);
        e._pools().bump_pool(&pool.asset);
        e._core().bump();
    }
}
