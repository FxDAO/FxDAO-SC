use crate::errors::SCErrors;
use crate::storage::vaults::{OptionalVaultKey, Vault, VaultKey};
use crate::utils::vaults::validate_prev_keys;
use soroban_sdk::{panic_with_error, Env, Vec};

pub fn assert_regular_vault_updates_validations(
    e: &Env,
    target_vault: &Vault,
    target_vault_key: &VaultKey,
    prev_key: &OptionalVaultKey,
    vault_key: &VaultKey,
    new_prev_key: &OptionalVaultKey,
    lowest_key: &VaultKey,
) {
    // We check that the prev_key denominations are the same of the target vault
    validate_prev_keys(
        &e,
        &vault_key,
        &Vec::from_array(&e, [prev_key.clone(), new_prev_key.clone()]),
    );

    // TODO: Test this
    if target_vault.index != vault_key.index {
        panic_with_error!(&e, &SCErrors::IndexProvidedIsNotTheOneSaved);
    }

    // If prev_key is None, the target Vault needs to be the lowest vault otherwise panic
    if prev_key == &OptionalVaultKey::None && target_vault_key != lowest_key {
        panic_with_error!(&e, &SCErrors::PrevVaultCantBeNone);
    }
}

pub fn assert_col_rate_under_min(
    e: &Env,
    rate_price: &i128,
    total_debt: &u128,
    total_collateral: &u128,
    opening_col_rate: &u128,
) {
    let new_collateral_value: u128 = (rate_price.clone() as u128) * total_collateral;

    let new_deposit_rate: u128 = new_collateral_value / total_debt;

    if &new_deposit_rate < opening_col_rate {
        panic_with_error!(e, SCErrors::CollateralRateUnderMinimum);
    }
}
