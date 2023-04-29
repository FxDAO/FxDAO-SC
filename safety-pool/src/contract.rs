use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::storage::deposits::Deposit;
use crate::utils::core::{can_init_contract, get_core_state, set_admin, set_core_state};
use crate::utils::deposits::{
    get_deposit, get_depositors, is_depositor_listed, make_deposit, make_withdrawal,
    remove_deposit, remove_depositor_from_depositors, save_deposit, save_depositors,
};
use soroban_sdk::{contractimpl, panic_with_error, Address, BytesN, Env, Vec};

pub trait SafetyPoolContractTrait {
    fn init(
        env: Env,
        contract_admin: Address,
        vaults_contract: Address,
        deposit_asset: BytesN<32>,
        min_deposit: u128,
    );

    fn deposit(env: Env, caller: Address, amount: u128);

    fn get_deposit(env: Env, caller: Address) -> Deposit;

    fn get_depositors(env: Env) -> Vec<Address>;

    fn withdraw(env: Env, caller: Address);
}

pub struct SafetyPoolContract;

// TODO: Add events for each function
#[contractimpl]
impl SafetyPoolContractTrait for SafetyPoolContract {
    fn init(
        env: Env,
        contract_admin: Address,
        vaults_contract: Address,
        deposit_asset: BytesN<32>,
        min_deposit: u128,
    ) {
        can_init_contract(&env);
        set_admin(&env, &contract_admin);
        set_core_state(
            &env,
            &CoreState {
                deposit_asset,
                vaults_contract,
                min_deposit,
            },
        )
    }

    fn deposit(env: Env, caller: Address, amount: u128) {
        caller.require_auth();

        let core_state: CoreState = get_core_state(&env);

        if amount < core_state.min_deposit {
            panic_with_error!(&env, SCErrors::BelowMinDeposit);
        }

        make_deposit(&env, &core_state.deposit_asset, &caller, &amount);

        let mut deposit: Deposit = get_deposit(&env, &caller);
        deposit.amount += amount;
        save_deposit(&env, &deposit);

        let mut depositors: Vec<Address> = get_depositors(&env);
        if !is_depositor_listed(&depositors, &caller) {
            depositors.push_back(caller);
            save_depositors(&env, &depositors)
        }
    }

    fn get_deposit(env: Env, caller: Address) -> Deposit {
        caller.require_auth();
        get_deposit(&env, &caller)
    }

    fn get_depositors(env: Env) -> Vec<Address> {
        get_depositors(&env)
    }

    fn withdraw(env: Env, caller: Address) {
        caller.require_auth();

        let deposit: Deposit = get_deposit(&env, &caller);
        if deposit.amount == 0 {
            panic_with_error!(&env, SCErrors::NothingToWithdraw);
        }

        let core_state: CoreState = get_core_state(&env);

        make_withdrawal(&env, &core_state.deposit_asset, &deposit);
        remove_deposit(&env, &caller);

        let mut depositors: Vec<Address> = get_depositors(&env);
        depositors = remove_depositor_from_depositors(&depositors, &caller);
        save_depositors(&env, &depositors);
    }
}
