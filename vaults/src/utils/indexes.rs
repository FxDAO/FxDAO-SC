use num_integer::div_floor;

pub fn calculate_user_vault_index(total_debt: u128, total_collateral: u128) -> u128 {
    div_floor(1_000000000 * total_collateral, total_debt)
}
