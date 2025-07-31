//! Storage key management and state for Starshop NFT-Fungible Batch Distribution Contract

use soroban_sdk::{Env, Address, Symbol};

// Key prefixes
const INIT_KEY: &str = "init";
const TOKEN_OWNER_KEY: &str = "owner"; // owner:{token_id}
const OWNER_BAL_KEY: &str = "balance"; // balance:{address}
const TOTAL_SUPPLY_KEY: &str = "supply";
const TOKEN_ID_KEY: &str = "token_id"; // for incrementing unique token ids

// Initialization guard
pub fn is_initialized(env: &Env) -> bool {
    env.storage().has(&Symbol::short(INIT_KEY))
}

pub fn set_initialized(env: &Env) {
    env.storage().set(&Symbol::short(INIT_KEY), &true);
}

// Token ID counter (for unique NFT IDs)
pub fn next_token_id(env: &Env) -> u128 {
    let key = Symbol::short(TOKEN_ID_KEY);
    let id: u128 = env.storage().get(&key).unwrap_or(0);
    env.storage().set(&key, &(id + 1));
    id + 1
}

// Set token owner
pub fn set_token_owner(env: &Env, token_id: u128, owner: &Address) {
    let key = (Symbol::short(TOKEN_OWNER_KEY), token_id);
    env.storage().set(&key, owner);
}

// Get token owner
pub fn get_token_owner(env: &Env, token_id: u128) -> Option<Address> {
    let key = (Symbol::short(TOKEN_OWNER_KEY), token_id);
    env.storage().get(&key)
}

// Set owner balance (number of NFTs owned)
pub fn set_owner_balance(env: &Env, owner: &Address, balance: u32) {
    let key = (Symbol::short(OWNER_BAL_KEY), owner.clone());
    env.storage().set(&key, &balance);
}

// Get owner balance
pub fn get_owner_balance(env: &Env, owner: &Address) -> u32 {
    let key = (Symbol::short(OWNER_BAL_KEY), owner.clone());
    env.storage().get(&key).unwrap_or(0)
}

// Increment total supply
pub fn increment_total_supply(env: &Env, amount: u32) {
    let key = Symbol::short(TOTAL_SUPPLY_KEY);
    let supply: u32 = env.storage().get(&key).unwrap_or(0);
    env.storage().set(&key, &(supply + amount));
}

// Get total supply
pub fn get_total_supply(env: &Env) -> u32 {
    let key = Symbol::short(TOTAL_SUPPLY_KEY);
    env.storage().get(&key).unwrap_or(0)
}