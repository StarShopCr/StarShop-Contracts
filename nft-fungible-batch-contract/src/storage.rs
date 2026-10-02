use soroban_sdk::{Env, Symbol, Address, Bytes};

pub fn ensure_not_initialized(env: &Env) {
    if env.storage().persistent().has(&Symbol::new(env, "initialized")) {
        panic!("already initialized");
    }
}

pub fn set_initialized(env: &Env, _val: bool) {
    env.storage().persistent().set(&Symbol::new(env, "initialized"), &true);
}

pub fn get_token_counter(env: &Env) -> u64 {
    env.storage().persistent().get(&Symbol::new(env, "token_counter")).unwrap_or(0u64)
}

pub fn increment_token_counter(env: &Env) -> u64 {
    let current = get_token_counter(env);
    let next = current + 1;
    env.storage().persistent().set(&Symbol::new(env, "token_counter"), &next);
    next
}

fn token_key(env: &Env, token_id: u64) -> Bytes {
    let mut b = Bytes::from_slice(env, b"token:");
    b.append(&Bytes::from_slice(env, &token_id.to_be_bytes()));
    b
}

fn owner_key(env: &Env, owner: &Address, token_id: u64) -> Bytes {
    let mut b = Bytes::from_slice(env, b"owner:");
    b.append(&Bytes::from_slice(env, owner.as_ref()));
    b.append(&Bytes::from_slice(env, b":"));
    b.append(&Bytes::from_slice(env, &token_id.to_be_bytes()));
    b
}

pub fn set_token_owner(env: &Env, token_id: u64, owner: &Address) {
    let key = token_key(env, token_id);
    env.storage().persistent().set(&key, owner);
    let okey = owner_key(env, owner, token_id);
    env.storage().persistent().set(&okey, &true);
}

pub fn get_token_owner(env: &Env, token_id: u64) -> Option<Address> {
    let key = token_key(env, token_id);
    env.storage().persistent().get(&key)
}
