use crate::errors::SCErrors;
use crate::utils::core::*;

use crate::storage::core::CoreState;
use crate::storage::currencies::{CurrenciesDataKeys, Currency};
use crate::storage::vaults::{OptionalVaultKey, Vault, VaultIndexKey, VaultKey, VaultsInfo};
use crate::utils::currencies::{
    get_currency, is_currency_active, save_currency, validate_currency,
};
use crate::utils::indexes::calculate_user_vault_index;
use crate::utils::payments::{
    calc_fee, deposit_collateral, deposit_stablecoin, pay_fee, withdraw_collateral,
    withdraw_stablecoin,
};
use crate::utils::vaults::{
    bump_vault, bump_vault_index, calculate_deposit_ratio, can_be_liquidated,
    create_and_insert_vault, get_vault, get_vaults, get_vaults_info, is_vaults_info_started,
    search_vault, set_vaults_info, validate_user_vault, vault_spot_available, withdraw_vault,
};
use num_integer::div_floor;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token, vec, Address, BytesN, Env, Map,
    Symbol, Vec,
};

pub const CONTRACT_DESCRIPTION: Symbol = symbol_short!("Vaults");
pub const CONTRACT_VERSION: Symbol = symbol_short!("0_3_0");

// TODO: Explain each function here
pub trait VaultsContractTrait {
    /// Set up and management
    fn init(
        env: Env,
        admin: Address,
        oracle_admin: Address,
        protocol_manager: Address,
        col_token: Address,
        stable_issuer: Address,
        treasury: Address,
        fee: u128,
    );

    fn get_core_state(env: Env) -> CoreState;

    fn set_admin(env: Env, address: Address);
    fn set_protocol_manager(env: Env, address: Address);

    // TODO: Test these
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
    fn version(env: Env) -> (Symbol, Symbol);

    /// Currencies methods
    fn create_currency(env: Env, denomination: Symbol, contract: Address);
    fn get_currency(env: Env, denomination: Symbol) -> Currency;
    fn set_currency_rate(env: Env, denomination: Symbol, rate: u128);
    fn toggle_currency(env: Env, denomination: Symbol, active: bool);

    /// Vaults methods
    fn set_vault_conditions(
        env: Env,
        min_col_rate: u128,
        min_debt_creation: u128,
        opening_col_rate: u128,
        denomination: Symbol,
    );
    fn get_vaults_info(env: Env, denomination: Symbol) -> VaultsInfo;
    fn calculate_deposit_ratio(currency_rate: u128, collateral: u128, debt: u128) -> u128;
    fn new_vault(
        env: Env,
        prev_key: OptionalVaultKey,
        caller: Address,
        initial_debt: u128,
        collateral_amount: u128,
        denomination: Symbol,
    );
    fn get_vault(env: Env, caller: Address, denomination: Symbol) -> Vault;
    fn get_vault_from_key(env: Env, vault_key: VaultKey) -> Vault;
    fn get_vaults(
        env: Env,
        prev_key: OptionalVaultKey,
        denomination: Symbol,
        total: u32,
        only_to_liquidate: bool,
    ) -> Vec<Vault>;
    fn increase_collateral(
        env: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    );
    fn increase_debt(
        env: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    );
    fn pay_debt(
        env: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    );

    /// Redeeming
    fn redeem(env: Env, caller: Address, denomination: Symbol);

    /// Liquidation
    fn liquidate(
        env: Env,
        liquidator: Address,
        denomination: Symbol,
        total_vaults_to_liquidate: u32,
    ) -> Vec<Vault>;
}

#[contract]
pub struct VaultsContract;

// TODO: Add events for each function
#[contractimpl]
impl VaultsContractTrait for VaultsContract {
    fn init(
        env: Env,
        admin: Address,
        oracle_admin: Address,
        protocol_manager: Address,
        col_token: Address,
        stable_issuer: Address,
        treasury: Address,
        fee: u128,
    ) {
        bump_instance(&env);
        if is_core_created(&env) {
            panic_with_error!(&env, &SCErrors::CoreAlreadySet);
        }

        let core_state: CoreState = CoreState {
            col_token,
            stable_issuer,
            admin,
            oracle_admin,
            protocol_manager,
            panic_mode: false,
            treasury,
            fee,
        };

        save_core_state(&env, &core_state);
    }

