use soroban_sdk::{Env};
use crate::storage;



pub fn initialize_token_redemption(env: &Env, token_id: u32) {
    let redeemed_key = (storage::REDEEMED_KEY, token_id);
    env.storage().persistent().set(&redeemed_key, &false);
}