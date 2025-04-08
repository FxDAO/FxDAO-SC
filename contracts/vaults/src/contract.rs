use crate::errors::SCErrors;

use crate::storage::core::{CoreFunc, CoreState};
use crate::storage::currencies::{CurrenciesDataKeys, CurrenciesFunc, Currency};
use crate::storage::vaults::{
    OptionalVaultKey, Vault, VaultIndexKey, VaultKey, VaultsFunc, VaultsInfo,
};
use crate::utils::currencies::get_currency_rate;
use crate::utils::indexes::calculate_user_vault_index;
use crate::utils::payments::{
    burn_stablecoin, calc_fee, deposit_collateral, mint_stablecoin, pay_fee, withdraw_collateral,
};
use crate::utils::vaults::{
    calculate_deposit_ratio, can_be_liquidated, create_and_insert_vault, get_vaults, search_vault,
    validate_prev_keys, withdraw_vault,
};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, BytesN, Env, Symbol, Vec};

use crate::oracle::PriceData;
use crate::utils::validations::{
    assert_col_rate_under_min, assert_regular_vault_updates_validations,
};

// TODO: Explain each function here
pub trait VaultsContractTrait {
    fn init(
        e: Env,
        admin: Address,
        protocol_manager: Address,
        col_token: Address,
        stable_issuer: Address,
        treasury: Address,
        fee: u128,
        oracle: Address,
    );

    fn get_core_state(e: Env) -> CoreState;

    fn set_address(e: Env, typ: u32, address: Address);

    fn set_fee(e: Env, new_fee: u128);

    fn upgrade(e: Env, hash: BytesN<32>);
    fn set_panic(e: Env, status: bool);

    fn set_next_key(e: Env, target_key: VaultKey, next_key: OptionalVaultKey);

    // Currencies methods
    fn create_currency(e: Env, denomination: Symbol, contract: Address);
    fn get_currency(e: Env, denomination: Symbol) -> Currency;
    fn toggle_currency(e: Env, denomination: Symbol, active: bool);

    // Vaults methods
    fn set_vault_conditions(
        e: Env,
        min_col_rate: u128,
        min_debt_creation: u128,
        opening_col_rate: u128,
        denomination: Symbol,
    );
    fn get_vaults_info(e: Env, denomination: Symbol) -> VaultsInfo;
    fn calculate_deposit_ratio(currency_rate: u128, collateral: u128, debt: u128) -> u128;
    fn new_vault(
        e: Env,
        prev_key: OptionalVaultKey,
        caller: Address,
        initial_debt: u128,
        collateral_amount: u128,
        denomination: Symbol,
    );
    fn get_vault(e: Env, caller: Address, denomination: Symbol) -> Vault;
    fn get_vault_from_key(e: Env, vault_key: VaultKey) -> Vault;
    fn get_vaults(
        e: Env,
        prev_key: OptionalVaultKey,
        denomination: Symbol,
        total: u32,
        only_to_liquidate: bool,
    ) -> Vec<Vault>;
    fn increase_collateral(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    );
    fn withdraw_collateral(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    );
    fn increase_debt(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    );
    fn pay_debt(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    );
    fn transfer_debt(e: Env, prev_key: OptionalVaultKey, vault_key: VaultKey, destination: Address);

    // Redeeming
    // fn redeem(
    //     e: Env,
    //     caller: Address,
    //     denomination: Symbol,
    //     new_prev_key: OptionalVaultKey,
    //     amount: u128,
    // );

    // Liquidation
    fn liquidate(
        e: Env,
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
        e: Env,
        admin: Address,
        protocol_manager: Address,
        col_token: Address,
        stable_issuer: Address,
        treasury: Address,
        fee: u128,
        oracle: Address,
    ) {
        e.bump_instance();
        if e.core_state().is_some() {
            panic_with_error!(&e, &SCErrors::CoreAlreadySet);
        }

        // The protocol should not have a fee higher than 1%
        if fee > 100000 {
            panic_with_error!(&e, &SCErrors::InvalidFee);
        }

        e.set_core_state(&CoreState {
            col_token,
            stable_issuer,
            admin,
            protocol_manager,
            panic_mode: false,
            treasury,
            fee,
            oracle,
        });
    }

