use soroban_sdk::{Env, Symbol};

pub fn emit_contract_initialized(env: &Env) {
    env.events().publish(
        (Symbol::new(env, "contract"), Symbol::new(env, "initialized")),
        ()
    );
}

pub fn emit_nft_minted(env: &Env, token_id: u64, recipient: &Address) {
    env.events().publish(
        (Symbol::new(env, "nft"), Symbol::new(env, "minted")),
        (token_id, recipient.clone())
    );
}

pub fn emit_nft_batch_minted(env: &Env, quantity: u32, recipient: &Address) {
    env.events().publish(
        (Symbol::new(env, "nft"), Symbol::new(env, "batch_minted")),
        (quantity, recipient.clone())
    );
}

pub fn emit_transfer(env: &Env, from: &Address, to: &Address, token_id: u64) {
    env.events().publish(
        (Symbol::new(env, "nft"), Symbol::new(env, "transfer")),
        (from.clone(), to.clone(), token_id)
    );
}

pub fn emit_role_granted(env: &Env, role: &Symbol, account: &Address) {
    env.events().publish(
        (Symbol::new(env, "role"), Symbol::new(env, "granted")),
        (role.clone(), account.clone())
    );
}

pub fn emit_role_revoked(env: &Env, role: &Symbol, account: &Address) {
    env.events().publish(
        (Symbol::new(env, "role"), Symbol::new(env, "revoked")),
        (role.clone(), account.clone())
    );
}
