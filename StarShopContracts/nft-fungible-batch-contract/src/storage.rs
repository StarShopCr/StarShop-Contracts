//! Storage key management and state for Starshop NFT-Fungible Batch Distribution Contract

use soroban_sdk::{Env, Address, symbol_short};

// Initialization guard
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&symbol_short!("init"))
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&symbol_short!("init"), &true);
}

// Token ID counter (for unique NFT IDs)
pub fn next_token_id(env: &Env) -> u128 {
    let key = symbol_short!("token_id");
    let id: u128 = env.storage().instance().get(&key).unwrap_or(0);
    env.storage().instance().set(&key, &(id + 1));
    id + 1
}

// Set token owner
pub fn set_token_owner(env: &Env, token_id: u128, owner: &Address) {
    let key = (symbol_short!("owner"), token_id);
    env.storage().instance().set(&key, owner);
}

// Get token owner
pub fn get_token_owner(env: &Env, token_id: u128) -> Option<Address> {
    let key = (symbol_short!("owner"), token_id);
    env.storage().instance().get(&key)
}

// Set owner balance (number of NFTs owned)
pub fn set_owner_balance(env: &Env, owner: &Address, balance: u32) {
    let key = (symbol_short!("balance"), owner.clone());
    env.storage().instance().set(&key, &balance);
}

// Get owner balance
pub fn get_owner_balance(env: &Env, owner: &Address) -> u32 {
    let key = (symbol_short!("balance"), owner.clone());
    env.storage().instance().get(&key).unwrap_or(0)
}

// Increment total supply
pub fn increment_total_supply(env: &Env, amount: u32) {
    let key = symbol_short!("supply");
    let supply: u32 = env.storage().instance().get(&key).unwrap_or(0);
    env.storage().instance().set(&key, &(supply + amount));
}

// Get total supply
pub fn get_total_supply(env: &Env) -> u32 {
    let key = symbol_short!("supply");
    env.storage().instance().get(&key).unwrap_or(0)
}