    fn get_core_state(e: Env) -> CoreState {
        e.bump_instance();
        e.core_state().unwrap()
    }

    fn set_address(e: Env, typ: u32, address: Address) {
        e.bump_instance();
        let mut core_state: CoreState = e.core_state().unwrap();

        if typ == 0 {
            core_state.admin.require_auth();
            core_state.admin = address;
        } else if typ == 1 {
            core_state.protocol_manager.require_auth();
            core_state.protocol_manager = address;
        } else if typ == 2 {
            core_state.protocol_manager.require_auth();
            core_state.oracle = address;
        } else {
            panic!();
        }

        e.set_core_state(&core_state);
    }

    fn set_fee(e: Env, new_fee: u128) {
        e.bump_instance();
        let mut core_state: CoreState = e.core_state().unwrap();
        core_state.admin.require_auth();
        core_state.fee = new_fee;
        e.set_core_state(&core_state);
    }

    fn upgrade(e: Env, hash: BytesN<32>) {
        e.bump_instance();
        e.core_state().unwrap().admin.require_auth();
        e.deployer().update_current_contract_wasm(hash);
    }

    fn set_panic(e: Env, status: bool) {
        e.bump_instance();
        e.core_state().unwrap().protocol_manager.require_auth();
        let mut core_state: CoreState = e.core_state().unwrap();
        core_state.panic_mode = status;
        e.set_core_state(&core_state);
    }

    // This is a management function, make sure the next key is correct before setting it.
    fn set_next_key(e: Env, target_key: VaultKey, next_key: OptionalVaultKey) {
        e.bump_instance();
        e.core_state().unwrap().protocol_manager.require_auth();

        validate_prev_keys(&e, &target_key, &Vec::from_array(&e, [next_key.clone()]));

        let mut target_vault: Vault = e.vault(&target_key).unwrap();
        target_vault.next_key = next_key;

        e.set_vault(&target_vault);
        e.bump_vault(&target_key);
        e.bump_vault_index(&VaultIndexKey {
            user: target_key.account.clone(),
            denomination: target_key.denomination.clone(),
        });
    }

    fn create_currency(e: Env, denomination: Symbol, contract: Address) {
        e.bump_instance();
        e.core_state().unwrap().protocol_manager.require_auth();

        if e.storage()
            .instance()
            .has(&CurrenciesDataKeys::Currency(denomination.clone()))
        {
            panic_with_error!(&e, &SCErrors::CurrencyAlreadyAdded);
        }

        e.set_currency(&Currency {
            denomination,
            active: false,
            contract,
        });
    }