    fn get_core_state(env: Env) -> CoreState {
        bump_instance(&env);
        get_core_state(&env)
    }

    fn set_admin(env: Env, address: Address) {
        bump_instance(&env);
        let mut core_state: CoreState = get_core_state(&env);
        core_state.admin.require_auth();
        core_state.admin = address;
        save_core_state(&env, &core_state);
    }

    fn set_protocol_manager(env: Env, address: Address) {
        bump_instance(&env);
        let mut core_state: CoreState = get_core_state(&env);
        core_state.protocol_manager.require_auth();
        core_state.protocol_manager = address;
        save_core_state(&env, &core_state);
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

    fn create_currency(env: Env, denomination: Symbol, contract: Address) {
        bump_instance(&env);
        get_core_state(&env).protocol_manager.require_auth();

        if env
            .storage()
            .instance()
            .has(&CurrenciesDataKeys::Currency(denomination.clone()))
        {
            panic_with_error!(&env, &SCErrors::CurrencyAlreadyAdded);
        }

        save_currency(
            &env,
            &Currency {
                denomination,
                active: false,
                contract,
                rate: 0,
                last_update: env.ledger().timestamp(),
            },
        );
    }

    fn get_currency(env: Env, denomination: Symbol) -> Currency {
        bump_instance(&env);
        validate_currency(&env, &denomination);
        get_currency(&env, &denomination)
    }

    fn set_currency_rate(env: Env, denomination: Symbol, rate: u128) {
        bump_instance(&env);
        // TODO: this method should be updated in the future once there are oracles in the network
        get_core_state(&env).oracle_admin.require_auth();
        validate_currency(&env, &denomination);

        let mut currency = get_currency(&env, &denomination);

        // TODO: Check if the price was updated recently
        if currency.rate != rate {
            currency.rate = rate;
            currency.last_update = env.ledger().timestamp();
            save_currency(&env, &currency);
        } else {
            // TODO: if the last time the rate was changed was more than 15 minutes ago shut down the issuance of new debt
        }
    }

    fn toggle_currency(env: Env, denomination: Symbol, active: bool) {
        bump_instance(&env);
        get_core_state(&env).admin.require_auth();
        validate_currency(&env, &denomination);
        let mut currency = get_currency(&env, &denomination);
        currency.active = active;
        save_currency(&env, &currency);
    }

    fn set_vault_conditions(
        env: Env,
        min_col_rate: u128,
        min_debt_creation: u128,
        opening_col_rate: u128,
        denomination: Symbol,
    ) {
        bump_instance(&env);
        get_core_state(&env).admin.require_auth();

        if !is_vaults_info_started(&env, &denomination) {
            set_vaults_info(
                &env,
                &VaultsInfo {
                    denomination,
                    min_col_rate,
                    min_debt_creation,
                    opening_col_rate,
                    total_vaults: 0,
                    total_col: 0,
                    total_debt: 0,
                    lowest_key: OptionalVaultKey::None,
                },
            );
        } else {
            let current_state: VaultsInfo = get_vaults_info(&env, &denomination);
            set_vaults_info(
                &env,
                &VaultsInfo {
                    denomination,
                    min_col_rate,
                    min_debt_creation,
                    opening_col_rate,
                    total_vaults: current_state.total_vaults,
                    total_col: current_state.total_col,
                    total_debt: current_state.total_debt,
                    lowest_key: current_state.lowest_key,
                },
            );
        }
    }

    fn get_vaults_info(env: Env, denomination: Symbol) -> VaultsInfo {
        bump_instance(&env);
        get_vaults_info(&env, &denomination)
    }

    fn calculate_deposit_ratio(currency_rate: u128, collateral: u128, debt: u128) -> u128 {
        calculate_deposit_ratio(&currency_rate, &collateral, &debt)
    }

    fn new_vault(
        env: Env,
        prev_key: OptionalVaultKey,
        caller: Address,
        initial_debt: u128,
        collateral_amount: u128,
        denomination: Symbol,
    ) {
        bump_instance(&env);
        // TODO: check if we are in panic mode once is implemented
        caller.require_auth();
        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);
        vault_spot_available(&env, caller.clone(), &denomination);

        // TODO: check if collateral price has been updated lately

        let core_state: CoreState = get_core_state(&env);
        let fee: u128 = calc_fee(&core_state.fee, &collateral_amount);
        let vault_col: u128 = collateral_amount - fee;

        if !is_vaults_info_started(&env, &denomination) {
            panic_with_error!(&env, &SCErrors::VaultsInfoHasNotStarted);
        }

        let mut vaults_info: VaultsInfo = get_vaults_info(&env, &denomination);

        if vaults_info.min_debt_creation > initial_debt {
            panic_with_error!(env, &SCErrors::InvalidMinDebtAmount);
        }

        let currency: Currency = get_currency(&env, &denomination);
        let deposit_collateral_rate: u128 =
            calculate_deposit_ratio(&currency.rate, &vault_col, &initial_debt);

        if deposit_collateral_rate < vaults_info.opening_col_rate {
            panic_with_error!(&env, &SCErrors::InvalidOpeningCollateralRatio);
        }

        let new_vault_index: u128 = calculate_user_vault_index(initial_debt, vault_col);
        let new_vault_key: VaultKey = VaultKey {
            index: new_vault_index.clone(),
            account: caller.clone(),
            denomination: denomination.clone(),
        };

        // In case prev value is not None, we confirm its index is not higher than the new Vault index
        match prev_key.clone() {
            OptionalVaultKey::None => {}
            OptionalVaultKey::Some(value) => {
                if new_vault_index < value.index {
                    panic_with_error!(&env, &SCErrors::InvalidPrevVaultIndex);
                }
            }
        }

        let (_, new_vault_key, new_vault_index_key, updated_lowest_key) = create_and_insert_vault(
            &env,
            &vaults_info.lowest_key,
            &new_vault_key,
            &prev_key,
            initial_debt.clone(),
            vault_col.clone(),
        );

        vaults_info.lowest_key = updated_lowest_key;
        vaults_info.total_vaults = vaults_info.total_vaults + 1;
        vaults_info.total_debt = vaults_info.total_debt + initial_debt;
        vaults_info.total_col = vaults_info.total_col + vault_col;
        set_vaults_info(&env, &vaults_info);

        deposit_collateral(&env, &core_state, &caller, vault_col as i128);
        withdraw_stablecoin(&env, &core_state, &currency, &caller, initial_debt as i128);
        pay_fee(&env, &core_state, &caller, fee as i128);

        bump_vault(&env, new_vault_key);
        bump_vault_index(&env, new_vault_index_key);
    }

