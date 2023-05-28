use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::storage::deposits::Deposit;
use crate::token;
use crate::utils::core::{can_init_contract, get_core_state, set_core_state};
use crate::utils::deposits::{
    get_deposit, get_depositors, is_depositor_listed, make_deposit, make_withdrawal,
    remove_deposit, remove_depositor_from_depositors, save_deposit, save_depositors,
};
use crate::vaults;
use crate::vaults::{Currency, UserVault};
use num_integer::div_floor;
use soroban_sdk::{contractimpl, panic_with_error, vec, Address, BytesN, Env, Symbol, Vec};

pub trait SafetyPoolContractTrait {
    fn init(
        env: Env,
        admin: Address,
        vaults_contract: Address,
        treasury_contract: Address,
        collateral_asset: BytesN<32>,
        deposit_asset: BytesN<32>,
        denomination_asset: Symbol,
        min_deposit: u128,
        treasury_share: Vec<u32>,
        liquidator_share: Vec<u32>,
    );

    fn get_core_state(env: Env) -> CoreState;

    fn update_contract_admin(env: Env, contract_admin: Address);

    fn update_vaults_contract(env: Env, vaults_contract: Address);

    fn update_treasury_contract(env: Env, treasury_contract: Address);

    fn update_min_deposit(env: Env, min_deposit: u128);

    fn update_treasury_share(env: Env, treasury_share: Vec<u32>);

    fn update_liquidator_share(env: Env, treasury_share: Vec<u32>);

    fn deposit(env: Env, caller: Address, amount: u128);

    fn get_deposit(env: Env, caller: Address) -> Deposit;

    fn get_depositors(env: Env) -> Vec<Address>;

    fn withdraw(env: Env, caller: Address);

    fn liquidate(env: Env, liquidator: Address);
}

pub struct SafetyPoolContract;

// TODO: Add events for each function
#[contractimpl]
impl SafetyPoolContractTrait for SafetyPoolContract {
    fn init(
        env: Env,
        admin: Address,
        vaults_contract: Address,
        treasury_contract: Address,
        collateral_asset: BytesN<32>,
        deposit_asset: BytesN<32>,
        denomination_asset: Symbol,
        min_deposit: u128,
        treasury_share: Vec<u32>,
        liquidator_share: Vec<u32>,
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
            },
        );
    }

    fn get_core_state(env: Env) -> CoreState {
        get_core_state(&env)
    }

    fn update_contract_admin(env: Env, contract_admin: Address) {
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.admin = contract_admin;
        set_core_state(&env, &core_state);
    }

    fn update_vaults_contract(env: Env, vaults_contract: Address) {
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.vaults_contract = vaults_contract;
        set_core_state(&env, &core_state);
    }

    fn update_treasury_contract(env: Env, treasury_contract: Address) {
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.treasury_contract = treasury_contract;
        set_core_state(&env, &core_state);
    }

    fn update_min_deposit(env: Env, min_deposit: u128) {
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.min_deposit = min_deposit;
        set_core_state(&env, &core_state);
    }

    fn update_treasury_share(env: Env, treasury_share: Vec<u32>) {
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.treasury_share = treasury_share;
        set_core_state(&env, &core_state);
    }

    fn update_liquidator_share(env: Env, liquidator_share: Vec<u32>) {
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.liquidator_share = liquidator_share;
        set_core_state(&env, &core_state);
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
        // TODO: We need to check if there are vaults that can be liquidated before allowing the withdraw.
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

    /// The liquidation process goes this way:
    /// 1.- We first get the balance in the contract to know how much we can liquidate
    /// 2.- We get all the vaults that can be liquidated
    /// 3.- We iterate among the vaults and calculate how many of them we can liquidate
    /// 4.- We call the vaults contract to liquidate the vaults (if is at least 1)
    /// 5.- After we receive the collateral, we distributed it to others minus the contract fee
    /// 6.- The collateral left is divided and distributed between the treasury and the liquidator
    fn liquidate(env: Env, liquidator: Address) {
        let core_state: CoreState = get_core_state(&env);
        let stablecoin_balance: i128 = token::Client::new(&env, &core_state.deposit_asset)
            .balance(&env.current_contract_address());

        let currency_stats: Currency =
            vaults::Client::new(&env, &core_state.vaults_contract.contract_id().unwrap())
                .get_currency(&core_state.denomination_asset);

        let vaults_to_liquidate: Vec<UserVault> =
            vaults::Client::new(&env, &core_state.vaults_contract.contract_id().unwrap())
                .vaults_to_liquidate(&core_state.denomination_asset);

        let mut target_users: Vec<Address> = vec![&env] as Vec<Address>;
        let mut amount_covered: i128 = 0;
        let mut total_collateral: i128 = 0;

        for result in vaults_to_liquidate.iter() {
            let user_vault: UserVault = result.unwrap();
            if amount_covered + user_vault.total_debt <= stablecoin_balance {
                target_users.push_back(user_vault.id);
                amount_covered += user_vault.total_debt;
                total_collateral += user_vault.total_col;
            } else {
                break;
            }
        }

        if target_users.len() == 0 {
            panic_with_error!(&env, SCErrors::CantLiquidateVaults);
        }

        let depositors: Vec<Address> = get_depositors(&env);

        token::Client::new(&env, &core_state.deposit_asset).incr_allow(
            &env.current_contract_address(),
            &core_state.vaults_contract,
            &amount_covered,
        );

        vaults::Client::new(&env, &core_state.vaults_contract.contract_id().unwrap()).liquidate(
            &env.current_contract_address(),
            &core_state.denomination_asset,
            &target_users,
        );

        let collateral_amount_paid: i128 =
            div_floor(amount_covered * 10000000, currency_stats.rate);

        let profit_from_liquidation: i128 = total_collateral - collateral_amount_paid;

        let share_of_profit = div_floor(
            profit_from_liquidation * core_state.treasury_share.get(0).unwrap().unwrap() as i128,
            core_state.treasury_share.get(1).unwrap().unwrap() as i128,
        );

        let amount_to_distribute: i128 = collateral_amount_paid + share_of_profit;

        for result in depositors.iter() {
            let depositor: Address = result.unwrap();
            let mut deposit: Deposit = get_deposit(&env, &depositor);
            let deposit_percentage: i128 =
                div_floor(deposit.amount as i128 * 10000000, stablecoin_balance);
            let collateral_to_send: i128 =
                div_floor(deposit_percentage * amount_to_distribute, 10000000);

            token::Client::new(&env, &core_state.collateral_asset).xfer(
                &env.current_contract_address(),
                &depositor,
                &collateral_to_send,
            );

            let deposit_amount_used: i128 =
                div_floor(deposit_percentage * amount_covered, 100_0000000) * 100;

            deposit.amount = deposit.amount - deposit_amount_used as u128;

            save_deposit(&env, &deposit);
        }

        let collateral_left: i128 = token::Client::new(&env, &core_state.collateral_asset)
            .balance(&env.current_contract_address());

        let liquidator_share: i128 = div_floor(
            collateral_left * core_state.liquidator_share.get(0).unwrap().unwrap() as i128,
            core_state.liquidator_share.get(1).unwrap().unwrap() as i128,
        );

        token::Client::new(&env, &core_state.collateral_asset).xfer(
            &env.current_contract_address(),
            &liquidator,
            &liquidator_share,
        );

        let treasury_share: i128 = collateral_left - liquidator_share;

        token::Client::new(&env, &core_state.collateral_asset).xfer(
            &env.current_contract_address(),
            &core_state.treasury_contract,
            &treasury_share,
        );
    }
}
