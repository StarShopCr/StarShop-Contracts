use soroban_sdk::{Env, Address};
use crate::storage;
use crate::events;

pub fn transfer(env: &Env, from: &Address, to: &Address, token_id: u64) {
    let owner = storage::get_token_owner(env, token_id).expect("token does not exist");
    if &owner != from {
        panic!("from is not owner");
    }
    storage::set_token_owner(env, token_id, to);
    events::emit_transfer(env, from, to, token_id);
}