    fn get_vault(env: Env, user: Address, denomination: Symbol) -> Vault {
        bump_instance(&env);

        let (user_vault, vault_key, vault_index_key) = search_vault(&env, &user, &denomination);

        bump_vault(&env, vault_key);
        bump_vault_index(&env, vault_index_key);

        user_vault
    }

    fn get_vault_from_key(env: Env, vault_key: VaultKey) -> Vault {
        bump_instance(&env);

        validate_user_vault(&env, vault_key.clone());

        let vault_index_key: VaultIndexKey = VaultIndexKey {
            user: vault_key.account.clone(),
            denomination: vault_key.denomination.clone(),
        };

        bump_vault(&env, vault_key.clone());
        bump_vault_index(&env, vault_index_key);

        get_vault(&env, vault_key)
    }

    fn get_vaults(
        env: Env,
        prev_key: OptionalVaultKey,
        denomination: Symbol,
        total: u32,
        only_to_liquidate: bool,
    ) -> Vec<Vault> {
        bump_instance(&env);

        let currency: Currency = get_currency(&env, &denomination);
        let vaults_info: VaultsInfo = get_vaults_info(&env, &denomination);

        if OptionalVaultKey::None == prev_key && OptionalVaultKey::None == vaults_info.lowest_key {
            return vec![&env] as Vec<Vault>;
        }

        get_vaults(
            &env,
            &prev_key,
            &currency,
            &vaults_info,
            total,
            only_to_liquidate,
        )
    }

