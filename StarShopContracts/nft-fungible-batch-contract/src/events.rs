use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

pub fn emit_contract_initialized(env: &Env, admin: &Address, name: &String, symbol: &String) {
    env.events().publish(
        (symbol_short!("INIT_EVT"), admin.clone()),
        (name.clone(), symbol.clone()),
    );
}

pub fn emit_nft_minted(env: &Env, recipient: &Address, token_id: u32) {
    env.events().publish(
        (symbol_short!("MINT"), recipient.clone()),
        token_id,
    );
}

pub fn emit_nft_batch_minted(
    env: &Env,
    minter: &Address,
    recipient: &Address,
    start_id: u32,
    quantity: u32,
) {
    env.events().publish(
        (symbol_short!("BATCHMNT"), minter.clone(), recipient.clone()),
        (start_id, quantity),
    );
}

pub fn emit_transfer(env: &Env, from: &Address, to: &Address, token_id: u32) {
    env.events().publish(
        (symbol_short!("TRANSFER"), from.clone(), to.clone()),
        token_id,
    );
}

pub fn emit_role_granted(env: &Env, role: Symbol, account: &Address, sender: &Address) {
    env.events().publish(
        (symbol_short!("ROLE_GRT"), role),
        (account.clone(), sender.clone()),
    );
}

pub fn emit_role_revoked(env: &Env, role: Symbol, account: &Address, sender: &Address) {
    env.events().publish(
        (symbol_short!("ROLE_REV"), role),
        (account.clone(), sender.clone()),
    );
}
