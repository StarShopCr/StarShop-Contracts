use crate::{roles, storage};
use soroban_sdk::{Address, Env, panic_with_error};
use stellar_tokens::non_fungible::Base;

pub fn initialize_token_redemption(env: &Env, token_id: u32) {
    let redeemed_key = (storage::REDEEMED_KEY, token_id);
    env.storage().persistent().set(&redeemed_key, &false);
}

pub fn redeem_token(env: &Env, caller: &Address, token_id: u32) {
    // Verify token exists by checking if it has an owner
    let owner = Base::owner_of(env, token_id);

    // Verify caller authorization
    let is_owner = caller == &owner;
    let has_redeemer_role = roles::has_role(env, storage::REDEEMER_ROLE, &caller);

    if !is_owner && !has_redeemer_role {
        panic_with_error!(env, storage::ContractError::MissingRole);
    }

    // Check if already redeemed
    if is_token_redeemed(env, token_id) {
        panic_with_error!(env, storage::ContractError::AlreadyRedeemed);
    }

    // Mark as redeemed
    let redeemed_key = (storage::REDEEMED_KEY, token_id);
    env.storage().persistent().set(&redeemed_key, &true);

    // Store redemption timestamp
    let timestamp = env.ledger().timestamp();
    let redeem_time_key = (storage::REDEEM_TIME_KEY, token_id);
    env.storage().persistent().set(&redeem_time_key, &timestamp);

    // Emit redemption event
    env.events()
        .publish((storage::REDEEMED_EVENT,), (token_id, caller, timestamp));
}

pub fn is_token_redeemed(env: &Env, token_id: u32) -> bool {
    let redeemed_key = (storage::REDEEMED_KEY, token_id);
    env.storage()
        .persistent()
        .get(&redeemed_key)
        .unwrap_or(false)
}

pub fn get_redemption_timestamp(env: &Env, token_id: u32) -> Option<u64> {
    let redeem_time_key = (storage::REDEEM_TIME_KEY, token_id);
    env.storage().persistent().get(&redeem_time_key)
}