    fn increase_collateral(
        env: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    ) {
        bump_instance(&env);
        vault_key.account.require_auth();
        validate_currency(&env, &vault_key.denomination);
        is_currency_active(&env, &vault_key.denomination);

        let core_state: CoreState = get_core_state(&env);

        let fee: u128 = calc_fee(&core_state.fee, &amount);
        let collateral: u128 = amount - fee;

        deposit_collateral(&env, &core_state, &vault_key.account, collateral as i128);
        pay_fee(&env, &core_state, &vault_key.account, fee as i128);

        let (target_vault, target_vault_key, _) =
            search_vault(&env, &vault_key.account, &vault_key.denomination);

        // TODO: Test this
        if target_vault.index != vault_key.index {
            panic_with_error!(&env, &SCErrors::IndexProvidedIsNotTheOneSaved);
        }

        let mut vaults_info: VaultsInfo = get_vaults_info(&env, &target_vault_key.denomination);

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&env, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        // If prev_key is None, the target Vault needs to be the lowest vault otherwise panic
        if prev_key == OptionalVaultKey::None && target_vault_key != lowest_key {
            panic_with_error!(&env, &SCErrors::PrevVaultCantBeNone);
        }

        withdraw_vault(&env, &target_vault, &prev_key);

        // If the target vault is the lowest, we update the lowest value
        if lowest_key == target_vault_key {
            vaults_info.lowest_key = target_vault.next_key.clone();
        }

        let new_vault_initial_debt: u128 = target_vault.total_debt.clone();
        let new_vault_collateral_amount: u128 = target_vault.total_collateral.clone() + collateral;
        let new_vault_key: VaultKey = VaultKey {
            index: calculate_user_vault_index(
                new_vault_initial_debt.clone(),
                new_vault_collateral_amount.clone(),
            ),
            account: target_vault.account,
            denomination: target_vault.denomination,
        };

        let (_, updated_target_vault_key, updated_target_vault_index_key, updated_lowest_key) =
            create_and_insert_vault(
                &env,
                &vaults_info.lowest_key,
                &new_vault_key,
                &new_prev_key,
                new_vault_initial_debt.clone(),
                new_vault_collateral_amount.clone(),
            );

        vaults_info.lowest_key = updated_lowest_key;
        vaults_info.total_col = vaults_info.total_col + collateral;
        set_vaults_info(&env, &vaults_info);

        bump_vault(&env, updated_target_vault_key);
        bump_vault_index(&env, updated_target_vault_index_key);
    }

    fn increase_debt(
        env: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    ) {
        bump_instance(&env);
        vault_key.account.require_auth();

        validate_currency(&env, &vault_key.denomination);
        is_currency_active(&env, &vault_key.denomination);

        let (target_vault, target_vault_key, _) =
            search_vault(&env, &vault_key.account, &vault_key.denomination);

        // TODO: Test this
        if target_vault.index != vault_key.index {
            panic_with_error!(&env, &SCErrors::IndexProvidedIsNotTheOneSaved);
        }

        // TODO: check if we are in panic mode once is implemented
        // TODO: check if collateral price has been updated lately

        let mut vaults_info: VaultsInfo = get_vaults_info(&env, &target_vault.denomination);

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&env, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        // If prev_key is None, the target Vault needs to be the lowest vault otherwise panic
        if prev_key == OptionalVaultKey::None && target_vault_key != lowest_key {
            panic_with_error!(&env, &SCErrors::PrevVaultCantBeNone);
        }

        withdraw_vault(&env, &target_vault, &prev_key);

        // If the target vault is the lowest, we update the lowest value
        if lowest_key == target_vault_key {
            vaults_info.lowest_key = target_vault.next_key.clone();
        }

        let core_state: CoreState = get_core_state(&env);

        let currency: Currency = get_currency(&env, &target_vault.denomination);

        let new_debt_amount: u128 = target_vault.total_debt + amount;

        let new_collateral_value: u128 = currency.rate * target_vault.total_collateral;

        let new_deposit_rate: u128 = div_floor(new_collateral_value, new_debt_amount);

        if new_deposit_rate < vaults_info.opening_col_rate {
            panic_with_error!(&env, SCErrors::CollateralRateUnderMinimum);
        }

        withdraw_stablecoin(
            &env,
            &core_state,
            &currency,
            &target_vault.account,
            amount as i128,
        );

        let new_vault_key: VaultKey = VaultKey {
            index: calculate_user_vault_index(
                new_debt_amount.clone(),
                target_vault.total_collateral.clone(),
            ),
            account: target_vault.account,
            denomination: target_vault.denomination,
        };

        let (_, updated_target_vault_key, updated_target_vault_index_key, updated_lowest_key) =
            create_and_insert_vault(
                &env,
                &vaults_info.lowest_key,
                &new_vault_key,
                &new_prev_key,
                new_debt_amount.clone(),
                target_vault.total_collateral.clone(),
            );

        vaults_info.lowest_key = updated_lowest_key;
        vaults_info.total_debt = vaults_info.total_debt + amount;
        set_vaults_info(&env, &vaults_info);

        bump_vault(&env, updated_target_vault_key);
        bump_vault_index(&env, updated_target_vault_index_key);
    }

