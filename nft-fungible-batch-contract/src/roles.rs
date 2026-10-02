use soroban_sdk::{Env, Address, Symbol, Bytes};
use crate::events;

pub fn default_admin_role(env: &Env) -> Symbol {
    Symbol::new(env, "DEFAULT_ADMIN_ROLE")
}

pub fn minter_role(env: &Env) -> Symbol {
    Symbol::new(env, "MINTER_ROLE")
}

fn role_key(env: &Env, role: &Symbol, account: &Address) -> Bytes {
    let mut b = Bytes::from_slice(env, b"role:");
    b.append(&Bytes::from_slice(env, role.as_ref()));
    b.append(&Bytes::from_slice(env, b":"));
    b.append(&Bytes::from_slice(env, account.as_ref()));
    b
}

pub fn has_role(env: &Env, role: &Symbol, account: &Address) -> bool {
    let key = role_key(env, role, account);
    env.storage().persistent().has(&key)
}

pub fn grant_role(env: &Env, role: &Symbol, account: &Address) {
    let key = role_key(env, role, account);
    env.storage().persistent().set(&key, &true);
    events::emit_role_granted(env, role, account);
}

pub fn revoke_role(env: &Env, role: &Symbol, account: &Address) {
    let key = role_key(env, role, account);
    env.storage().persistent().remove(&key);
    events::emit_role_revoked(env, role, account);
}

pub fn require_role(env: &Env, account: &Address, role: &Symbol) {
    if !has_role(env, role, account) {
        panic!("missing role");
    }
}

pub fn require_admin(env: &Env, account: &Address) {
    require_role(env, account, &default_admin_role(env));
}

pub fn require_minter(env: &Env, account: &Address) {
    require_role(env, account, &minter_role(env));
}

pub fn setup_admin(env: &Env, admin: &Address) {
    grant_role(env, &default_admin_role(env), admin);
}
