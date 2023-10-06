use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::storage::deposits::Deposit;
use crate::utils::core::{
    bump_instance, can_init_contract, get_core_state, get_last_governance_token_distribution_time,
    set_core_state, set_last_governance_token_distribution_time,
};
use crate::utils::deposits::{
    bump_deposit, bump_depositors, get_deposit, get_depositors, has_deposit, is_depositor_listed,
    make_deposit, make_withdrawal, remove_deposit, remove_depositor_from_depositors, save_deposit,
    save_depositors, validate_deposit_asset,
};
use num_integer::div_floor;
use soroban_sdk::{
    contract, contractimpl, map, panic_with_error, symbol_short, token, Address, BytesN, Env, Map,
    Symbol, Vec,
};

pub const CONTRACT_DESCRIPTION: Symbol = symbol_short!("StableLP");
pub const CONTRACT_VERSION: Symbol = symbol_short!("0_3_0");

pub trait StableLiquidityPoolContractTrait {
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

    fn withdraw(
        env: Env,
        caller: Address,
        shares_to_redeem: u128,
        assets_orders: Map<Address, u128>,
    );

    fn get_deposit(env: Env, caller: Address) -> Deposit;

    fn get_depositors(env: Env) -> Vec<Address>;

    fn get_supported_assets(env: Env) -> Vec<Address>;

    fn swap(env: Env, caller: Address, from_asset: Address, to_asset: Address, amount: u128);

    fn last_gov_distribution_time(env: Env) -> u64;

    fn distribute_governance_token(env: Env);
}

#[contract]
pub struct StableLiquidityPoolContract;

