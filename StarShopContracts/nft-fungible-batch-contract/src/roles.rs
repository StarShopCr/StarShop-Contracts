use crate::events;
use crate::storage::{ContractError, DataKey, DEFAULT_ADMIN_ROLE};
use soroban_sdk::{panic_with_error, Address, Env, Symbol};

pub fn has_role(env: &Env, role: Symbol, account: &Address) -> bool {
    let key = DataKey::Role(role, account.clone());
    env.storage().persistent().get(&key).unwrap_or(false)
}

pub fn require_role(env: &Env, account: &Address, role: Symbol) {
    if !has_role(env, role, account) {
        panic_with_error!(env, ContractError::MissingRole);
    }
}

pub fn grant_role(env: &Env, role: Symbol, account: &Address, caller: &Address) {
    // Only DEFAULT_ADMIN_ROLE can grant roles
    caller.require_auth();
    require_role(env, caller, DEFAULT_ADMIN_ROLE);

    let key = DataKey::Role(role.clone(), account.clone());
    env.storage().persistent().set(&key, &true);

    events::emit_role_granted(env, role, account, caller);
}

pub fn revoke_role(env: &Env, role: Symbol, account: &Address, caller: &Address) {
    // Only DEFAULT_ADMIN_ROLE can revoke roles
    caller.require_auth();
    require_role(env, caller, DEFAULT_ADMIN_ROLE);

    let key = DataKey::Role(role.clone(), account.clone());
    env.storage().persistent().remove(&key);

    events::emit_role_revoked(env, role, account, caller);
}
