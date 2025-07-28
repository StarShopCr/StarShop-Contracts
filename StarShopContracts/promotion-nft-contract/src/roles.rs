use soroban_sdk::{Address, Env, Symbol, panic_with_error};
use crate::storage;

pub const DEFAULT_ADMIN_ROLE: u32 = 0;
pub const MINTER_ROLE: u32 = 1;
pub const REDEEMER_ROLE: u32 = 2;


pub fn require_role(env: &Env, role: Symbol) {
    let caller = env.current_contract_address();
    if !has_role(env, role, &caller) {
        panic_with_error!(env, storage::ContractError::MissingRole);
    }
}

pub fn has_role(env: &Env, role: Symbol, account: &Address) -> bool {
    let role_key = (role, account.clone());
    env.storage().persistent().get(&role_key).unwrap_or(false)
}

pub fn grant_role(env: &Env, role: Symbol, account: &Address) {
    // Only admin can grant roles
    require_role(env, storage::DEFAULT_ADMIN_ROLE);

    let role_key = (role.clone(), account.clone());
    env.storage().persistent().set(&role_key, &true);

    env.events().publish(
        (storage::ROLE_GRANTED_EVENT,),
        (role, account.clone())
    );
}

pub fn revoke_role(env: &Env, role: Symbol, account: &Address) {
    // Only admin can revoke roles
    require_role(env, storage::DEFAULT_ADMIN_ROLE);

    let role_key = (role.clone(), account.clone());
    env.storage().persistent().remove(&role_key);

    env.events().publish(
        (storage::ROLE_REVOKED_EVENT,),
        (role, account.clone())
    );
}