#[contractimpl]
impl StableLiquidityPoolContractTrait for StableLiquidityPoolContract {
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
                share_price: 1_0000000,
                total_shares: 0,
                treasury,
            },
        );
        bump_instance(&env);
    }

    fn get_core_state(env: Env) -> CoreState {
        bump_instance(&env);
        get_core_state(&env)
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        bump_instance(&env);
        get_core_state(&env).admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn version(env: Env) -> (Symbol, Symbol) {
        bump_instance(&env);
        (CONTRACT_DESCRIPTION, CONTRACT_VERSION)
    }

    fn deposit(env: Env, caller: Address, asset: Address, amount_deposit: u128) {
        bump_instance(&env);
        caller.require_auth();
        let mut core_state: CoreState = get_core_state(&env);

        if !validate_deposit_asset(&core_state.accepted_assets, &asset) {
            panic_with_error!(&env, &SCErrors::InvalidAsset);
        }

        make_deposit(&env, &caller, &asset, &amount_deposit);

        let shares_to_issue: u128 = div_floor(amount_deposit * 1_0000000, core_state.share_price);
        let mut deposit: Deposit = get_deposit(&env, &caller);
        deposit.last_deposit = env.ledger().timestamp();
        deposit.shares = deposit.shares + shares_to_issue;
        save_deposit(&env, &deposit);

        let mut depositors: Vec<Address> = get_depositors(&env);
        if !is_depositor_listed(&depositors, &caller) {
            depositors.push_back(caller.clone());
            save_depositors(&env, &depositors)
        }

        core_state.total_deposited = core_state.total_deposited + amount_deposit;
        core_state.total_shares = core_state.total_shares + shares_to_issue;
        set_core_state(&env, &core_state);

        bump_deposit(&env, caller);
        bump_depositors(&env);
    }

    fn withdraw(
        env: Env,
        caller: Address,
        shares_to_redeem: u128,
        assets_orders: Map<Address, u128>,
    ) {
        bump_instance(&env);
        caller.require_auth();
        let mut core_state: CoreState = get_core_state(&env);
        let calculated_amount_to_withdraw: u128 = div_floor(
            shares_to_redeem * core_state.total_deposited,
            core_state.total_shares,
        );

        let mut deposit: Deposit = get_deposit(&env, &caller);
        if deposit.shares == 0 {
            panic_with_error!(&env, &SCErrors::NothingToWithdraw);
        }

        if &deposit.shares < &shares_to_redeem {
            panic_with_error!(&env, &SCErrors::NotEnoughSharesToWithdraw);
        }

        let min_timestamp: u64 = deposit.last_deposit + (3600 * 48);

        if env.ledger().timestamp() < min_timestamp {
            panic_with_error!(&env, &SCErrors::LockedPeriodUncompleted);
        }

        let mut withdraw_amount: u128 = 0;

        for token in core_state.accepted_assets.iter() {
            if assets_orders.contains_key(token.clone()) {
                withdraw_amount = withdraw_amount + assets_orders.get(token.clone()).unwrap();
            }
        }

        if calculated_amount_to_withdraw != withdraw_amount {
            panic_with_error!(&env, &SCErrors::InvalidWithdraw);
        }

        for (asset, amount) in assets_orders.iter() {
            if amount != 0 {
                make_withdrawal(&env, &deposit.depositor, &asset, &amount);
            }
        }

        if shares_to_redeem < deposit.shares {
            deposit.shares = deposit.shares - shares_to_redeem;
            save_deposit(&env, &deposit);
            bump_deposit(&env, caller);
        } else {
            remove_deposit(&env, &caller);
            let mut depositors: Vec<Address> = get_depositors(&env);
            depositors = remove_depositor_from_depositors(&depositors, &caller);
            save_depositors(&env, &depositors);
        }

        core_state.total_deposited = core_state.total_deposited - withdraw_amount;
        core_state.total_shares = core_state.total_shares - shares_to_redeem;
        if core_state.total_deposited == 0 && core_state.total_shares == 0 {
            core_state.share_price = 1_0000000;
        }
        set_core_state(&env, &core_state);

        bump_depositors(&env);
    }

    fn get_deposit(env: Env, caller: Address) -> Deposit {
        bump_instance(&env);
        if has_deposit(&env, &caller) {
            bump_deposit(&env, caller.clone());
        }
        bump_depositors(&env);

        get_deposit(&env, &caller)
    }

    fn get_depositors(env: Env) -> Vec<Address> {
        bump_instance(&env);
        bump_depositors(&env);

        get_depositors(&env)
    }

    fn get_supported_assets(env: Env) -> Vec<Address> {
        bump_instance(&env);
        bump_depositors(&env);
        get_core_state(&env).accepted_assets
    }

    fn swap(env: Env, caller: Address, from_asset: Address, to_asset: Address, amount: u128) {
        bump_instance(&env);
        caller.require_auth();

        let mut core_state: CoreState = get_core_state(&env);

        if !validate_deposit_asset(&core_state.accepted_assets, &from_asset) {
            panic_with_error!(&env, &SCErrors::InvalidAsset);
        }

        if !validate_deposit_asset(&core_state.accepted_assets, &to_asset) {
            panic_with_error!(&env, &SCErrors::InvalidAsset);
        }

        let fee: u128 = div_floor(amount * core_state.fee_percentage, 1_0000000);
        let protocol_share: u128 = div_floor(fee, 2);
        let amount_to_exchange: u128 = amount - fee;

        make_deposit(&env, &caller, &from_asset, &amount);
        make_withdrawal(&env, &caller, &to_asset, &amount_to_exchange);

        token::Client::new(&env, &from_asset).transfer(
            &env.current_contract_address(),
            &core_state.treasury,
            &(protocol_share.clone() as i128),
        );

        let pool_profit: u128 = fee - protocol_share;
        let new_total_deposited: u128 = core_state.total_deposited + pool_profit;
        let new_share_price: u128 = div_floor(
            new_total_deposited * core_state.share_price,
            core_state.total_deposited,
        );

        core_state.share_price = new_share_price;
        core_state.total_deposited = new_total_deposited;

        set_core_state(&env, &core_state);
    }

    // Update the way we distribute the governance tokens
    fn last_gov_distribution_time(env: Env) -> u64 {
        bump_instance(&env);
        bump_depositors(&env);
        get_last_governance_token_distribution_time(&env)
    }

    fn distribute_governance_token(env: Env) {
        bump_instance(&env);
        let daily_distribution: u128 = 16438_0000000;
        let core_state: CoreState = get_core_state(&env);

        let last_distribution = get_last_governance_token_distribution_time(&env);

        if env.ledger().timestamp() < last_distribution + (3600 * 24) {
            panic_with_error!(&env, &SCErrors::RecentDistribution);
        }

        let depositors: Vec<Address> = get_depositors(&env);
        let governance_token = token::Client::new(&env, &core_state.governance_token);

        for depositor in depositors.iter() {
            let deposit: Deposit = get_deposit(&env, &depositor);
            let deposit_percentage =
                div_floor(deposit.shares * 1_0000000, core_state.total_deposited);

            let amount_to_send: u128 =
                div_floor(deposit_percentage * daily_distribution, 1_0000000);

            governance_token.transfer(
                &env.current_contract_address(),
                &deposit.depositor,
                &(amount_to_send as i128),
            );
        }

        set_last_governance_token_distribution_time(&env);
        bump_depositors(&env);
    }
}
