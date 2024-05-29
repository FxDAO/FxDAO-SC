use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStats};
use crate::storage::deposits::Deposit;
use crate::storage::liquidations::Liquidation;
use crate::utils::core::{
    bump_instance, can_init_contract, get_core_state, get_core_stats, set_core_state,
    set_core_stats,
};
use crate::utils::deposits::{
    bump_deposit, bump_depositors, get_deposit, get_depositors, has_deposit, is_depositor_listed,
    make_deposit, make_withdrawal, remove_deposit, remove_depositor_from_depositors, save_deposit,
    save_depositors,
};
use crate::utils::liquidations::{
    bump_liquidation, check_liquidation_exist, get_liquidation, set_liquidation,
};
use crate::vaults;
use crate::vaults::{Currency, OptionalVaultKey, Vault};
use num_integer::div_floor;
use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, vec, Address, BytesN, Env, IntoVal,
    Symbol, Vec,
};

use crate::oracle::{Asset, Client as OracleClient, PriceData};

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
        governance_token: Address,
        oracle_contract: Address,
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

    fn withdraw_col(env: Env, caller: Address);

    fn liquidate(env: Env, liquidator: Address);

    fn get_liquidations(env: Env, indexes: Vec<u64>) -> Vec<Liquidation>;

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
        governance_token: Address,
        oracle_contract: Address,
    ) {
        can_init_contract(&env);

        let share: Vec<u32> = Vec::from_array(&env, [1, 2]);
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
                treasury_share: share.clone(),
                liquidator_share: share.clone(),
                governance_token,
                oracle_contract,
            },
        );

        set_core_stats(
            &env,
            &CoreStats {
                total_deposits: 0,
                lifetime_deposited: 0,
                current_deposited: 0,
                lifetime_profit: 0,
                lifetime_liquidated: 0,
                liquidation_index: 0,
                rewards_factor: 0,
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
            liquidation_index: core_stats.liquidation_index,
        };
        save_deposit(&env, &deposit);

        let mut depositors: Vec<Address> = get_depositors(&env);
        if !is_depositor_listed(&depositors, &caller) {
            depositors.push_back(caller.clone());
            save_depositors(&env, &depositors)
        }

        core_stats.total_deposits += 1;
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

        if deposit.liquidation_index < core_stats.liquidation_index {
            panic_with_error!(&env, &SCErrors::CollateralAvailable);
        }

        let min_timestamp: u64 = deposit.last_deposit + (3600 * 48);

        if env.ledger().timestamp() < min_timestamp {
            panic_with_error!(&env, &SCErrors::LockedPeriodUncompleted);
        }

        remove_deposit(&env, &caller);

        // We first calculate the amount of stables to withdraw
        let calculated_stable_to_withdraw: u128 = div_floor(
            deposit.shares * core_stats.current_deposited,
            core_stats.total_shares,
        );

        core_stats.current_deposited -= calculated_stable_to_withdraw;
        core_stats.total_shares -= deposit.shares;
        core_stats.total_deposits -= 1;

        make_withdrawal(
            &env,
            &core_state.deposit_asset,
            &deposit.depositor,
            calculated_stable_to_withdraw as i128,
        );

        set_core_stats(&env, &core_stats);

        let mut depositors: Vec<Address> = get_depositors(&env);
        depositors = remove_depositor_from_depositors(&depositors, &caller);
        save_depositors(&env, &depositors);

        bump_depositors(&env);
    }

    fn withdraw_col(env: Env, caller: Address) {
        bump_instance(&env);
        caller.require_auth();

        let core_state: CoreState = get_core_state(&env);
        let core_stats: CoreStats = get_core_stats(&env);

        if !has_deposit(&env, &caller) {
            panic_with_error!(&env, &SCErrors::DepositDoesntExist);
        }

        let mut deposit: Deposit = get_deposit(&env, &caller);

        if deposit.liquidation_index == core_stats.liquidation_index {
            panic_with_error!(&env, &SCErrors::NoCollateralAvailable);
        }

        let target_index: u64 = if deposit.liquidation_index + 10 > core_stats.liquidation_index - 1
        {
            core_stats.liquidation_index - 1
        } else {
            deposit.liquidation_index + 10
        };

        let mut col_to_withdraw: u128 = 0;
        let mut total_debt_covered: u128 = 0;
        let mut current_index: u64 = deposit.liquidation_index.clone();
        let mut finished: bool = false;

        while !finished {
            if current_index > target_index {
                finished = true;
                break;
            }

            let mut liquidation: Liquidation = get_liquidation(&env, current_index.clone());
            let shares_left: u128 = liquidation.total_shares - liquidation.shares_redeemed;

            if shares_left == 0 {
                // We shouldn't be able to reach this line but just in case we generate an error so we can debug later
                panic_with_error!(&env, &SCErrors::UnexpectedError);
            }

            let percentage_of_debt_paid: u128 =
                div_floor(deposit.shares * 1_0000000, liquidation.total_shares);
            let percentage_col_owned: u128 = div_floor(deposit.shares * 1_0000000, shares_left);

            let share_of_col_liquidated: u128 = div_floor(
                liquidation.col_to_withdraw * percentage_col_owned,
                1_0000000,
            );

            let share_of_debt_paid: u128 = div_floor(
                liquidation.total_debt_paid * percentage_of_debt_paid,
                1_0000000,
            );

            liquidation.shares_redeemed += deposit.shares;
            liquidation.col_to_withdraw -= share_of_col_liquidated;
            set_liquidation(&env, &liquidation);

            col_to_withdraw += share_of_col_liquidated;
            total_debt_covered += share_of_debt_paid;
            current_index += 1;

            if total_debt_covered >= deposit.amount
                || (deposit.amount - total_debt_covered) < 1_0000000
            {
                finished = true;
            }
        }

        deposit.liquidation_index = core_stats.liquidation_index;

        save_deposit(&env, &deposit);

        make_withdrawal(
            &env,
            &core_state.collateral_asset,
            &deposit.depositor,
            col_to_withdraw as i128,
        );
    }

    // The liquidation process goes this way:
    // 1.- We first get the balance in the contract to know how much we can liquidate
    // 2.- We get all the vaults that can be liquidated
    // 3.- We iterate among the vaults and calculate how many of them we can liquidate
    // 4.- We call the vaults contract to liquidate the vaults (if is at least 1)
    // 5.- After we receive the collateral, we distributed it to others minus the contract fee
    // 6.- The collateral left is divided and distributed between the treasury and the liquidator
    fn liquidate(env: Env, liquidator: Address) {
        bump_instance(&env);
        liquidator.require_auth();
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

        env.authorize_as_current_contract(Vec::from_array(
            &env,
            [InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: core_state.deposit_asset.clone(),
                    fn_name: symbol_short!("burn"),
                    args: (
                        env.current_contract_address(),
                        total_debt_to_pay.clone() as i128,
                    )
                        .into_val(&env),
                },
                sub_invocations: Vec::new(&env),
            })],
        ));

        let vaults_contract = vaults::Client::new(&env, &core_state.vaults_contract);

        let vaults_liquidated: Vec<Vault> = vaults_contract.liquidate(
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

        let rate: PriceData = OracleClient::new(&env, &core_state.oracle_contract)
            .lastprice(
                &env.current_contract_address(),
                &Asset::Other(core_state.denomination_asset.clone()),
            )
            .unwrap();

        let collateral_paid_for: u128 = div_floor(total_debt_paid * 1_0000000, rate.price as u128);

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

        // We set a record of the liquidation
        set_liquidation(
            &env,
            &Liquidation {
                index: core_stats.liquidation_index,
                total_deposits: core_stats.total_deposits,
                total_debt_paid,
                total_col_liquidated: end_collateral,
                col_to_withdraw: end_collateral,
                share_price: core_stats.share_price,
                total_shares: core_stats.total_shares,
                shares_redeemed: 0,
            },
        );

        bump_liquidation(&env, core_stats.liquidation_index);

        let new_total_deposited: u128 = core_stats.current_deposited - total_debt_paid;

        core_stats.share_price = div_floor(
            new_total_deposited * core_stats.share_price,
            core_stats.current_deposited,
        );
        core_stats.current_deposited -= total_debt_paid;
        core_stats.lifetime_profit += collateral_gained;
        core_stats.lifetime_liquidated += end_collateral;
        core_stats.liquidation_index += 1;

        set_core_stats(&env, &core_stats);
        bump_depositors(&env);
    }

    fn get_liquidations(env: Env, indexes: Vec<u64>) -> Vec<Liquidation> {
        bump_instance(&env);

        if indexes.len() > 10 {
            panic_with_error!(&env, &SCErrors::CantGetMoreThanTenLiquidations);
        }

        let mut liquidations: Vec<Liquidation> = Vec::new(&env);
        for index in indexes.iter() {
            if check_liquidation_exist(&env, index.clone()) {
                bump_liquidation(&env, index.clone());
                liquidations.push_back(get_liquidation(&env, index));
            } else {
                panic_with_error!(&env, &SCErrors::LiquidationDoesntExist);
            }
        }

        liquidations
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
