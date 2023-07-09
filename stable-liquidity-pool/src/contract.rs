use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::storage::deposits::Deposit;
use crate::utils::core::{
    can_init_contract, get_core_state, get_last_governance_token_distribution_time, set_core_state,
    set_last_governance_token_distribution_time,
};
use crate::utils::deposits::{
    calculate_depositor_withdrawal, get_deposit, get_depositors, is_depositor_listed, make_deposit,
    make_withdrawal, remove_deposit, remove_depositor_from_depositors, save_deposit,
    save_depositors, validate_deposit_asset,
};
use num_integer::div_floor;
use soroban_sdk::{contractimpl, panic_with_error, token, vec, Address, BytesN, Env, Symbol, Vec};

pub const CONTRACT_DESCRIPTION: Symbol = Symbol::short("StableLP");
pub const CONTRACT_VERSION: Symbol = Symbol::short("0_3_0");

pub trait StableLiquidityPoolTrait {
    fn init(
        env: Env,
        admin: Address,
        manager: Address,
        governance_token: Address,
        accepted_assets: Vec<Address>,
        fee_percentage: u128,
        treasury: Address,
    );

    fn get_core_state(env: Env) -> CoreState;

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);

    fn version(env: Env) -> (Symbol, Symbol);

    fn deposit(env: Env, caller: Address, asset: Address, amount: u128);

    fn withdraw(env: Env, caller: Address, asset: Address);

    fn get_deposit(env: Env, caller: Address) -> Deposit;

    fn swap(env: Env, caller: Address, from_asset: Address, to_asset: Address, amount: u128);

    fn last_gov_distribution_time(env: Env) -> u64;

    fn distribute_governance_token(env: Env, caller: Address);
}

pub struct StableLiquidityPool;

#[contractimpl]
impl StableLiquidityPoolTrait for StableLiquidityPool {
    fn init(
        env: Env,
        admin: Address,
        manager: Address,
        governance_token: Address,
        accepted_assets: Vec<Address>,
        fee_percentage: u128,
        treasury: Address,
    ) {
        can_init_contract(&env);
        set_core_state(
            &env,
            &CoreState {
                admin,
                manager,
                governance_token,
                accepted_assets,
                fee_percentage,
                total_deposited: 0,
                treasury,
            },
        );
    }

    fn get_core_state(env: Env) -> CoreState {
        get_core_state(&env)
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        get_core_state(&env).admin.require_auth();
        env.update_current_contract_wasm(&new_wasm_hash);
    }

    fn version(env: Env) -> (Symbol, Symbol) {
        (CONTRACT_DESCRIPTION, CONTRACT_VERSION)
    }

    fn deposit(env: Env, caller: Address, asset: Address, amount: u128) {
        caller.require_auth();
        let mut core_state: CoreState = get_core_state(&env);

        if !validate_deposit_asset(&env, &core_state.accepted_assets, &asset) {
            panic_with_error!(&env, SCErrors::InvalidAsset);
        }

        make_deposit(&env, &caller, &asset, &amount);

        let mut deposit: Deposit = get_deposit(&env, &caller);
        deposit.last_deposit = env.ledger().timestamp();
        deposit.amount = deposit.amount + amount;

        save_deposit(&env, &deposit);

        let mut depositors: Vec<Address> = get_depositors(&env);
        if !is_depositor_listed(&depositors, &caller) {
            depositors.push_back(caller);
            save_depositors(&env, &depositors)
        }

        core_state.total_deposited = core_state.total_deposited + amount;
        set_core_state(&env, &core_state);
    }

    fn withdraw(env: Env, caller: Address, asset: Address) {
        caller.require_auth();
        let mut core_state: CoreState = get_core_state(&env);

        let deposit: Deposit = get_deposit(&env, &caller);
        if deposit.amount == 0 {
            panic_with_error!(&env, SCErrors::NothingToWithdraw);
        }

        let min_amount: u64 = deposit.last_deposit + (3600 * 48);

        if env.ledger().timestamp() < min_amount {
            panic_with_error!(&env, SCErrors::LockedPeriodUncompleted);
        }

        let asset_balance: i128 =
            token::Client::new(&env, &asset).balance(&env.current_contract_address());

        let amount_to_withdraw: u128 =
            calculate_depositor_withdrawal(&deposit, &core_state.total_deposited, &asset_balance);

        make_withdrawal(&env, &deposit.depositor, &asset, &amount_to_withdraw);
        remove_deposit(&env, &caller);

        let mut depositors: Vec<Address> = get_depositors(&env);
        depositors = remove_depositor_from_depositors(&depositors, &caller);
        save_depositors(&env, &depositors);

        core_state.total_deposited = core_state.total_deposited - deposit.amount;
        set_core_state(&env, &core_state);
    }

    fn get_deposit(env: Env, caller: Address) -> Deposit {
        get_deposit(&env, &caller)
    }

    fn swap(env: Env, caller: Address, from_asset: Address, to_asset: Address, amount: u128) {
        caller.require_auth();

        let core_state: CoreState = get_core_state(&env);

        if !validate_deposit_asset(&env, &core_state.accepted_assets, &from_asset) {
            panic_with_error!(&env, SCErrors::InvalidAsset);
        }

        if !validate_deposit_asset(&env, &core_state.accepted_assets, &to_asset) {
            panic_with_error!(&env, SCErrors::InvalidAsset);
        }

        let fee: u128 = div_floor(amount * core_state.fee_percentage, 1_0000000);
        let protocol_share: u128 = div_floor(fee, 2);
        let amount_to_exchange: u128 = amount - fee;

        make_deposit(&env, &caller, &from_asset, &amount_to_exchange);
        make_withdrawal(&env, &caller, &to_asset, &amount_to_exchange);

        token::Client::new(&env, &from_asset).transfer(
            &caller,
            &core_state.treasury,
            &(protocol_share.clone() as i128),
        );
    }

    fn last_gov_distribution_time(env: Env) -> u64 {
        get_last_governance_token_distribution_time(&env)
    }

    fn distribute_governance_token(env: Env, caller: Address) {
        caller.require_auth();
        let daily_distribution: u128 = 16438_0000000;
        let core_state: CoreState = get_core_state(&env);

        let last_distribution = get_last_governance_token_distribution_time(&env);

        if env.ledger().timestamp() < last_distribution + (3600 * 24) {
            panic_with_error!(&env, &SCErrors::RecentDistribution);
        }

        let depositors: Vec<Address> = get_depositors(&env);
        let governance_token = token::Client::new(&env, &core_state.governance_token);

        for item in depositors.iter() {
            let deposit: Deposit = get_deposit(&env, &item.unwrap());
            let deposit_percentage =
                div_floor(deposit.amount * 1_0000000, core_state.total_deposited);

            let amount_to_send: u128 =
                div_floor(deposit_percentage * daily_distribution, 1_0000000);

            governance_token.transfer(
                &env.current_contract_address(),
                &deposit.depositor,
                &(amount_to_send as i128),
            );
        }

        set_last_governance_token_distribution_time(&env);
    }
}
