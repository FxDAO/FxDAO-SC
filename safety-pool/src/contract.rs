use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStats};
use crate::storage::deposits::Deposit;
use crate::utils::core::{
    bump_instance, can_init_contract, get_core_state, get_core_stats,
    get_last_governance_token_distribution_time, set_core_state, set_core_stats,
    set_last_governance_token_distribution_time,
};
use crate::utils::deposits::{
    bump_deposit, bump_depositors, get_contract_balance, get_deposit, get_depositors, has_deposit,
    is_depositor_listed, make_deposit, make_withdrawal, remove_deposit,
    remove_depositor_from_depositors, save_deposit, save_depositors,
};
use crate::vaults;
use crate::vaults::{Currency, OptionalVaultKey, Vault};
use num_integer::div_floor;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token, vec, Address, BytesN, Env,
    Symbol, Vec,
};
use token::Client as TokenClient;

pub const CONTRACT_DESCRIPTION: Symbol = symbol_short!("SafetyP");
pub const CONTRACT_VERSION: Symbol = symbol_short!("0_3_0");

pub trait SafetyPoolContractTrait {
    fn init(
        env: Env,
        admin: Address,
        vaults_contract: Address,
        treasury_contract: Address,
        collateral_asset: Address,
        deposit_asset: Address,
        denomination_asset: Symbol,
        min_deposit: u128,
        treasury_share: Vec<u32>,
        liquidator_share: Vec<u32>,
        governance_token: Address,
    );

    fn get_core_state(env: Env) -> CoreState;
    fn get_core_stats(env: Env) -> CoreStats;

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);

    fn version(env: Env) -> (Symbol, Symbol);

    fn update_contract_admin(env: Env, contract_admin: Address);

    fn update_vaults_contract(env: Env, vaults_contract: Address);

    fn update_treasury_contract(env: Env, treasury_contract: Address);

    fn update_min_deposit(env: Env, min_deposit: u128);

    fn update_treasury_share(env: Env, treasury_share: Vec<u32>);

    fn update_liquidator_share(env: Env, treasury_share: Vec<u32>);

    fn deposit(env: Env, caller: Address, deposit_amount: u128);

    fn get_deposit(env: Env, caller: Address) -> Deposit;

    fn get_depositors(env: Env) -> Vec<Address>;

    // TODO: Improve the logic which distributes the earned collateral
    fn withdraw(env: Env, caller: Address);

    fn liquidate(env: Env, liquidator: Address);

    // fn last_gov_distribution_time(env: Env) -> u64;
    //
    // fn distribute_governance_token(env: Env, address: Address);
}

#[contract]
pub struct SafetyPoolContract;

// TODO: Add events for each function
#[contractimpl]
impl SafetyPoolContractTrait for SafetyPoolContract {
    fn init(
        env: Env,
        admin: Address,
        vaults_contract: Address,
        treasury_contract: Address,
        collateral_asset: Address,
        deposit_asset: Address,
        denomination_asset: Symbol,
        min_deposit: u128,
        treasury_share: Vec<u32>,
        liquidator_share: Vec<u32>,
        governance_token: Address,
    ) {
        can_init_contract(&env);
        set_core_state(
            &env,
            &CoreState {
                admin,
                collateral_asset,
                deposit_asset,
                vaults_contract,
                treasury_contract,
                denomination_asset,
                min_deposit,
                treasury_share,
                liquidator_share,
                governance_token,
            },
        );

        set_core_stats(
            &env,
            &CoreStats {
                lifetime_deposited: 0,
                current_deposited: 0,
                lifetime_profit: 0,
                lifetime_liquidated: 0,
                current_liquidated: 0,
                collateral_factor: 0,
                total_shares: 0,
                share_price: 1_0000000,
            },
        );

        bump_instance(&env);
    }

    fn get_core_state(env: Env) -> CoreState {
        bump_instance(&env);
        get_core_state(&env)
    }