    fn get_currency(e: Env, denomination: Symbol) -> Currency {
        e.bump_instance();
        e.currency(&denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist))
    }

    fn toggle_currency(e: Env, denomination: Symbol, active: bool) {
        e.bump_instance();
        e.core_state().unwrap().admin.require_auth();
        let mut currency: Currency = e
            .currency(&denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist));

        currency.active = active;
        e.set_currency(&currency);
    }

    fn set_vault_conditions(
        e: Env,
        min_col_rate: u128,
        min_debt_creation: u128,
        opening_col_rate: u128,
        denomination: Symbol,
    ) {
        e.bump_instance();
        e.core_state().unwrap().admin.require_auth();

        if opening_col_rate <= min_col_rate {
            panic_with_error!(&e, &SCErrors::InvalidOpeningCollateralRatio);
        }

        match e.vaults_info(&denomination) {
            None => {
                e.set_vaults_info(&VaultsInfo {
                    denomination,
                    min_col_rate,
                    min_debt_creation,
                    opening_col_rate,
                    total_vaults: 0,
                    total_col: 0,
                    total_debt: 0,
                    lowest_key: OptionalVaultKey::None,
                });
            }
            Some(vaults_info) => {
                e.set_vaults_info(&VaultsInfo {
                    denomination,
                    min_col_rate,
                    min_debt_creation,
                    opening_col_rate,
                    total_vaults: vaults_info.total_vaults,
                    total_col: vaults_info.total_col,
                    total_debt: vaults_info.total_debt,
                    lowest_key: vaults_info.lowest_key,
                });
            }
        }
    }

    fn get_vaults_info(e: Env, denomination: Symbol) -> VaultsInfo {
        e.bump_instance();
        e.vaults_info(&denomination).unwrap()
    }

    fn calculate_deposit_ratio(currency_rate: u128, collateral: u128, debt: u128) -> u128 {
        calculate_deposit_ratio(&currency_rate, &collateral, &debt)
    }

    fn new_vault(
        e: Env,
        prev_key: OptionalVaultKey,
        caller: Address,
        initial_debt: u128,
        collateral_amount: u128,
        denomination: Symbol,
    ) {
        e.bump_instance();
        caller.require_auth();
        let currency: Currency = e
            .currency(&denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist));

        if !currency.active {
            panic_with_error!(&e, &SCErrors::CurrencyIsInactive);
        }

        if e.vault_index(&VaultIndexKey {
            user: caller.clone(),
            denomination: denomination.clone(),
        })
        .is_some()
        {
            panic_with_error!(&e, &SCErrors::UserAlreadyHasDenominationVault);
        }

        let core_state: CoreState = e.core_state().unwrap();

        let rate: PriceData = get_currency_rate(&e, &core_state, &denomination);

        // If price of the collateral hasn't been updated in more than 20 minutes or the protocol is in panic mode we throw
        if core_state.panic_mode || rate.timestamp < e.ledger().timestamp().saturating_sub(1200) {
            panic_with_error!(&e, &SCErrors::PanicModeEnabled);
        }

        let fee: u128 = calc_fee(&core_state.fee, &collateral_amount);
        let vault_col: u128 = collateral_amount - fee;

        let mut vaults_info: VaultsInfo = e
            .vaults_info(&denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::VaultsInfoHasNotStarted));

        if vaults_info.min_debt_creation > initial_debt {
            panic_with_error!(e, &SCErrors::InvalidMinDebtAmount);
        }

        let deposit_collateral_rate: u128 =
            calculate_deposit_ratio(&(rate.price as u128), &vault_col, &initial_debt);

        if deposit_collateral_rate < vaults_info.opening_col_rate {
            panic_with_error!(&e, &SCErrors::InvalidOpeningCollateralRatio);
        }

        let new_vault_index: u128 = calculate_user_vault_index(initial_debt, vault_col);
        let new_vault_key: VaultKey = VaultKey {
            index: new_vault_index.clone(),
            account: caller.clone(),
            denomination: denomination.clone(),
        };

        // In case prev value is not None, we confirm it exists and its index is not higher than the new Vault index
        // We also check that the prev_key uses the same denomination to prevent people sending a prev_key from another denomination
        match prev_key.clone() {
            OptionalVaultKey::None => {}
            OptionalVaultKey::Some(value) => {
                if new_vault_index < value.index {
                    panic_with_error!(&e, &SCErrors::InvalidPrevVaultIndex);
                }

                if e.vault(&value).is_none() {
                    panic_with_error!(&e, &SCErrors::PrevVaultDoesntExist);
                }

                if value.denomination != denomination {
                    panic_with_error!(&e, &SCErrors::InvalidPrevKeyDenomination);
                }
            }
        }

        let (_, new_vault_key, new_vault_index_key, updated_lowest_key) = create_and_insert_vault(
            &e,
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
        e.set_vaults_info(&vaults_info);

        deposit_collateral(&e, &core_state, &caller, vault_col as i128);
        mint_stablecoin(&e, &currency, &caller, initial_debt as i128);
        pay_fee(&e, &core_state, &caller, fee as i128);

        e.bump_vault(&new_vault_key);
        e.bump_vault_index(&new_vault_index_key);
    }

    fn get_vault(e: Env, user: Address, denomination: Symbol) -> Vault {
        e.bump_instance();

        let (user_vault, vault_key, vault_index_key) = search_vault(&e, &user, &denomination);

        e.bump_vault(&vault_key);
        e.bump_vault_index(&vault_index_key);

        user_vault
    }

    fn get_vault_from_key(e: Env, vault_key: VaultKey) -> Vault {
        e.bump_instance();

        if e.vault(&vault_key).is_none() {
            panic_with_error!(&e, SCErrors::VaultDoesntExist);
        }

        let vault_index_key: VaultIndexKey = VaultIndexKey {
            user: vault_key.account.clone(),
            denomination: vault_key.denomination.clone(),
        };

        e.bump_vault(&vault_key.clone());
        e.bump_vault_index(&vault_index_key);

        e.vault(&vault_key).unwrap()
    }

    fn get_vaults(
        e: Env,
        prev_key: OptionalVaultKey,
        denomination: Symbol,
        total: u32,
        only_to_liquidate: bool,
    ) -> Vec<Vault> {
        e.bump_instance();

        let core_state: CoreState = e.core_state().unwrap();
        let rate: PriceData = get_currency_rate(&e, &core_state, &denomination);
        let vaults_info: VaultsInfo = e.vaults_info(&denomination).unwrap();

        if OptionalVaultKey::None == prev_key && OptionalVaultKey::None == vaults_info.lowest_key {
            return Vec::new(&e);
        }

        if let OptionalVaultKey::Some(key) = &prev_key {
            if key.denomination != denomination {
                panic_with_error!(&e, &SCErrors::InvalidPrevKeyDenomination);
            }
        } else if OptionalVaultKey::None == vaults_info.lowest_key {
            return Vec::new(&e);
        }

        get_vaults(
            &e,
            &prev_key,
            &vaults_info,
            total,
            only_to_liquidate,
            rate.price as u128,
        )
    }

    fn increase_collateral(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    ) {
        e.bump_instance();
        vault_key.account.require_auth();

        let currency: Currency = e
            .currency(&vault_key.denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist));

        if !currency.active {
            panic_with_error!(&e, &SCErrors::CurrencyIsInactive);
        }

        let core_state: CoreState = e.core_state().unwrap();

        if amount < (core_state.fee * 10) {
            panic_with_error!(&e, &SCErrors::InvalidMinCollateralAmount);
        }

        let fee: u128 = calc_fee(&core_state.fee, &amount);
        let collateral: u128 = amount - fee;

        deposit_collateral(&e, &core_state, &vault_key.account, collateral as i128);
        pay_fee(&e, &core_state, &vault_key.account, fee as i128);

        let (target_vault, target_vault_key, _) =
            search_vault(&e, &vault_key.account, &vault_key.denomination);

        let mut vaults_info: VaultsInfo = e.vaults_info(&target_vault_key.denomination).unwrap();

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&e, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        assert_regular_vault_updates_validations(
            &e,
            &target_vault,
            &target_vault_key,
            &prev_key,
            &vault_key,
            &new_prev_key,
            &lowest_key,
        );

        withdraw_vault(&e, &target_vault, &prev_key);

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
                &e,
                &vaults_info.lowest_key,
                &new_vault_key,
                &new_prev_key,
                new_vault_initial_debt.clone(),
                new_vault_collateral_amount.clone(),
            );

        vaults_info.lowest_key = updated_lowest_key;
        vaults_info.total_col = vaults_info.total_col + collateral;
        e.set_vaults_info(&vaults_info);

        e.bump_vault(&updated_target_vault_key);
        e.bump_vault_index(&updated_target_vault_index_key);
    }

    fn withdraw_collateral(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    ) {
        e.bump_instance();
        vault_key.account.require_auth();

        let currency: Currency = e
            .currency(&vault_key.denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist));

        if !currency.active {
            panic_with_error!(&e, &SCErrors::CurrencyIsInactive);
        }

        let (target_vault, target_vault_key, _) =
            search_vault(&e, &vault_key.account, &vault_key.denomination);

        let core_state: CoreState = e.core_state().unwrap();

        let rate: PriceData = get_currency_rate(&e, &core_state, &target_vault.denomination);

        // If price of the collateral hasn't been updated in more than 20 minutes or the protocol is in panic mode we throw
        if core_state.panic_mode || rate.timestamp < e.ledger().timestamp().saturating_sub(1200) {
            panic_with_error!(&e, &SCErrors::PanicModeEnabled);
        }

        let mut vaults_info: VaultsInfo = e.vaults_info(&target_vault.denomination).unwrap();

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&e, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        assert_regular_vault_updates_validations(
            &e,
            &target_vault,
            &target_vault_key,
            &prev_key,
            &vault_key,
            &new_prev_key,
            &lowest_key,
        );

        withdraw_vault(&e, &target_vault, &prev_key);

        // If the target vault is the lowest, we update the lowest value
        if lowest_key == target_vault_key {
            vaults_info.lowest_key = target_vault.next_key.clone();
        }

        let new_collateral_amount: u128 = target_vault.total_collateral.saturating_sub(amount);

        assert_col_rate_under_min(
            &e,
            &rate.price,
            &target_vault.total_debt,
            &new_collateral_amount,
            &vaults_info.opening_col_rate,
        );

        // We send the remaining collateral to the owner of the Vault
        withdraw_collateral(&e, &core_state, &vault_key.account, amount as i128);

        let new_vault_key: VaultKey = VaultKey {
            index: calculate_user_vault_index(
                target_vault.total_debt.clone(),
                new_collateral_amount.clone(),
            ),
            account: target_vault.account,
            denomination: target_vault.denomination,
        };

        let (_, updated_target_vault_key, updated_target_vault_index_key, updated_lowest_key) =
            create_and_insert_vault(
                &e,
                &vaults_info.lowest_key,
                &new_vault_key,
                &new_prev_key,
                target_vault.total_debt.clone(),
                new_collateral_amount.clone(),
            );

        vaults_info.lowest_key = updated_lowest_key;
        vaults_info.total_col = vaults_info.total_col - amount;
        e.set_vaults_info(&vaults_info);

        e.bump_vault(&updated_target_vault_key);
        e.bump_vault_index(&updated_target_vault_index_key);
    }

    fn increase_debt(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    ) {
        e.bump_instance();
        vault_key.account.require_auth();

        let currency: Currency = e
            .currency(&vault_key.denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist));

        if !currency.active {
            panic_with_error!(&e, &SCErrors::CurrencyIsInactive);
        }

        let (target_vault, target_vault_key, _) =
            search_vault(&e, &vault_key.account, &vault_key.denomination);

        let core_state: CoreState = e.core_state().unwrap();

        let rate: PriceData = get_currency_rate(&e, &core_state, &target_vault.denomination);

        // If price of the collateral hasn't been updated in more than 20 minutes or the protocol is in panic mode we throw
        if core_state.panic_mode || rate.timestamp < e.ledger().timestamp().saturating_sub(1200) {
            panic_with_error!(&e, &SCErrors::PanicModeEnabled);
        }

        let mut vaults_info: VaultsInfo = e.vaults_info(&target_vault.denomination).unwrap();

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&e, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        assert_regular_vault_updates_validations(
            &e,
            &target_vault,
            &target_vault_key,
            &prev_key,
            &vault_key,
            &new_prev_key,
            &lowest_key,
        );

        withdraw_vault(&e, &target_vault, &prev_key);

        // If the target vault is the lowest, we update the lowest value
        if lowest_key == target_vault_key {
            vaults_info.lowest_key = target_vault.next_key.clone();
        }

        let new_debt_amount: u128 = target_vault.total_debt + amount;

        assert_col_rate_under_min(
            &e,
            &rate.price,
            &new_debt_amount,
            &target_vault.total_collateral,
            &vaults_info.opening_col_rate,
        );

        mint_stablecoin(&e, &currency, &target_vault.account, amount as i128);

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
                &e,
                &vaults_info.lowest_key,
                &new_vault_key,
                &new_prev_key,
                new_debt_amount.clone(),
                target_vault.total_collateral.clone(),
            );

        vaults_info.lowest_key = updated_lowest_key;
        vaults_info.total_debt = vaults_info.total_debt + amount;
        e.set_vaults_info(&vaults_info);

        e.bump_vault(&updated_target_vault_key);
        e.bump_vault_index(&updated_target_vault_index_key);
    }

    fn pay_debt(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        new_prev_key: OptionalVaultKey,
        amount: u128,
    ) {
        e.bump_instance();
        vault_key.account.require_auth();

        let currency: Currency = e
            .currency(&vault_key.denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist));

        let (target_vault, target_vault_key, _) =
            search_vault(&e, &vault_key.account, &vault_key.denomination);

        let mut vaults_info: VaultsInfo = e.vaults_info(&target_vault_key.denomination).unwrap();

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&e, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        assert_regular_vault_updates_validations(
            &e,
            &target_vault,
            &target_vault_key,
            &prev_key,
            &vault_key,
            &new_prev_key,
            &lowest_key,
        );

        if amount > target_vault.total_debt {
            panic_with_error!(&e, SCErrors::DepositAmountIsMoreThanTotalDebt);
        }

        let core_state: CoreState = e.core_state().unwrap();

        burn_stablecoin(&e, &currency, &target_vault.account, amount as i128);

        if target_vault.total_debt == amount {
            // If the amount is equal to the debt it means it is paid in full, so we release the collateral and remove the vault

            // If new_prev_key is not None, we panic because we are removing the vault
            if let OptionalVaultKey::Some(_) = new_prev_key {
                panic_with_error!(&e, &SCErrors::NextPrevVaultShouldBeNone);
            }

            vaults_info.total_vaults = vaults_info.total_vaults - 1;
            vaults_info.total_col = vaults_info.total_col - target_vault.total_collateral;

            let fee: u128 = calc_fee(&core_state.fee, &target_vault.total_collateral);

            withdraw_collateral(
                &e,
                &core_state,
                &target_vault.account,
                (target_vault.total_collateral - fee) as i128,
            );

            pay_fee(&e, &core_state, &e.current_contract_address(), fee as i128);

            withdraw_vault(&e, &target_vault, &prev_key);

            // If the target vault is the lowest, we update the lowest value
            if lowest_key == target_vault_key {
                vaults_info.lowest_key = target_vault.next_key.clone();
            }
        } else {
            // If amount is not enough to pay all the debt, we check the debt value is not lower than the minimum and if is ok we just updated the stats of the user's vault
            let new_vault_debt: u128 = target_vault.total_debt - amount;
            if new_vault_debt < vaults_info.min_debt_creation {
                panic_with_error!(&e, &SCErrors::InvalidMinDebtAmount);
            }
            let new_vault_collateral: u128 = target_vault.total_collateral.clone();
            let new_vault_index: u128 =
                calculate_user_vault_index(new_vault_debt.clone(), new_vault_collateral.clone());

            withdraw_vault(&e, &target_vault, &prev_key);

            // If the target vault is the lowest, we update the lowest value
            if lowest_key == target_vault_key {
                vaults_info.lowest_key = target_vault.next_key.clone();
            }

            let (_, updated_target_vault_key, updated_target_vault_index_key, updated_lowest_key) =
                create_and_insert_vault(
                    &e,
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

            e.bump_vault(&updated_target_vault_key);
            e.bump_vault_index(&updated_target_vault_index_key);
        }

        vaults_info.total_debt = vaults_info.total_debt - amount;
        e.set_vaults_info(&vaults_info);
    }

    fn transfer_debt(
        e: Env,
        prev_key: OptionalVaultKey,
        vault_key: VaultKey,
        destination: Address,
    ) {
        e.bump_instance();
        vault_key.account.require_auth();

        let (mut target_vault, mut target_vault_key, _) =
            search_vault(&e, &vault_key.account, &vault_key.denomination);

        let vaults_info: VaultsInfo = e.vaults_info(&target_vault_key.denomination).unwrap();

        let lowest_key = match vaults_info.lowest_key.clone() {
            // It should be impossible to reach this case, but just in case we panic if it happens.
            OptionalVaultKey::None => panic_with_error!(&e, &SCErrors::ThereAreNoVaults),
            OptionalVaultKey::Some(key) => key,
        };

        assert_regular_vault_updates_validations(
            &e,
            &target_vault,
            &target_vault_key,
            &prev_key,
            &vault_key,
            &prev_key,
            &lowest_key,
        );

        // We remove the vault so we can update it to the new owner
        withdraw_vault(&e, &target_vault, &prev_key);

        target_vault.account = destination.clone();
        target_vault_key.account = destination;

        let (_, updated_target_vault_key, updated_target_vault_index_key, updated_lowest_key) =
            create_and_insert_vault(
                &e,
                &vaults_info.lowest_key,
                &target_vault_key,
                &prev_key,
                target_vault.total_debt.clone(),
                target_vault.total_collateral.clone(),
            );

        e.bump_vault(&updated_target_vault_key);
        e.bump_vault_index(&updated_target_vault_index_key);
    }

    // fn redeem(
    //     e: Env,
    //     caller: Address,
    //     denomination: Symbol,
    //     new_prev_key: OptionalVaultKey,
    //     amount: u128,
    // ) {
    //     e.bump_instance();
    //     caller.require_auth();
    //
    //     let currency: Currency = e
    //         .currency(&denomination)
    //         .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist));
    //
    //     if !currency.active {
    //         panic_with_error!(&e, &SCErrors::CurrencyIsInactive);
    //     }
    //
    //     let core_state: CoreState = e.core_state().unwrap();
    //     let rate: PriceData = get_currency_rate(&e, &core_state, &denomination);
    //     let mut vaults_info: VaultsInfo = e.vaults_info(&denomination).unwrap();
    //
    //     let lowest_key = match vaults_info.lowest_key.clone() {
    //         // It should be impossible to reach this case, but just in case we panic if it happens.
    //         OptionalVaultKey::None => panic_with_error!(&e, &SCErrors::ThereAreNoVaults),
    //         OptionalVaultKey::Some(key) => key,
    //     };
    //
    //     validate_prev_keys(
    //         &e,
    //         &lowest_key,
    //         &Vec::from_array(&e, [new_prev_key.clone()]),
    //     );
    //
    //     let lowest_vault: Vault = e.vault(&lowest_key).unwrap();
    //
    //     if amount > lowest_vault.total_debt {
    //         panic_with_error!(&e, SCErrors::DepositAmountIsMoreThanTotalDebt);
    //     }
    //
    //     burn_stablecoin(&e, &currency, &caller, amount as i128);
    //
    //     // We withdraw the caller redeemed collateral and pay the fee to the protocol
    //     let collateral_to_redeem: u128 = (amount * 10000000) / (rate.price as u128);
    //
    //     // If the redeem amount is the same as the vault total debt, it gets the lower fee
    //     // If the amount is lower than the full debt amount then the fee is 75 times higher
    //     // This is done to prevent 'burning only arbitragers'. This type of arbitrager should do it with its own vault instead.
    //     let fee: u128 = if amount == lowest_vault.total_debt {
    //         calc_fee(&core_state.fee, &collateral_to_redeem)
    //     } else {
    //         calc_fee(&core_state.fee, &collateral_to_redeem) * 75
    //     };
    //     let collateral_to_withdraw: u128 = collateral_to_redeem - fee;
    //
    //     withdraw_collateral(&e, &core_state, &caller, collateral_to_withdraw as i128);
    //     pay_fee(&e, &core_state, &e.current_contract_address(), fee as i128);
    //
    //     vaults_info.total_col = vaults_info.total_col - collateral_to_redeem;
    //     vaults_info.total_debt = vaults_info.total_debt - amount;
    //
    //     // If the amount is equal to the debt it means it is a full redeem, so we release the collateral and remove the vault
    //     if amount == lowest_vault.total_debt {
    //         // If new_prev_key is not None, we panic because we are removing the vault
    //         if let OptionalVaultKey::Some(_) = new_prev_key {
    //             panic_with_error!(&e, &SCErrors::NextPrevVaultShouldBeNone);
    //         }
    //
    //         withdraw_vault(&e, &lowest_vault, &OptionalVaultKey::None);
    //         vaults_info.total_vaults = vaults_info.total_vaults - 1;
    //         vaults_info.lowest_key = lowest_vault.next_key.clone();
    //
    //         let extra_fee: u128 = calc_fee(
    //             &core_state.fee,
    //             &(lowest_vault.total_collateral - collateral_to_redeem),
    //         );
    //
    //         // We update the total_col again because we are reducing the collateral in the remaining of the collateral from the removed vault
    //         vaults_info.total_col =
    //             vaults_info.total_col - (lowest_vault.total_collateral - collateral_to_redeem);
    //
    //         // We send the remaining collateral to the owner of the Vault and pay the fee
    //         withdraw_collateral(
    //             &e,
    //             &core_state,
    //             &lowest_vault.account,
    //             (lowest_vault.total_collateral - collateral_to_redeem - extra_fee) as i128,
    //         );
    //         pay_fee(
    //             &e,
    //             &core_state,
    //             &e.current_contract_address(),
    //             extra_fee as i128,
    //         );
    //     } else {
    //         // If amount is not enough to pay all the debt, we check the debt value is not lower than the minimum and if is ok we just updated the stats of the user's vault
    //         let new_vault_debt: u128 = lowest_vault.total_debt - amount;
    //         if new_vault_debt < vaults_info.min_debt_creation {
    //             panic_with_error!(&e, &SCErrors::InvalidMinDebtAmount);
    //         }
    //         let new_vault_collateral: u128 = lowest_vault.total_collateral - collateral_to_redeem;
    //         let new_vault_index: u128 =
    //             calculate_user_vault_index(new_vault_debt.clone(), new_vault_collateral.clone());
    //
    //         // In theory the collateral rate should not go down
    //         // But we still check the col rate is not under min ratio
    //         assert_col_rate_under_min(
    //             &e,
    //             &rate.price,
    //             &new_vault_debt,
    //             &new_vault_collateral,
    //             &vaults_info.min_col_rate,
    //         );
    //
    //         withdraw_vault(&e, &lowest_vault, &OptionalVaultKey::None);
    //
    //         // We are working with the lowest vault so we update the lowest value
    //         vaults_info.lowest_key = lowest_vault.next_key.clone();
    //
    //         let (_, updated_target_vault_key, updated_target_vault_index_key, updated_lowest_key) =
    //             create_and_insert_vault(
    //                 &e,
    //                 &vaults_info.lowest_key,
    //                 &VaultKey {
    //                     index: new_vault_index.clone(),
    //                     account: lowest_vault.account.clone(),
    //                     denomination: lowest_vault.denomination.clone(),
    //                 },
    //                 &new_prev_key,
    //                 new_vault_debt.clone(),
    //                 new_vault_collateral.clone(),
    //             );
    //
    //         vaults_info.lowest_key = updated_lowest_key;
    //
    //         e.bump_vault(&updated_target_vault_key);
    //         e.bump_vault_index(&updated_target_vault_index_key);
    //     }
    //
    //     e.set_vaults_info(&vaults_info);
    // }

    fn liquidate(
        e: Env,
        liquidator: Address,
        denomination: Symbol,
        total_vaults_to_liquidate: u32,
    ) -> Vec<Vault> {
        e.bump_instance();
        liquidator.require_auth();

        let core_state: CoreState = e.core_state().unwrap();
        let rate: PriceData = get_currency_rate(&e, &core_state, &denomination);
        let currency: Currency = e
            .currency(&denomination)
            .unwrap_or_else(|| panic_with_error!(&e, &SCErrors::CurrencyDoesntExist));
        let mut vaults_info: VaultsInfo = e.vaults_info(&denomination).unwrap();
        let mut collateral_to_withdraw: u128 = 0;
        let mut amount_to_deposit: u128 = 0;
        let vaults_to_liquidate: Vec<Vault> = get_vaults(
            &e,
            &OptionalVaultKey::None,
            &vaults_info,
            total_vaults_to_liquidate,
            true,
            rate.price as u128,
        );

        if vaults_to_liquidate.len() < total_vaults_to_liquidate {
            panic_with_error!(&e, &SCErrors::NotEnoughVaultsToLiquidate);
        }

        for vault in vaults_to_liquidate.iter() {
            if !can_be_liquidated(&vault, &vaults_info, &(rate.price as u128)) {
                panic_with_error!(&e, SCErrors::UserVaultCantBeLiquidated);
            }

            collateral_to_withdraw = collateral_to_withdraw + vault.total_collateral;
            amount_to_deposit = amount_to_deposit + vault.total_debt;

            vaults_info.total_vaults = vaults_info.total_vaults - 1;
            vaults_info.total_col = vaults_info.total_col - vault.total_collateral;
            vaults_info.total_debt = vaults_info.total_debt - vault.total_debt;

            withdraw_vault(&e, &vault, &OptionalVaultKey::None);

            vaults_info.lowest_key = vault.next_key;
        }

        e.set_vaults_info(&vaults_info);
        burn_stablecoin(&e, &currency, &liquidator, amount_to_deposit as i128);

        let fee: u128 = calc_fee(&core_state.fee, &collateral_to_withdraw);
        let end_collateral: u128 = collateral_to_withdraw - fee;
        withdraw_collateral(&e, &core_state, &liquidator, end_collateral as i128);
        pay_fee(&e, &core_state, &e.current_contract_address(), fee as i128);

        vaults_to_liquidate
    }
}