    fn pay_debt(
        env: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    ) {
        bump_instance(&env);
        vault_key.account.require_auth();
        validate_currency(&env, &vault_key.denomination);
        is_currency_active(&env, &vault_key.denomination);

        let (target_vault, target_vault_key, _) =
            search_vault(&env, &vault_key.account, &vault_key.denomination);

        // TODO: Test this
        if target_vault.index != vault_key.index {
            panic_with_error!(&env, &SCErrors::IndexProvidedIsNotTheOneSaved);
        }

        let mut vaults_info: VaultsInfo = get_vaults_info(&env, &target_vault_key.denomination);

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&env, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        // If prev_key is None, the target Vault needs to be the lowest vault otherwise panic
        if prev_key == OptionalVaultKey::None && target_vault_key != lowest_key {
            panic_with_error!(&env, &SCErrors::PrevVaultCantBeNone);
        }

        let currency: Currency = get_currency(&env, &target_vault.denomination);

        if amount > target_vault.total_debt {
            panic_with_error!(&env, SCErrors::DepositAmountIsMoreThanTotalDebt);
        }

        let core_state: CoreState = get_core_state(&env);

        deposit_stablecoin(
            &env,
            &core_state,
            &currency,
            &target_vault.account,
            amount as i128,
        );

        if target_vault.total_debt == amount {
            // If the amount is equal to the debt it means it is paid in full so we release the collateral and remove the vault

            // If new_prev_key is not None, we panic because we are removing the vault
            if let OptionalVaultKey::Some(_) = new_prev_key {
                panic_with_error!(&env, &SCErrors::NextPrevVaultShouldBeNone);
            }

            vaults_info.total_vaults = vaults_info.total_vaults - 1;
            vaults_info.total_col = vaults_info.total_col - target_vault.total_collateral;

            let fee: u128 = calc_fee(&core_state.fee, &target_vault.total_collateral);

            withdraw_collateral(
                &env,
                &core_state,
                &target_vault.account,
                (target_vault.total_collateral - fee) as i128,
            );

            pay_fee(
                &env,
                &core_state,
                &env.current_contract_address(),
                fee as i128,
            );

            withdraw_vault(&env, &target_vault, &prev_key);

            // If the target vault is the lowest, we update the lowest value
            if lowest_key == target_vault_key {
                vaults_info.lowest_key = target_vault.next_key.clone();
            }
        } else {
            // If amount is not enough to pay all the debt, we check the debt value is not lower than the minimum and if is ok we just updated the stats of the user's vault
            let new_vault_debt: u128 = target_vault.total_debt - amount;
            if new_vault_debt < vaults_info.min_debt_creation {
                panic_with_error!(&env, &SCErrors::InvalidMinDebtAmount);
            }
            let new_vault_collateral: u128 = target_vault.total_collateral.clone();
            let new_vault_index: u128 =
                calculate_user_vault_index(new_vault_debt.clone(), new_vault_collateral.clone());

            withdraw_vault(&env, &target_vault, &prev_key);

            // If the target vault is the lowest, we update the lowest value
            if lowest_key == target_vault_key {
                vaults_info.lowest_key = target_vault.next_key.clone();
            }

            let (_, updated_target_vault_key, updated_target_vault_index_key, updated_lowest_key) =
                create_and_insert_vault(
                    &env,
                    &vaults_info.lowest_key,
                    &VaultKey {
                        index: new_vault_index.clone(),
                        account: target_vault.account.clone(),
                        denomination: target_vault.denomination.clone(),
                    },
                    &new_prev_key,
                    new_vault_debt.clone(),
                    new_vault_collateral.clone(),
                );

            vaults_info.lowest_key = updated_lowest_key;

            bump_vault(&env, updated_target_vault_key);
            bump_vault_index(&env, updated_target_vault_index_key);
        }

        vaults_info.total_debt = vaults_info.total_debt - amount;
        set_vaults_info(&env, &vaults_info);
    }

