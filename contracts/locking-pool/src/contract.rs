use crate::errors::ContractErrors;
use crate::storage::core::{CoreDataKeys, CoreStorageFunc};
use crate::storage::deposits::{Deposit, DepositsStorageFunc};
use crate::storage::pools::{Pool, PoolsDataFunc};
use crate::utils::core::validate;
use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, BytesN, Env, Vec};

pub trait LockingPoolContractTrait {
    fn init(e: Env, admin: Address, manager: Address, reward_asset: Address);
    fn upgrade(e: Env, hash: BytesN<32>);
    fn set_admin(e: Env, address: Address);
    fn set_manager(e: Env, address: Address);
    fn set_pool(e: Env, deposit_asset: Address, lock_period: u64, min_deposit: u128);
    fn clone_pool(e: Env, existing_asset: Address, new_asset: Address);
    fn toggle_pool(e: Env, deposit_asset: Address, status: bool);
    fn remove_pool(e: Env, deposit_asset: Address);
    fn migrate_deposits(e: Env, old_asset: Address, new_asset: Address, depositors: Vec<Address>);
    fn deposit(e: Env, deposit_asset: Address, caller: Address, amount: u128);
    fn withdraw(e: Env, deposit_asset: Address, caller: Address);
    fn distribute(e: Env, caller: Address, deposit_asset: Address, amount: u128);
}

#[contract]
pub struct LockingPoolContract;

#[contractimpl]
impl LockingPoolContractTrait for LockingPoolContract {
    fn init(e: Env, admin: Address, manager: Address, reward_asset: Address) {
        if e._core().address(&CoreDataKeys::Admin).is_some() {
            panic_with_error!(&e, &ContractErrors::AlreadyStarted);
        }

        e._core().set_address(&CoreDataKeys::Admin, &admin);
        e._core().set_address(&CoreDataKeys::Manager, &manager);
        e._core()
            .set_address(&CoreDataKeys::RewardsAsset, &reward_asset);

        e._core().bump();
    }

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

    fn set_pool(e: Env, deposit_asset: Address, lock_period: u64, min_deposit: u128) {
        validate(&e, CoreDataKeys::Manager);

        let new_pool: Pool;
        if let Some(pool) = e._pools().pool(&deposit_asset) {
            new_pool = Pool {
                active: pool.active,
                asset: pool.asset,
                balance: pool.balance,
                deposits: pool.deposits,
                factor: pool.factor,
                lock_period,
                min_deposit,
            };
        } else {
            new_pool = Pool {
                active: false,
                asset: deposit_asset,
                balance: 0,
                deposits: 0,
                factor: 0,
                lock_period,
                min_deposit,
            };
        }

        e._pools().set_pool(&new_pool);
        e._pools().bump_pool(&new_pool.asset);
        e._core().bump();
    }

    fn clone_pool(e: Env, existing_asset: Address, new_asset: Address) {
        validate(&e, CoreDataKeys::Manager);
        let existing_pool: Pool = e._pools().pool(&existing_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        let new_pool: Pool = Pool {
            active: false,
            asset: new_asset,
            balance: existing_pool.balance,
            deposits: existing_pool.deposits,
            factor: existing_pool.factor,
            lock_period: existing_pool.lock_period,
            min_deposit: existing_pool.min_deposit,
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

    fn remove_pool(e: Env, deposit_asset: Address) {
        validate(&e, CoreDataKeys::Manager);

        let pool: Pool = e._pools().pool(&deposit_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        if pool.deposits > 0 {
            panic_with_error!(&e, &ContractErrors::PoolCanNotBeDeleted);
        }

        e._pools().remove_pool(&deposit_asset);

        e._core().bump();
    }

    fn migrate_deposits(e: Env, old_asset: Address, new_asset: Address, depositors: Vec<Address>) {
        validate(&e, CoreDataKeys::Manager);

        e._pools().pool(&old_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        e._pools().pool(&new_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        for depositor in depositors.iter() {
            let old_deposit: Deposit =
                e._deposits()
                    .get(&old_asset, &depositor)
                    .unwrap_or_else(|| {
                        panic_with_error!(&e, &ContractErrors::DepositDoesntExist);
                    });

            e._deposits().set(&new_asset, &depositor, &old_deposit);
            e._deposits().bump(&new_asset, &depositor);
            e._deposits().remove(&old_asset, &depositor);
        }
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

    fn distribute(e: Env, caller: Address, deposit_asset: Address, amount: u128) {
        caller.require_auth();

        if amount < 100_0000000 {
            panic_with_error!(&e, &ContractErrors::CantDistributeReward);
        }

        let mut pool: Pool = e._pools().pool(&deposit_asset).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::PoolDoesntExist);
        });

        if pool.balance == 0 {
            panic_with_error!(&e, &ContractErrors::CantDistributeReward);
        }

        let result =
            token::Client::new(&e, &e._core().address(&CoreDataKeys::RewardsAsset).unwrap())
                .try_transfer(&caller, &e.current_contract_address(), &(amount as i128));

        if result.is_err() {
            panic_with_error!(&e, &ContractErrors::RewardsDepositFailed);
        }

        pool.factor += (amount * 1_0000000) / pool.balance;
        e._pools().set_pool(&pool);
        e._pools().bump_pool(&pool.asset);
        e._core().bump();
    }
}
