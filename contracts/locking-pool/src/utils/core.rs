use crate::errors::ContractErrors;
use crate::storage::core::{CoreDataKeys, CoreStorageFunc};
use soroban_sdk::{panic_with_error, Env};

pub fn validate(e: &Env, typ: CoreDataKeys) {
    match match typ {
        CoreDataKeys::Admin => e._core().address(&CoreDataKeys::Admin),
        CoreDataKeys::Manager => e._core().address(&CoreDataKeys::Manager),
        _ => None,
    } {
        None => panic_with_error!(&e, &ContractErrors::NotStarted),
        Some(v) => v.require_auth(),
    }
}