    fn redeem(env: Env, caller: Address, denomination: Symbol) {
        bump_instance(&env);
        caller.require_auth();

        validate_currency(&env, &denomination);
        is_currency_active(&env, &denomination);

        let core_state: CoreState = get_core_state(&env);
        let currency: Currency = get_currency(&env, &denomination);
        let mut vaults_info: VaultsInfo = get_vaults_info(&env, &denomination);

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&env, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        let lowest_vault: Vault = get_vault(&env, lowest_key);

        deposit_stablecoin(
            &env,
            &core_state,
            &currency,
            &caller,
            lowest_vault.total_debt as i128,
        );

        // Update the redeemable vaults information
        let fee: u128 = calc_fee(&core_state.fee, &lowest_vault.total_collateral);
        let collateral_to_withdraw: u128 =
            div_floor(lowest_vault.total_debt * 10000000, currency.rate) - fee;

        vaults_info.total_vaults = vaults_info.total_vaults - 1;
        vaults_info.total_col = vaults_info.total_col - lowest_vault.total_collateral;
        vaults_info.total_debt = vaults_info.total_debt - lowest_vault.total_debt;
        vaults_info.lowest_key = lowest_vault.next_key.clone();

        // We send the remaining collateral to the owner of the Vault
        withdraw_collateral(
            &env,
            &core_state,
            &lowest_vault.account,
            (lowest_vault.total_collateral - collateral_to_withdraw - fee) as i128,
        );

        withdraw_vault(&env, &lowest_vault, &OptionalVaultKey::None);

        withdraw_collateral(&env, &core_state, &caller, collateral_to_withdraw as i128);

        pay_fee(
            &env,
            &core_state,
            &env.current_contract_address(),
            fee as i128,
        );

        set_vaults_info(&env, &vaults_info);
    }

    fn liquidate(
        env: Env,
        liquidator: Address,
        denomination: Symbol,
        total_vaults_to_liquidate: u32,
    ) -> Vec<Vault> {
        bump_instance(&env);
        liquidator.require_auth();

        let core_state: CoreState = get_core_state(&env);
        let currency: Currency = get_currency(&env, &denomination);
        let mut vaults_info: VaultsInfo = get_vaults_info(&env, &denomination);
        let mut collateral_to_withdraw: u128 = 0;
        let mut amount_to_deposit: u128 = 0;
        let vaults_to_liquidate: Vec<Vault> = get_vaults(
            &env,
            &OptionalVaultKey::None,
            &currency,
            &vaults_info,
            total_vaults_to_liquidate,
            true,
        );

        if vaults_to_liquidate.len() < total_vaults_to_liquidate {
            panic_with_error!(&env, &SCErrors::NotEnoughVaultsToLiquidate);
        }

        for vault in vaults_to_liquidate.iter() {
            if !can_be_liquidated(&vault, &currency, &vaults_info) {
                panic_with_error!(&env, SCErrors::UserVaultCantBeLiquidated);
            }

            collateral_to_withdraw = collateral_to_withdraw + vault.total_collateral;
            amount_to_deposit = amount_to_deposit + vault.total_debt;

            vaults_info.total_vaults = vaults_info.total_vaults - 1;
            vaults_info.total_col = vaults_info.total_col - vault.total_collateral;
            vaults_info.total_debt = vaults_info.total_debt - vault.total_debt;

            withdraw_vault(&env, &vault, &OptionalVaultKey::None);

            vaults_info.lowest_key = vault.next_key;
        }

        set_vaults_info(&env, &vaults_info);
        deposit_stablecoin(
            &env,
            &core_state,
            &currency,
            &liquidator,
            amount_to_deposit as i128,
        );

        let end_collateral: u128 =
            collateral_to_withdraw - calc_fee(&core_state.fee, &collateral_to_withdraw);
        withdraw_collateral(&env, &core_state, &liquidator, end_collateral as i128);

        vaults_to_liquidate
    }
}