    fn get_core_stats(env: Env) -> CoreStats {
        bump_instance(&env);
        get_core_stats(&env)
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        bump_instance(&env);
        let core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn version(env: Env) -> (Symbol, Symbol) {
        bump_instance(&env);
        (CONTRACT_DESCRIPTION, CONTRACT_VERSION)
    }

    fn update_contract_admin(env: Env, contract_admin: Address) {
        bump_instance(&env);
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.admin = contract_admin;
        set_core_state(&env, &core_state);
    }

    fn update_vaults_contract(env: Env, vaults_contract: Address) {
        bump_instance(&env);
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.vaults_contract = vaults_contract;
        set_core_state(&env, &core_state);
    }

    fn update_treasury_contract(env: Env, treasury_contract: Address) {
        bump_instance(&env);
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.treasury_contract = treasury_contract;
        set_core_state(&env, &core_state);
    }

    fn update_min_deposit(env: Env, min_deposit: u128) {
        bump_instance(&env);
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.min_deposit = min_deposit;
        set_core_state(&env, &core_state);
    }

    fn update_treasury_share(env: Env, treasury_share: Vec<u32>) {
        bump_instance(&env);
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.treasury_share = treasury_share;
        set_core_state(&env, &core_state);
    }

    fn update_liquidator_share(env: Env, liquidator_share: Vec<u32>) {
        bump_instance(&env);
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.liquidator_share = liquidator_share;
        set_core_state(&env, &core_state);
    }

    fn deposit(env: Env, caller: Address, amount: u128) {
        bump_instance(&env);
        caller.require_auth();
        let core_state: CoreState = get_core_state(&env);
        let mut core_stats: CoreStats = get_core_stats(&env);

        if amount < core_state.min_deposit {
            panic_with_error!(&env, SCErrors::BelowMinDeposit);
        }

        if has_deposit(&env, &caller) {
            panic_with_error!(&env, &SCErrors::DepositAlreadyCreated);
        }

        make_deposit(&env, &core_state.deposit_asset, &caller, &amount);

        let shares_to_issue: u128 = div_floor(amount * 1_0000000, core_stats.share_price);
        let deposit: Deposit = Deposit {
            depositor: caller.clone(),
            amount: amount.clone(),
            last_deposit: env.ledger().timestamp(),
            shares: shares_to_issue,
            share_price_paid: core_stats.share_price,
            current_collateral_factor: core_stats.collateral_factor,
        };
        save_deposit(&env, &deposit);

        let mut depositors: Vec<Address> = get_depositors(&env);
        if !is_depositor_listed(&depositors, &caller) {
            depositors.push_back(caller.clone());
            save_depositors(&env, &depositors)
        }

        core_stats.lifetime_deposited += amount;
        core_stats.current_deposited += amount;
        core_stats.total_shares += shares_to_issue;
        set_core_stats(&env, &core_stats);

        bump_deposit(&env, caller);
        bump_depositors(&env);
    }

    fn get_deposit(env: Env, caller: Address) -> Deposit {
        bump_instance(&env);
        caller.require_auth();

        if !has_deposit(&env, &caller) {
            panic_with_error!(&env, &SCErrors::DepositDoesntExist);
        }

        bump_deposit(&env, caller.clone());
        bump_depositors(&env);

        get_deposit(&env, &caller)
    }

    fn get_depositors(env: Env) -> Vec<Address> {
        bump_instance(&env);
        bump_depositors(&env);
        get_depositors(&env)
    }

    fn withdraw(env: Env, caller: Address) {
        bump_instance(&env);
        // TODO: We need to check if there are vaults that can be liquidated before allowing the withdraw.
        caller.require_auth();

        let core_state: CoreState = get_core_state(&env);
        let mut core_stats: CoreStats = get_core_stats(&env);

        if !has_deposit(&env, &caller) {
            panic_with_error!(&env, &SCErrors::DepositDoesntExist);
        }

        let deposit: Deposit = get_deposit(&env, &caller);

        let min_timestamp: u64 = deposit.last_deposit + (3600 * 48);

        if env.ledger().timestamp() < min_timestamp {
            panic_with_error!(&env, &SCErrors::LockedPeriodUncompleted);
        }

        remove_deposit(&env, &caller);

        // We first calculate the amount of stables to withdraw
        let calculated_stable_to_withdraw: u128 = div_floor(
            // deposit.shares * core_stats.current_deposited,
            // core_stats.total_shares,
            deposit.amount * core_stats.share_price,
            deposit.share_price_paid,
        );

        core_stats.current_deposited -= calculated_stable_to_withdraw;
        core_stats.total_shares -= deposit.shares;
        make_withdrawal(
            &env,
            &core_state.deposit_asset,
            &deposit.depositor,
            calculated_stable_to_withdraw as i128,
        );

        // Then we withdraw the collateral
        let mut calculated_collateral_to_withdraw: u128 = div_floor(
            // div_floor(
            deposit.amount * (core_stats.collateral_factor - deposit.current_collateral_factor),
            // deposit.share_price_paid,
            1_0000000,
        );
        // 1_0000000,
        // );

        if calculated_collateral_to_withdraw > 0 {
            if calculated_collateral_to_withdraw > core_stats.current_liquidated {
                calculated_collateral_to_withdraw = core_stats.current_liquidated;
            }
            core_stats.current_liquidated -= calculated_collateral_to_withdraw;
            make_withdrawal(
                &env,
                &core_state.collateral_asset,
                &deposit.depositor,
                calculated_collateral_to_withdraw as i128,
            );
        }

        set_core_stats(&env, &core_stats);

        let mut depositors: Vec<Address> = get_depositors(&env);
        depositors = remove_depositor_from_depositors(&depositors, &caller);
        save_depositors(&env, &depositors);

        bump_deposit(&env, caller);
        bump_depositors(&env);
    }

    /// The liquidation process goes this way:
    /// 1.- We first get the balance in the contract to know how much we can liquidate
    /// 2.- We get all the vaults that can be liquidated
    /// 3.- We iterate among the vaults and calculate how many of them we can liquidate
    /// 4.- We call the vaults contract to liquidate the vaults (if is at least 1)
    /// 5.- After we receive the collateral, we distributed it to others minus the contract fee
    /// 6.- The collateral left is divided and distributed between the treasury and the liquidator
    fn liquidate(env: Env, liquidator: Address) {
        bump_instance(&env);
        let core_state: CoreState = get_core_state(&env);
        let mut core_stats: CoreStats = get_core_stats(&env);

        let currency: Currency = vaults::Client::new(&env, &core_state.vaults_contract)
            .get_currency(&core_state.denomination_asset);

        let vaults_to_liquidate: Vec<Vault> =
            vaults::Client::new(&env, &core_state.vaults_contract).get_vaults(
                &OptionalVaultKey::None,
                &core_state.denomination_asset,
                &10,
                &true,
            );

        let mut total_debt_to_pay: u128 = 0;
        let mut total_vaults: u32 = 0;

        for user_vault in vaults_to_liquidate.iter() {
            if total_debt_to_pay + user_vault.total_debt <= core_stats.current_deposited {
                total_debt_to_pay += user_vault.total_debt;
                total_vaults += 1;
            } else {
                break;
            }
        }

        if total_vaults == 0 {
            panic_with_error!(&env, SCErrors::CantLiquidateVaults);
        }

        let vaults_liquidated: Vec<Vault> = vaults::Client::new(&env, &core_state.vaults_contract)
            .liquidate(
                &env.current_contract_address(),
                &core_state.denomination_asset,
                &total_vaults,
            );

        let mut total_debt_paid: u128 = 0;
        let mut total_collateral_received: u128 = 0;
        for vault in vaults_liquidated {
            total_debt_paid += vault.total_debt;
            total_collateral_received += vault.total_collateral;
        }

        let collateral_paid_for: u128 =
            div_floor(total_debt_paid * 1_0000000, currency.rate as u128);

        // If collateral paid for is higher than the amount received it means there was a lost in the liquidation.
        let collateral_gained: u128 = if collateral_paid_for > total_collateral_received {
            0
        } else {
            total_collateral_received - collateral_paid_for
        };

        // The "shareable_profit" is the part of the profit that belongs to the treasury and from there the protocol pays the liquidator
        let shareable_profit = div_floor(
            collateral_gained * core_state.treasury_share.get(0).unwrap() as u128,
            core_state.treasury_share.get(1).unwrap() as u128,
        );

        let liquidator_share: u128 = div_floor(
            shareable_profit * core_state.liquidator_share.get(0).unwrap() as u128,
            core_state.liquidator_share.get(1).unwrap() as u128,
        );

        let treasury_share: u128 = shareable_profit - liquidator_share;

        if liquidator_share > 0 {
            make_withdrawal(
                &env,
                &core_state.collateral_asset,
                &liquidator,
                liquidator_share as i128,
            );
        }

        if treasury_share > 0 {
            make_withdrawal(
                &env,
                &core_state.collateral_asset,
                &core_state.treasury_contract,
                treasury_share as i128,
            );
        }

        let end_collateral: u128 = total_collateral_received - shareable_profit;
        core_stats.collateral_factor +=
            div_floor(end_collateral * 1_0000000, core_stats.lifetime_deposited);

        let new_total_deposited: u128 = core_stats.current_deposited - total_debt_paid;
        core_stats.share_price = div_floor(
            new_total_deposited * core_stats.share_price,
            core_stats.current_deposited,
        );

        core_stats.current_deposited -= total_debt_paid;
        core_stats.lifetime_profit += collateral_gained;
        core_stats.lifetime_liquidated += end_collateral;
        core_stats.current_liquidated += end_collateral;

        set_core_stats(&env, &core_stats);
        bump_depositors(&env);
    }

    // fn last_gov_distribution_time(env: Env) -> u64 {
    //     bump_instance(&env);
    //     bump_depositors(&env);
    //     get_last_governance_token_distribution_time(&env)
    // }
    //
    // fn distribute_governance_token(env: Env, caller: Address) {
    //     bump_instance(&env);
    //     caller.require_auth();
    //     let daily_distribution: u128 = 8219_0000000;
    //     let core_state: CoreState = get_core_state(&env);
    //
    //     let last_distribution = get_last_governance_token_distribution_time(&env);
    //
    //     if env.ledger().timestamp() < last_distribution + (3600 * 24) {
    //         panic_with_error!(&env, &SCErrors::RecentDistribution);
    //     }
    //
    //     let depositors = get_depositors(&env);
    //     let mut approved_users: Vec<Deposit> = vec![&env] as Vec<Deposit>;
    //     let mut total_approved_users_deposit: u128 = 0;
    //     // Min deposit must be 48 hrs before this moment
    //     let max_deposit_time: u64 = env.ledger().timestamp() - (3600 * 48);
    //     let governance_token: TokenClient = TokenClient::new(&env, &core_state.governance_token);
    //
    //     for depositor in depositors.iter() {
    //         let deposit: Deposit = get_deposit(&env, &depositor);
    //
    //         if deposit.last_deposit < max_deposit_time && governance_token.authorized(&depositor) {
    //             total_approved_users_deposit = total_approved_users_deposit + deposit.shares;
    //             approved_users.push_front(deposit);
    //         }
    //     }
    //
    //     for deposit in approved_users.iter() {
    //         let deposit_percentage =
    //             div_floor(deposit.shares * 1_0000000, total_approved_users_deposit);
    //
    //         let amount_to_send: u128 =
    //             div_floor(deposit_percentage * daily_distribution, 1_0000000);
    //
    //         governance_token.transfer(
    //             &env.current_contract_address(),
    //             &deposit.depositor,
    //             &(amount_to_send as i128),
    //         );
    //     }
    //
    //     set_last_governance_token_distribution_time(&env);
    //     bump_depositors(&env);
    // }
}
